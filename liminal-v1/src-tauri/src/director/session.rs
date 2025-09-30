use super::claude_agent::TurnResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionState {
    Created,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnRecord {
    pub turn_id: usize,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub duration_ms: u64,
    pub artifacts: Vec<PathBuf>,
    pub output_log: PathBuf,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub session_id: String,
    pub epoch_id: String,
    pub runbook_path: PathBuf,
    pub state: SessionState,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub turn_records: HashMap<usize, TurnRecord>,
}

impl Session {
    pub fn new(epoch_id: String, runbook_path: PathBuf) -> Self {
        let session_id = Self::generate_session_id(&epoch_id);
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            session_id,
            epoch_id,
            runbook_path,
            state: SessionState::Created,
            created_at,
            started_at: None,
            completed_at: None,
            turn_records: HashMap::new(),
        }
    }

    pub fn load(session_path: &PathBuf) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(session_path)?;
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse session: {}", e),
            )
        })
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let session_dir = PathBuf::from(".uncan/director/sessions");
        fs::create_dir_all(&session_dir)?;

        let session_path = session_dir.join(format!("{}.json", self.session_id));
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize session: {}", e),
            )
        })?;

        fs::write(session_path, content)?;
        Ok(())
    }

    pub fn start(&mut self) {
        self.state = SessionState::Running;
        if self.started_at.is_none() {
            self.started_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
        }
    }

    pub fn pause(&mut self) {
        self.state = SessionState::Paused;
    }

    pub fn resume(&mut self) {
        self.state = SessionState::Running;
    }

    pub fn finalize(&mut self) {
        self.state = SessionState::Completed;
        self.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    pub fn record_turn_completion(&mut self, turn_id: usize, result: TurnResult) {
        let completed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record = TurnRecord {
            turn_id,
            started_at: completed_at - result.duration.as_secs(),
            completed_at: Some(completed_at),
            duration_ms: result.duration.as_millis() as u64,
            artifacts: result.artifacts,
            output_log: result.output_log,
            error_message: result.error_message,
        };

        self.turn_records.insert(turn_id, record);
    }

    fn generate_session_id(epoch_id: &str) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("{}_{}", epoch_id, timestamp)
    }

    pub fn get_turn_record(&self, turn_id: usize) -> Option<&TurnRecord> {
        self.turn_records.get(&turn_id)
    }

    pub fn total_duration_ms(&self) -> Option<u64> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            Some((completed - started) * 1000)
        } else {
            None
        }
    }
}
