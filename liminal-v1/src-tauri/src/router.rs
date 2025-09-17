use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Info = 0,       // Telemetry, logs
    Coordinate = 1, // Tasking, routine operations
    Blocking = 2,   // Sender blocked
    Critical = 3,   // System failures, alerts
}

#[derive(Clone)]
pub struct Message {
    pub content: String,
    pub priority: Priority,
    pub sender: String,
    pub recipient: String,
}

pub struct UnifiedMessageRouter {
    pub message_queues: [Arc<RwLock<VecDeque<Message>>>; 4],
}

impl UnifiedMessageRouter {
    pub fn new() -> Self {
        Self {
            message_queues: [
                Arc::new(RwLock::new(VecDeque::new())),
                Arc::new(RwLock::new(VecDeque::new())),
                Arc::new(RwLock::new(VecDeque::new())),
                Arc::new(RwLock::new(VecDeque::new())),
            ],
        }
    }

    pub async fn route_message(&self, msg: Message) {
        let queue_idx = msg.priority as usize;
        let mut queue = self.message_queues[queue_idx].write().await;
        queue.push_back(msg);
    }

    pub async fn get_pending_messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();

        for priority in (0..4).rev() {
            let queue = self.message_queues[priority].read().await;
            messages.extend(queue.iter().cloned());
        }

        messages
    }
}
