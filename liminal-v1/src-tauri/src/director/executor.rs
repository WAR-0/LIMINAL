use super::orchestrator::{DirectorAgent, OrchestratorError, RunbookSummary, TurnUpdate};
use super::runbook::{AgentRole, TurnStatus};
use crate::metrics::MetricsCollector;
use crate::router::UnifiedMessageRouter;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

const EVENT_CHANNEL_CAPACITY: usize = 100;
const STATUS_POLL_INTERVAL_MS: u64 = 500;

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Orchestrator error: {0}")]
    OrchestratorError(#[from] OrchestratorError),
    #[error("Already executing")]
    AlreadyExecuting,
    #[error("Not executing")]
    NotExecuting,
    #[error("No runbook loaded")]
    NoRunbookLoaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ExecutionEvent {
    RunbookStarted {
        epoch_id: String,
        total_turns: usize,
        timestamp: u64,
    },
    TurnStarted {
        turn_id: usize,
        specialist: AgentRole,
        timestamp: u64,
    },
    TurnCompleted {
        turn_id: usize,
        duration_ms: u64,
        artifacts_count: usize,
    },
    TurnFailed {
        turn_id: usize,
        error_message: String,
    },
    RunbookCompleted {
        total_duration_ms: u64,
        completed_turns: usize,
        failed_turns: usize,
    },
    RunbookFailed {
        error_message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnSummary {
    pub turn_id: usize,
    pub specialist: AgentRole,
    pub status: TurnStatus,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionSummary {
    pub epoch_id: String,
    pub total_turns: usize,
    pub completed_turns: usize,
    pub failed_turns: usize,
    pub total_duration_ms: u64,
    pub turn_summaries: Vec<TurnSummary>,
}

pub struct RunbookExecutor {
    orchestrator: Arc<DirectorAgent>,
    execution_handle: Arc<RwLock<Option<JoinHandle<Result<ExecutionSummary, ExecutorError>>>>>,
    event_tx: broadcast::Sender<ExecutionEvent>,
    start_time: Arc<RwLock<Option<std::time::Instant>>>,
    working_dir: PathBuf,
    metrics: MetricsCollector,
    router: Arc<UnifiedMessageRouter>,
}

impl RunbookExecutor {
    pub fn new(
        working_dir: PathBuf,
        metrics: MetricsCollector,
        router: UnifiedMessageRouter,
        max_parallel: usize,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        let orchestrator = DirectorAgent::new(working_dir.clone(), metrics.clone(), router)
            .with_max_parallel(max_parallel);

        Self {
            orchestrator: Arc::new(orchestrator),
            execution_handle: Arc::new(RwLock::new(None)),
            event_tx,
            start_time: Arc::new(RwLock::new(None)),
            working_dir,
            metrics,
            router: Arc::new(UnifiedMessageRouter::new()),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ExecutionEvent> {
        self.event_tx.subscribe()
    }

    pub async fn load_runbook(&self, path: &Path) -> Result<RunbookSummary, ExecutorError> {
        eprintln!("[Executor] Loading runbook from {:?}", path);
        let result = self
            .orchestrator
            .load_runbook(path)
            .await
            .map_err(ExecutorError::from);
        if let Ok(ref summary) = result {
            eprintln!(
                "[Executor] Runbook loaded: {} ({} turns)",
                summary.epoch_id, summary.total_turns
            );
        } else {
            eprintln!("[Executor] Failed to load runbook");
        }
        result
    }

    pub async fn execute(&mut self) -> Result<ExecutionSummary, ExecutorError> {
        {
            let handle_guard = self.execution_handle.read().unwrap();
            if handle_guard.is_some() {
                return Err(ExecutorError::AlreadyExecuting);
            }
        }

        let summary = self
            .orchestrator
            .get_summary()
            .ok_or(ExecutorError::NoRunbookLoaded)?;

        {
            let mut start_time = self.start_time.write().unwrap();
            *start_time = Some(std::time::Instant::now());
        }

        eprintln!(
            "[Executor] Starting execution: {} ({} turns)",
            summary.epoch_id, summary.total_turns
        );

        let _ = self.event_tx.send(ExecutionEvent::RunbookStarted {
            epoch_id: summary.epoch_id.clone(),
            total_turns: summary.total_turns,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        self.orchestrator.start_execution().await?;
        eprintln!("[Executor] Orchestrator started, monitoring execution...");

        let orchestrator = Arc::clone(&self.orchestrator);
        let event_tx = self.event_tx.clone();
        let start_time = Arc::clone(&self.start_time);

        let handle = tokio::spawn(async move {
            Self::monitor_execution(orchestrator, event_tx, start_time).await
        });

        {
            let mut handle_guard = self.execution_handle.write().unwrap();
            *handle_guard = Some(handle);
        }

        let result = {
            let mut handle_guard = self.execution_handle.write().unwrap();
            if let Some(handle) = handle_guard.take() {
                handle.await.unwrap_or_else(|e| {
                    Err(ExecutorError::OrchestratorError(
                        OrchestratorError::TurnExecutionFailed(e.to_string()),
                    ))
                })
            } else {
                Err(ExecutorError::NotExecuting)
            }
        };

        result
    }

    async fn monitor_execution(
        orchestrator: Arc<DirectorAgent>,
        event_tx: broadcast::Sender<ExecutionEvent>,
        start_time: Arc<RwLock<Option<std::time::Instant>>>,
    ) -> Result<ExecutionSummary, ExecutorError> {
        let mut last_status_map: std::collections::HashMap<usize, TurnStatus> =
            std::collections::HashMap::new();

        loop {
            tokio::time::sleep(Duration::from_millis(STATUS_POLL_INTERVAL_MS)).await;

            let turn_updates = orchestrator.get_turn_status();
            let summary = orchestrator
                .get_summary()
                .ok_or(ExecutorError::NoRunbookLoaded)?;

            for update in &turn_updates {
                let last_status = last_status_map.get(&update.turn_id);

                match (&update.status, last_status) {
                    (TurnStatus::InProgress, Some(TurnStatus::Pending) | None) => {
                        eprintln!(
                            "[Executor] Turn {} started ({:?})",
                            update.turn_id, update.specialist
                        );
                        let _ = event_tx.send(ExecutionEvent::TurnStarted {
                            turn_id: update.turn_id,
                            specialist: update.specialist.clone(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        });
                    }
                    (TurnStatus::Completed, Some(TurnStatus::InProgress)) => {
                        eprintln!(
                            "[Executor] Turn {} completed ({}ms)",
                            update.turn_id,
                            update.duration_ms.unwrap_or(0)
                        );
                        let _ = event_tx.send(ExecutionEvent::TurnCompleted {
                            turn_id: update.turn_id,
                            duration_ms: update.duration_ms.unwrap_or(0),
                            artifacts_count: 0,
                        });
                    }
                    (TurnStatus::Failed, Some(TurnStatus::InProgress)) => {
                        eprintln!(
                            "[Executor] Turn {} failed: {}",
                            update.turn_id,
                            update
                                .error_message
                                .as_ref()
                                .unwrap_or(&"Unknown error".to_string())
                        );
                        let _ = event_tx.send(ExecutionEvent::TurnFailed {
                            turn_id: update.turn_id,
                            error_message: update
                                .error_message
                                .clone()
                                .unwrap_or_else(|| "Unknown error".to_string()),
                        });
                    }
                    _ => {}
                }

                last_status_map.insert(update.turn_id, update.status.clone());
            }

            if summary.in_progress_turns == 0
                && (summary.completed_turns + summary.failed_turns) == summary.total_turns
            {
                let total_duration = {
                    let start = start_time.read().unwrap();
                    start.map(|s| s.elapsed().as_millis() as u64).unwrap_or(0)
                };

                eprintln!(
                    "[Executor] Runbook execution complete: {} completed, {} failed ({}ms)",
                    summary.completed_turns, summary.failed_turns, total_duration
                );

                if summary.failed_turns > 0 {
                    let _ = event_tx.send(ExecutionEvent::RunbookCompleted {
                        total_duration_ms: total_duration,
                        completed_turns: summary.completed_turns,
                        failed_turns: summary.failed_turns,
                    });
                } else {
                    let _ = event_tx.send(ExecutionEvent::RunbookCompleted {
                        total_duration_ms: total_duration,
                        completed_turns: summary.completed_turns,
                        failed_turns: summary.failed_turns,
                    });
                }

                let turn_summaries: Vec<TurnSummary> = turn_updates
                    .into_iter()
                    .map(|update| TurnSummary {
                        turn_id: update.turn_id,
                        specialist: update.specialist,
                        status: update.status,
                        duration_ms: update.duration_ms.unwrap_or(0),
                        error_message: update.error_message,
                    })
                    .collect();

                return Ok(ExecutionSummary {
                    epoch_id: summary.epoch_id,
                    total_turns: summary.total_turns,
                    completed_turns: summary.completed_turns,
                    failed_turns: summary.failed_turns,
                    total_duration_ms: total_duration,
                    turn_summaries,
                });
            }
        }
    }

    pub async fn cancel(&self, force: bool) -> Result<(), ExecutorError> {
        eprintln!("[Executor] Cancelling execution (force: {})", force);

        if force {
            eprintln!("[Executor] Force shutdown: terminating agents immediately");
            self.orchestrator.shutdown().await?;
        } else {
            eprintln!("[Executor] Graceful shutdown: pausing then shutting down");
            self.orchestrator.pause_execution().await?;
            tokio::time::sleep(Duration::from_secs(2)).await;
            self.orchestrator.shutdown().await?;
        }

        {
            let mut handle_guard = self.execution_handle.write().unwrap();
            if let Some(handle) = handle_guard.take() {
                handle.abort();
            }
        }

        eprintln!("[Executor] Cancellation complete");
        Ok(())
    }

    pub fn get_status(&self) -> Vec<TurnUpdate> {
        self.orchestrator.get_turn_status()
    }

    pub fn get_summary(&self) -> Option<RunbookSummary> {
        self.orchestrator.get_summary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let working_dir = PathBuf::from("/tmp/test");
        let metrics = MetricsCollector::new();
        let router = UnifiedMessageRouter::new();

        let _executor = RunbookExecutor::new(working_dir, metrics, router, 3);
    }

    #[test]
    fn test_event_subscription() {
        let working_dir = PathBuf::from("/tmp/test");
        let metrics = MetricsCollector::new();
        let router = UnifiedMessageRouter::new();

        let executor = RunbookExecutor::new(working_dir, metrics, router, 3);
        let _rx1 = executor.subscribe();
        let _rx2 = executor.subscribe();
    }

    #[tokio::test]
    async fn test_execute_without_runbook() {
        let working_dir = PathBuf::from("/tmp/test");
        let metrics = MetricsCollector::new();
        let router = UnifiedMessageRouter::new();

        let mut executor = RunbookExecutor::new(working_dir, metrics, router, 3);
        let result = executor.execute().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExecutorError::NoRunbookLoaded
        ));
    }

    #[tokio::test]
    async fn test_cancel_not_executing() {
        let working_dir = PathBuf::from("/tmp/test");
        let metrics = MetricsCollector::new();
        let router = UnifiedMessageRouter::new();

        let executor = RunbookExecutor::new(working_dir, metrics, router, 3);
        let result = executor.cancel(false).await;
        assert!(result.is_ok());
    }
}
