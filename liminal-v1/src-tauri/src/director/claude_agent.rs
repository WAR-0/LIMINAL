use super::runbook::{AgentRole, Turn, TurnStatus};
use crate::agent::{AgentEvent, AgentProcess};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

const TURN_TIMEOUT_SECS: u64 = 1800;

#[derive(Debug, Error)]
pub enum ClaudeAgentError {
    #[error("Failed to spawn Claude CLI: {0}")]
    SpawnFailed(String),
    #[error("Failed to send prompt: {0}")]
    PromptSendFailed(String),
    #[error("Turn execution timed out after {0}s")]
    TurnTimeout(u64),
    #[error("Agent not ready: {0}")]
    NotReady(String),
    #[error("Turn execution failed: {0}")]
    ExecutionFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Spawning,
    Ready,
    ExecutingTurn,
    Completed,
    Failed,
    ShuttingDown,
}

#[derive(Debug, Clone)]
pub struct TurnResult {
    pub turn_id: usize,
    pub status: TurnStatus,
    pub artifacts: Vec<PathBuf>,
    pub output_log: PathBuf,
    pub duration: Duration,
    pub error_message: Option<String>,
}

pub struct ClaudeCodeAgent {
    pub role: AgentRole,
    pty_process: Option<AgentProcess>,
    status: Arc<Mutex<AgentStatus>>,
    current_turn: Arc<Mutex<Option<Turn>>>,
    artifacts: Arc<Mutex<Vec<PathBuf>>>,
    event_receiver: Arc<Mutex<Option<UnboundedReceiver<AgentEvent>>>>,
    turn_start: Arc<Mutex<Option<Instant>>>,
    working_dir: PathBuf,
    output_buffer: Arc<Mutex<Vec<String>>>,
}

impl ClaudeCodeAgent {
    pub fn new(role: AgentRole, working_dir: PathBuf) -> Self {
        Self {
            role,
            pty_process: None,
            status: Arc::new(Mutex::new(AgentStatus::Idle)),
            current_turn: Arc::new(Mutex::new(None)),
            artifacts: Arc::new(Mutex::new(Vec::new())),
            event_receiver: Arc::new(Mutex::new(None)),
            turn_start: Arc::new(Mutex::new(None)),
            working_dir,
            output_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn spawn(
        &mut self,
        event_sender: UnboundedSender<AgentEvent>,
    ) -> Result<(), ClaudeAgentError> {
        {
            let mut status = self.status.lock().unwrap();
            *status = AgentStatus::Spawning;
        }

        let agent_id = format!("claude_{:?}", self.role).to_lowercase();

        let process = AgentProcess::spawn(
            &agent_id,
            vec!["claude", "--dangerously-skip-permissions", "--verbose"],
            event_sender,
        );

        self.pty_process = Some(process);

        {
            let mut status = self.status.lock().unwrap();
            *status = AgentStatus::Ready;
        }

        Ok(())
    }

    pub fn send_turn_prompt(&mut self, turn: &Turn) -> Result<(), ClaudeAgentError> {
        let status = self.status.lock().unwrap().clone();
        if status != AgentStatus::Ready && status != AgentStatus::Completed {
            return Err(ClaudeAgentError::NotReady(format!("{:?}", status)));
        }

        let process = self
            .pty_process
            .as_ref()
            .ok_or_else(|| ClaudeAgentError::NotReady("PTY process not spawned".to_string()))?;

        {
            let mut current_turn = self.current_turn.lock().unwrap();
            *current_turn = Some(turn.clone());
        }

        {
            let mut turn_start = self.turn_start.lock().unwrap();
            *turn_start = Some(Instant::now());
        }

        {
            let mut status = self.status.lock().unwrap();
            *status = AgentStatus::ExecutingTurn;
        }

        let formatted_prompt = self.format_turn_prompt(turn);

        process
            .send_command(&formatted_prompt)
            .map_err(|e| ClaudeAgentError::PromptSendFailed(e.to_string()))?;

        Ok(())
    }

    fn format_turn_prompt(&self, turn: &Turn) -> String {
        let mut prompt = String::new();

        prompt.push_str("=== LIMINAL TURN EXECUTION ===\n");
        prompt.push_str(&format!("Turn ID: {}\n", turn.id));
        prompt.push_str(&format!("Role: {:?}\n", turn.specialist));
        prompt.push('\n');

        prompt.push_str("PROMPT:\n");
        prompt.push_str(&turn.prompt);
        prompt.push_str("\n\n");

        if !turn.acceptance_criteria.is_empty() {
            prompt.push_str("ACCEPTANCE CRITERIA:\n");
            for criterion in &turn.acceptance_criteria {
                prompt.push_str(&format!("- {}\n", criterion));
            }
            prompt.push('\n');
        }

        prompt.push_str("When complete, respond with: TURN_COMPLETE\n");
        prompt.push_str("=== END TURN EXECUTION ===\n");

        prompt
    }

    pub async fn wait_for_completion(
        &self,
        timeout: Option<Duration>,
    ) -> Result<TurnResult, ClaudeAgentError> {
        let timeout_duration = timeout.unwrap_or(Duration::from_secs(TURN_TIMEOUT_SECS));
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout_duration {
                let mut status = self.status.lock().unwrap();
                *status = AgentStatus::Failed;
                return Err(ClaudeAgentError::TurnTimeout(timeout_duration.as_secs()));
            }

            let status = self.status.lock().unwrap().clone();

            match status {
                AgentStatus::Completed => {
                    return self.build_turn_result(TurnStatus::Completed, None);
                }
                AgentStatus::Failed => {
                    return self.build_turn_result(
                        TurnStatus::Failed,
                        Some("Agent failed during execution".to_string()),
                    );
                }
                AgentStatus::ExecutingTurn => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                _ => {
                    return Err(ClaudeAgentError::NotReady(format!(
                        "Unexpected status: {:?}",
                        status
                    )));
                }
            }
        }
    }

    fn build_turn_result(
        &self,
        status: TurnStatus,
        error: Option<String>,
    ) -> Result<TurnResult, ClaudeAgentError> {
        let turn = self
            .current_turn
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| ClaudeAgentError::NotReady("No current turn".to_string()))?;

        let start_time = self
            .turn_start
            .lock()
            .unwrap()
            .ok_or_else(|| ClaudeAgentError::NotReady("Turn start time not set".to_string()))?;

        let duration = start_time.elapsed();
        let artifacts = self.artifacts.lock().unwrap().clone();

        let output_log = self
            .working_dir
            .join(".uncan")
            .join(format!("{:?}", self.role).to_lowercase())
            .join("context")
            .join(format!("turn_{}_output.log", turn.id));

        Ok(TurnResult {
            turn_id: turn.id,
            status,
            artifacts,
            output_log,
            duration,
            error_message: error,
        })
    }

