use super::claude_agent::{AgentStatus, ClaudeAgentError, ClaudeCodeAgent, TurnResult};
use super::runbook::{AgentRole, Runbook, Turn, TurnStatus};
use super::session::Session;
use crate::metrics::MetricsCollector;
use crate::router::UnifiedMessageRouter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task::JoinHandle;

const DEFAULT_TURN_TIMEOUT_SECS: u64 = 1800;
const DEFAULT_MAX_PARALLEL: usize = 3;
const AGENT_SPAWN_RETRY_LIMIT: u32 = 1;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("Failed to load runbook: {0}")]
    RunbookLoadFailed(String),
    #[error("No runbook loaded")]
    NoRunbookLoaded,
    #[error("Agent spawn failed: {0}")]
    AgentSpawnFailed(#[from] ClaudeAgentError),
    #[error("Turn execution failed: {0}")]
    TurnExecutionFailed(String),
    #[error("Session error: {0}")]
    SessionError(String),
    #[error("Orchestrator is already executing")]
    AlreadyExecuting,
    #[error("Orchestrator is paused")]
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunbookSummary {
    pub epoch_id: String,
    pub goal: String,
    pub total_turns: usize,
    pub completed_turns: usize,
    pub failed_turns: usize,
    pub in_progress_turns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnUpdate {
    pub turn_id: usize,
    pub status: TurnStatus,
    pub specialist: AgentRole,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Escalation {
    pub turn_id: usize,
    pub reason: String,
    pub severity: String,
    pub timestamp: u64,
}

pub struct DirectorAgent {
    current_runbook: Arc<RwLock<Option<Runbook>>>,
    agents: Arc<RwLock<HashMap<AgentRole, ClaudeCodeAgent>>>,
    turn_status: Arc<RwLock<HashMap<usize, TurnExecutionState>>>,
    session: Arc<RwLock<Option<Session>>>,
    metrics: MetricsCollector,
    router: Arc<UnifiedMessageRouter>,
    working_dir: PathBuf,
    max_parallel: usize,
    turn_timeout: Duration,
    execution_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    paused: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
struct TurnExecutionState {
    status: TurnStatus,
    started_at: Option<Instant>,
    completed_at: Option<Instant>,
    error_message: Option<String>,
    retry_count: u32,
}

impl DirectorAgent {
    pub fn new(
        working_dir: PathBuf,
        metrics: MetricsCollector,
        router: UnifiedMessageRouter,
    ) -> Self {
        Self {
            current_runbook: Arc::new(RwLock::new(None)),
            agents: Arc::new(RwLock::new(HashMap::new())),
            turn_status: Arc::new(RwLock::new(HashMap::new())),
            session: Arc::new(RwLock::new(None)),
            metrics,
            router: Arc::new(router),
            working_dir,
            max_parallel: DEFAULT_MAX_PARALLEL,
            turn_timeout: Duration::from_secs(DEFAULT_TURN_TIMEOUT_SECS),
            execution_task: Arc::new(RwLock::new(None)),
            paused: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_max_parallel(mut self, max: usize) -> Self {
        self.max_parallel = max;
        self
    }

    pub fn with_turn_timeout(mut self, timeout: Duration) -> Self {
        self.turn_timeout = timeout;
        self
    }

    pub async fn load_runbook(&self, path: &Path) -> Result<RunbookSummary, OrchestratorError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| OrchestratorError::RunbookLoadFailed(e.to_string()))?;
        let parser = super::parser::RunbookParser::new(content);
        let mut runbook = parser
            .parse()
            .map_err(|e| OrchestratorError::RunbookLoadFailed(e.to_string()))?;

        runbook.build_dependency_graph();

        let summary = RunbookSummary {
            epoch_id: runbook.epoch_id.clone(),
            goal: runbook.goal.clone(),
            total_turns: runbook.turns.len(),
            completed_turns: 0,
            failed_turns: 0,
            in_progress_turns: 0,
        };

        {
            let mut current = self.current_runbook.write().unwrap();
            *current = Some(runbook.clone());
        }

        {
            let mut session_guard = self.session.write().unwrap();
            let session = Session::new(runbook.epoch_id.clone(), path.to_path_buf());
            *session_guard = Some(session);
        }

        Ok(summary)
    }

    pub async fn start_execution(&self) -> Result<(), OrchestratorError> {
        {
            let execution_guard = self.execution_task.read().unwrap();
            if execution_guard.is_some() {
                return Err(OrchestratorError::AlreadyExecuting);
            }
        }

        {
            let runbook_guard = self.current_runbook.read().unwrap();
            if runbook_guard.is_none() {
                return Err(OrchestratorError::NoRunbookLoaded);
            }
        }

        {
            let mut paused = self.paused.write().unwrap();
            *paused = false;
        }

        let current_runbook = Arc::clone(&self.current_runbook);
        let agents = Arc::clone(&self.agents);
        let turn_status = Arc::clone(&self.turn_status);
        let session = Arc::clone(&self.session);
        let metrics = self.metrics.clone();
        let router = Arc::clone(&self.router);
        let working_dir = self.working_dir.clone();
        let max_parallel = self.max_parallel;
        let turn_timeout = self.turn_timeout;
        let paused = Arc::clone(&self.paused);

        let handle = tokio::spawn(async move {
            let _ = Self::execute_runbook_loop(
                current_runbook,
                agents,
                turn_status,
                session,
                metrics,
                router,
                working_dir,
                max_parallel,
                turn_timeout,
                paused,
            )
            .await;
        });

        {
            let mut execution_guard = self.execution_task.write().unwrap();
            *execution_guard = Some(handle);
        }

        Ok(())
    }

    async fn execute_runbook_loop(
        current_runbook: Arc<RwLock<Option<Runbook>>>,
        agents: Arc<RwLock<HashMap<AgentRole, ClaudeCodeAgent>>>,
        turn_status: Arc<RwLock<HashMap<usize, TurnExecutionState>>>,
        session: Arc<RwLock<Option<Session>>>,
        metrics: MetricsCollector,
        router: Arc<UnifiedMessageRouter>,
        working_dir: PathBuf,
        max_parallel: usize,
        turn_timeout: Duration,
        paused: Arc<RwLock<bool>>,
    ) -> Result<(), OrchestratorError> {
        loop {
            if *paused.read().unwrap() {
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }

            let executable_turns: Vec<Turn> = {
                let runbook_guard = current_runbook.read().unwrap();
                if let Some(runbook) = runbook_guard.as_ref() {
                    runbook
                        .get_executable_turns()
                        .into_iter()
                        .cloned()
                        .collect()
                } else {
                    break;
                }
            };

            if executable_turns.is_empty() {
                let all_complete = {
                    let runbook_guard = current_runbook.read().unwrap();
                    if let Some(runbook) = runbook_guard.as_ref() {
                        runbook.turns.iter().all(|t| {
                            t.status == TurnStatus::Completed || t.status == TurnStatus::Failed
                        })
                    } else {
                        true
                    }
                };

                if all_complete {
                    Self::finalize_session(&session);
                    break;
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }

            let parallel_group = executable_turns.first().and_then(|t| t.parallel_group);
            let turns_to_execute: Vec<Turn> = if parallel_group.is_some() {
                executable_turns
                    .into_iter()
                    .filter(|t| t.parallel_group == parallel_group)
                    .take(max_parallel)
                    .collect()
            } else {
                executable_turns.into_iter().take(1).collect()
            };

            let mut handles = Vec::new();

            for turn in turns_to_execute {
                let agents_clone = Arc::clone(&agents);
                let turn_status_clone = Arc::clone(&turn_status);
                let session_clone = Arc::clone(&session);
                let current_runbook_clone = Arc::clone(&current_runbook);
                let metrics_clone = metrics.clone();
                let router_clone = Arc::clone(&router);
                let working_dir_clone = working_dir.clone();

                let handle = tokio::spawn(async move {
                    let result = Self::execute_turn(
                        &turn,
                        agents_clone,
                        turn_status_clone,
                        metrics_clone,
                        router_clone,
                        working_dir_clone,
                        turn_timeout,
                    )
                    .await;

                    Self::handle_turn_completion(
                        &turn,
                        result,
                        current_runbook_clone,
                        session_clone,
                    )
                    .await;
                });

                handles.push(handle);
            }

            for handle in handles {
                let _ = handle.await;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn execute_turn(
        turn: &Turn,
        agents: Arc<RwLock<HashMap<AgentRole, ClaudeCodeAgent>>>,
        turn_status: Arc<RwLock<HashMap<usize, TurnExecutionState>>>,
        metrics: MetricsCollector,
        router: Arc<UnifiedMessageRouter>,
        working_dir: PathBuf,
        timeout: Duration,
    ) -> Result<TurnResult, OrchestratorError> {
        let start_time = Instant::now();

        {
            let mut status_map = turn_status.write().unwrap();
            status_map.insert(
                turn.id,
                TurnExecutionState {
                    status: TurnStatus::InProgress,
                    started_at: Some(start_time),
                    completed_at: None,
                    error_message: None,
                    retry_count: 0,
                },
            );
        }

        let agent_spawn_start = Instant::now();
        let mut retry_count = 0;

        loop {
            let spawn_result = Self::get_or_spawn_agent(
                &turn.specialist,
                Arc::clone(&agents),
                working_dir.clone(),
            )
            .await;

            match spawn_result {
                Ok(_) => break,
                Err(e) => {
                    retry_count += 1;
                    if retry_count > AGENT_SPAWN_RETRY_LIMIT {
                        let mut status_map = turn_status.write().unwrap();
                        if let Some(state) = status_map.get_mut(&turn.id) {
                            state.status = TurnStatus::Failed;
                            state.error_message = Some(format!("Agent spawn failed: {}", e));
                            state.completed_at = Some(Instant::now());
                        }
                        return Err(OrchestratorError::AgentSpawnFailed(e));
                    }
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }
        }

        metrics.record_agent_spawn(agent_spawn_start.elapsed().as_millis() as f64);

        let send_result = {
            let mut agents_map = agents.write().unwrap();
            if let Some(agent_ref) = agents_map.get_mut(&turn.specialist) {
                agent_ref.send_turn_prompt(turn)
            } else {
                Err(ClaudeAgentError::NotReady("Agent not found".to_string()))
            }
        };

        if let Err(e) = send_result {
            let mut status_map = turn_status.write().unwrap();
            if let Some(state) = status_map.get_mut(&turn.id) {
                state.status = TurnStatus::Failed;
                state.error_message = Some(format!("Failed to send turn prompt: {}", e));
                state.completed_at = Some(Instant::now());
            }
            return Err(OrchestratorError::TurnExecutionFailed(e.to_string()));
        }

        let agent_exists = {
            let agents_map = agents.read().unwrap();
            agents_map.contains_key(&turn.specialist)
        };

        if !agent_exists {
            return Err(OrchestratorError::TurnExecutionFailed(
                "Agent not found".to_string(),
            ));
        }

        let result = loop {
            tokio::time::sleep(Duration::from_millis(500)).await;

            if start_time.elapsed() > timeout {
                break Err(ClaudeAgentError::TurnTimeout(timeout.as_secs()));
            }

            let status = {
                let agents_map = agents.read().unwrap();
                agents_map
                    .get(&turn.specialist)
                    .map(|agent| agent.get_status())
                    .unwrap_or(AgentStatus::Failed)
            };

            if status == AgentStatus::Completed || status == AgentStatus::Failed {
                let turn_result = {
                    let agents_map = agents.read().unwrap();
                    agents_map
                        .get(&turn.specialist)
                        .and_then(|agent| agent.get_current_turn())
                };

                if turn_result.is_some() {
                    let artifacts = {
                        let mut agents_map = agents.write().unwrap();
                        agents_map
                            .get_mut(&turn.specialist)
                            .map(|agent| agent.collect_artifacts().ok())
                            .flatten()
                            .unwrap_or_default()
                    };

                    let output_log = {
                        let agents_map = agents.read().unwrap();
                        agents_map
                            .get(&turn.specialist)
                            .and_then(|agent| agent.save_output_log().ok())
                            .unwrap_or_else(|| working_dir.join("output.log"))
                    };

                    if status == AgentStatus::Completed {
                        break Ok(TurnResult {
                            turn_id: turn.id,
                            status: TurnStatus::Completed,
                            artifacts,
                            output_log,
                            duration: start_time.elapsed(),
                            error_message: None,
                        });
                    } else {
                        break Ok(TurnResult {
                            turn_id: turn.id,
                            status: TurnStatus::Failed,
                            artifacts,
                            output_log,
                            duration: start_time.elapsed(),
                            error_message: Some("Agent failed during execution".to_string()),
                        });
                    }
                } else {
                    break Err(ClaudeAgentError::NotReady("Turn not found".to_string()));
                }
            }
        };

        result.map_err(|e| OrchestratorError::TurnExecutionFailed(e.to_string()))
    }

    async fn get_or_spawn_agent(
        role: &AgentRole,
        agents: Arc<RwLock<HashMap<AgentRole, ClaudeCodeAgent>>>,
        working_dir: PathBuf,
    ) -> Result<(), ClaudeAgentError> {
        {
            let agents_map = agents.read().unwrap();
            if let Some(agent) = agents_map.get(role) {
                let status = agent.get_status();
                if status == AgentStatus::Ready
                    || status == AgentStatus::Completed
                    || status == AgentStatus::Idle
                {
                    return Ok(());
                }
            }
        }

        let mut new_agent = ClaudeCodeAgent::new(role.clone(), working_dir);
        let (tx, _rx) = unbounded_channel();
        new_agent.spawn(tx)?;

        {
            let mut agents_map = agents.write().unwrap();
            agents_map.insert(role.clone(), new_agent);
        }

        Ok(())
    }

    async fn handle_turn_completion(
        turn: &Turn,
        result: Result<TurnResult, OrchestratorError>,
        current_runbook: Arc<RwLock<Option<Runbook>>>,
        session: Arc<RwLock<Option<Session>>>,
    ) {
        let new_status = match &result {
            Ok(turn_result) => turn_result.status.clone(),
            Err(_) => TurnStatus::Failed,
        };

        {
            let mut runbook_guard = current_runbook.write().unwrap();
            if let Some(runbook) = runbook_guard.as_mut() {
                if let Some(turn_ref) = runbook.turns.iter_mut().find(|t| t.id == turn.id) {
                    turn_ref.status = new_status.clone();
                }
            }
        }

        {
            let mut session_guard = session.write().unwrap();
            if let Some(sess) = session_guard.as_mut() {
                if let Ok(turn_result) = result {
                    sess.record_turn_completion(turn.id, turn_result);
                }
                let _ = sess.save();
            }
        }
    }

    fn finalize_session(session: &Arc<RwLock<Option<Session>>>) {
        let mut session_guard = session.write().unwrap();
        if let Some(sess) = session_guard.as_mut() {
            sess.finalize();
            let _ = sess.save();
        }
    }

    pub fn get_turn_status(&self) -> Vec<TurnUpdate> {
        let status_map = self.turn_status.read().unwrap();
        let mut updates = Vec::new();

        let runbook_guard = self.current_runbook.read().unwrap();
        if let Some(runbook) = runbook_guard.as_ref() {
            for turn in &runbook.turns {
                let state = status_map.get(&turn.id);

                updates.push(TurnUpdate {
                    turn_id: turn.id,
                    status: state
                        .map(|s| s.status.clone())
                        .unwrap_or(TurnStatus::Pending),
                    specialist: turn.specialist.clone(),
                    started_at: state
                        .and_then(|s| s.started_at.map(|t| t.elapsed().as_millis() as u64)),
                    completed_at: state
                        .and_then(|s| s.completed_at.map(|t| t.elapsed().as_millis() as u64)),
                    duration_ms: state.and_then(|s| {
                        s.started_at.and_then(|start| {
                            s.completed_at
                                .map(|end| end.duration_since(start).as_millis() as u64)
                        })
                    }),
                    error_message: state.and_then(|s| s.error_message.clone()),
                });
            }
        }

        updates
    }

    pub fn get_summary(&self) -> Option<RunbookSummary> {
        let runbook_guard = self.current_runbook.read().unwrap();
        runbook_guard.as_ref().map(|runbook| {
            let completed = runbook
                .turns
                .iter()
                .filter(|t| t.status == TurnStatus::Completed)
                .count();
            let failed = runbook
                .turns
                .iter()
                .filter(|t| t.status == TurnStatus::Failed)
                .count();
            let in_progress = runbook
                .turns
                .iter()
                .filter(|t| t.status == TurnStatus::InProgress)
                .count();

            RunbookSummary {
                epoch_id: runbook.epoch_id.clone(),
                goal: runbook.goal.clone(),
                total_turns: runbook.turns.len(),
                completed_turns: completed,
                failed_turns: failed,
                in_progress_turns: in_progress,
            }
        })
    }

    pub async fn pause_execution(&self) -> Result<(), OrchestratorError> {
        let mut paused = self.paused.write().unwrap();
        *paused = true;
        Ok(())
    }

    pub async fn resume_execution(&self) -> Result<(), OrchestratorError> {
        let paused_state = {
            let paused = self.paused.read().unwrap();
            *paused
        };

        if !paused_state {
            return Ok(());
        }

        let mut paused = self.paused.write().unwrap();
        *paused = false;
        Ok(())
    }

    pub fn handle_escalation(&self, _escalation: Escalation) -> Result<(), OrchestratorError> {
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), OrchestratorError> {
        {
            let mut paused = self.paused.write().unwrap();
            *paused = true;
        }

        {
            let mut execution_guard = self.execution_task.write().unwrap();
            if let Some(handle) = execution_guard.take() {
                handle.abort();
            }
        }

        {
            let mut agents_map = self.agents.write().unwrap();
            for (_, agent) in agents_map.iter_mut() {
                let _ = agent.shutdown(false);
            }
            agents_map.clear();
        }

        Ok(())
    }
}
