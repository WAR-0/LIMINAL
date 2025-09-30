use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentRole {
    Systems,
    Interface,
    Router,
    Testing,
    Research,
    Director,
}

impl AgentRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "systems" => Some(AgentRole::Systems),
            "interface" => Some(AgentRole::Interface),
            "router" => Some(AgentRole::Router),
            "testing" => Some(AgentRole::Testing),
            "research" => Some(AgentRole::Research),
            "director" => Some(AgentRole::Director),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TurnStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub id: usize,
    pub specialist: AgentRole,
    pub prompt: String,
    pub acceptance_criteria: Vec<String>,
    pub parallel_group: Option<usize>,
    pub dependencies: Vec<usize>,
    pub status: TurnStatus,
    pub metadata: HashMap<String, String>,
}

impl Turn {
    pub fn new(id: usize, specialist: AgentRole, prompt: String) -> Self {
        Self {
            id,
            specialist,
            prompt,
            acceptance_criteria: Vec::new(),
            parallel_group: None,
            dependencies: Vec::new(),
            status: TurnStatus::Pending,
            metadata: HashMap::new(),
        }
    }

    pub fn with_acceptance(mut self, criteria: Vec<String>) -> Self {
        self.acceptance_criteria = criteria;
        self
    }

    pub fn with_parallel_group(mut self, group: Option<usize>) -> Self {
        self.parallel_group = group;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub epoch_id: String,
    pub goal: String,
    pub turns: Vec<Turn>,
    pub metadata: HashMap<String, String>,
}

impl Runbook {
    pub fn new(epoch_id: String, goal: String) -> Self {
        Self {
            epoch_id,
            goal,
            turns: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_turn(&mut self, turn: Turn) {
        self.turns.push(turn);
    }

    pub fn build_dependency_graph(&mut self) {
        let mut parallel_groups: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut sequential_turns: Vec<usize> = Vec::new();
        let all_turn_ids: Vec<usize> = self.turns.iter().map(|t| t.id).collect();

        for turn in &self.turns {
            if let Some(group) = turn.parallel_group {
                parallel_groups.entry(group).or_default().push(turn.id);
            } else {
                sequential_turns.push(turn.id);
            }
        }

        for turn in &mut self.turns {
            turn.dependencies.clear();

            if let Some(current_group) = turn.parallel_group {
                for (&group, turn_ids) in &parallel_groups {
                    if group < current_group {
                        turn.dependencies.extend(turn_ids);
                    }
                }
                for &seq_id in &sequential_turns {
                    if seq_id < turn.id {
                        turn.dependencies.push(seq_id);
                    }
                }
            } else {
                for &other_id in &all_turn_ids {
                    if other_id < turn.id {
                        turn.dependencies.push(other_id);
                    }
                }
            }

            turn.dependencies.sort_unstable();
            turn.dependencies.dedup();
        }
    }

    pub fn get_executable_turns(&self) -> Vec<&Turn> {
        let completed: std::collections::HashSet<_> = self
            .turns
            .iter()
            .filter(|t| t.status == TurnStatus::Completed)
            .map(|t| t.id)
            .collect();

        self.turns
            .iter()
            .filter(|turn| {
                turn.status == TurnStatus::Pending
                    && turn.dependencies.iter().all(|dep| completed.contains(dep))
            })
            .collect()
    }
}