    pub fn check_completion(&self, output: &str) -> bool {
        output.contains("TURN_COMPLETE") || output.contains("Turn complete")
    }

    pub fn collect_artifacts(&mut self) -> Result<Vec<PathBuf>, ClaudeAgentError> {
        let output = std::process::Command::new("git")
            .args(["status", "--short"])
            .current_dir(&self.working_dir)
            .output()
            .map_err(|e| ClaudeAgentError::ExecutionFailed(format!("Git status failed: {}", e)))?;

        let status_output = String::from_utf8_lossy(&output.stdout);
        let mut artifacts = Vec::new();

        for line in status_output.lines() {
            if line.len() > 3 {
                let file_path = line[3..].trim();
                artifacts.push(self.working_dir.join(file_path));
            }
        }

        {
            let mut stored_artifacts = self.artifacts.lock().unwrap();
            *stored_artifacts = artifacts.clone();
        }

        Ok(artifacts)
    }

    pub fn save_output_log(&self) -> Result<PathBuf, ClaudeAgentError> {
        let turn = self
            .current_turn
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| ClaudeAgentError::NotReady("No current turn".to_string()))?;

        let output_dir = self
            .working_dir
            .join(".uncan")
            .join(format!("{:?}", self.role).to_lowercase())
            .join("context");

        std::fs::create_dir_all(&output_dir).map_err(|e| {
            ClaudeAgentError::ExecutionFailed(format!("Failed to create output dir: {}", e))
        })?;

        let output_path = output_dir.join(format!("turn_{}_output.log", turn.id));

        let buffer = self.output_buffer.lock().unwrap();
        let content = buffer.join("\n");

        std::fs::write(&output_path, content).map_err(|e| {
            ClaudeAgentError::ExecutionFailed(format!("Failed to write output log: {}", e))
        })?;

        Ok(output_path)
    }

    pub fn append_output(&self, line: String) {
        let mut buffer = self.output_buffer.lock().unwrap();
        buffer.push(line);
    }

    pub fn shutdown(&mut self, force: bool) -> Result<(), ClaudeAgentError> {
        {
            let mut status = self.status.lock().unwrap();
            *status = AgentStatus::ShuttingDown;
        }

        if let Some(process) = &self.pty_process {
            if force {
                process.send_command("\x03").map_err(|e| {
                    ClaudeAgentError::ExecutionFailed(format!("Failed to send SIGINT: {}", e))
                })?;
            } else {
                process.send_command("exit").map_err(|e| {
                    ClaudeAgentError::ExecutionFailed(format!("Failed to send exit: {}", e))
                })?;
            }
        }

        self.pty_process = None;

        {
            let mut status = self.status.lock().unwrap();
            *status = AgentStatus::Idle;
        }

        Ok(())
    }

    pub fn get_status(&self) -> AgentStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn get_current_turn(&self) -> Option<Turn> {
        self.current_turn.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;

    #[test]
    fn test_format_turn_prompt() {
        let agent = ClaudeCodeAgent::new(AgentRole::Systems, PathBuf::from("/tmp/test"));

        let turn = Turn::new(1, AgentRole::Systems, "Test prompt".to_string())
            .with_acceptance(vec!["Criterion 1".to_string(), "Criterion 2".to_string()]);

        let formatted = agent.format_turn_prompt(&turn);

        assert!(formatted.contains("Turn ID: 1"));
        assert!(formatted.contains("Role: Systems"));
        assert!(formatted.contains("Test prompt"));
        assert!(formatted.contains("Criterion 1"));
        assert!(formatted.contains("Criterion 2"));
        assert!(formatted.contains("TURN_COMPLETE"));
    }

    #[test]
    fn test_check_completion() {
        let agent = ClaudeCodeAgent::new(AgentRole::Systems, PathBuf::from("/tmp/test"));

        assert!(agent.check_completion("Task done. TURN_COMPLETE"));
        assert!(agent.check_completion("Turn complete successfully"));
        assert!(!agent.check_completion("Still working on it"));
    }

    #[test]
    fn test_agent_lifecycle() {
        let mut agent = ClaudeCodeAgent::new(AgentRole::Systems, PathBuf::from("/tmp/test"));

        assert_eq!(agent.get_status(), AgentStatus::Idle);

        let turn = Turn::new(1, AgentRole::Systems, "Test".to_string());
        let result = agent.send_turn_prompt(&turn);
        assert!(result.is_err());
    }
}
