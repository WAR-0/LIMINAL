use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type ResourcePath = String;
pub type AgentId = String;

#[derive(Default)]
pub struct TerritoryManager {
    leases: Arc<RwLock<HashMap<ResourcePath, AgentId>>>,
}

impl TerritoryManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn acquire_lease(&self, agent_id: &AgentId, resource: &ResourcePath) -> bool {
        let mut leases = self.leases.write().unwrap();
        if leases.contains_key(resource) {
            return false;
        }
        leases.insert(resource.clone(), agent_id.clone());
        true
    }

    pub fn release_lease(&self, agent_id: &AgentId, resource: &ResourcePath) {
        let mut leases = self.leases.write().unwrap();
        if let Some(holder) = leases.get(resource) {
            if holder == agent_id {
                leases.remove(resource);
            }
        }
    }
}
