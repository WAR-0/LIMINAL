use crate::config::{parse_duration as parse_duration_str, RouterConfig};
use crate::metrics::MetricsCollector;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{broadcast, watch, Mutex, Notify, RwLock};
use tokio::task::JoinHandle;

const PRIORITY_LEVELS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Info = 0,
    Coordinate = 1,
    Blocking = 2,
    Critical = 3,
    DirectorOverride = 4,
}

impl Priority {
    pub fn as_index(self) -> usize {
        self as usize
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Priority::Info => "info",
            Priority::Coordinate => "coordinate",
            Priority::Blocking => "blocking",
            Priority::Critical => "critical",
            Priority::DirectorOverride => "directorOverride",
        }
    }

    pub fn token_cost(self) -> f64 {
        match self {
            Priority::Info => 1.0,
            Priority::Coordinate => 5.0,
            Priority::Blocking => 20.0,
            Priority::Critical => 100.0,
            Priority::DirectorOverride => 0.0,
        }
    }

    pub fn boost(self, levels: u8) -> Self {
        let target = (self.as_index() + levels as usize).min(Priority::Critical.as_index());
        Self::from_index(target)
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Priority::Info,
            1 => Priority::Coordinate,
            2 => Priority::Blocking,
            3 => Priority::Critical,
            _ => Priority::DirectorOverride,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub content: String,
    pub priority: Priority,
    pub sender: String,
    pub recipient: String,
}

#[derive(Clone, Debug)]
struct QueuedMessage {
    message: Message,
    enqueued_at: Instant,
    effective_priority: Priority,
    aging_boosts: u8,
    retry_count: u32,
    last_attempt_at: Option<Instant>,
}

impl QueuedMessage {
    fn new(message: Message) -> Self {
        Self {
            effective_priority: message.priority,
            message,
            enqueued_at: Instant::now(),
            aging_boosts: 0,
            retry_count: 0,
            last_attempt_at: None,
        }
    }

    fn eligible_for_boost(&self, threshold: Duration, max_boosts: u8) -> bool {
        self.aging_boosts < max_boosts && self.enqueued_at.elapsed() >= threshold
    }

    fn record_attempt(&mut self) {
        self.retry_count += 1;
        self.last_attempt_at = Some(Instant::now());
    }
}

#[derive(Clone, Debug)]
pub struct RouterDelivery {
    pub message: Message,
    pub effective_priority: Priority,
    pub wait_time: Duration,
    pub queue_depths: [usize; PRIORITY_LEVELS],
    pub aging_boosts: u8,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct DispatcherConfig {
    pub aging_threshold: Duration,
    pub max_aging_boosts: u8,
    pub idle_backoff: Duration,
    pub token_capacity: f64,
    pub token_refill_rate: f64,
    pub initial_tokens: f64,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            aging_threshold: Duration::from_millis(500),
            max_aging_boosts: 2,
            idle_backoff: Duration::from_millis(5),
            token_capacity: 200.0,
            token_refill_rate: 60.0,
            initial_tokens: 200.0,
        }
    }
}

impl DispatcherConfig {
    pub fn from_router_config(config: Option<&RouterConfig>) -> Self {
        let mut current = Self::default();
        if let Some(cfg) = config {
            if let Some(capacity) = cfg.token_bucket_capacity {
                current.token_capacity = capacity;
            }
            if let Some(refill) = cfg.token_bucket_refill_rate {
                current.token_refill_rate = refill;
            }
            if let Some(initial) = cfg.token_bucket_initial {
                current.initial_tokens = initial;
            } else if cfg.token_bucket_capacity.is_some() {
                current.initial_tokens = current.token_capacity;
            }
            if let Some(duration) = cfg.aging_threshold.as_deref().and_then(parse_duration_str) {
                current.aging_threshold = duration;
            }
            if let Some(boosts) = cfg.max_aging_boosts {
                current.max_aging_boosts = boosts;
            }
            if let Some(duration) = cfg.idle_backoff.as_deref().and_then(parse_duration_str) {
                current.idle_backoff = duration;
            }
        }
        if current.initial_tokens > current.token_capacity {
            current.initial_tokens = current.token_capacity;
        }
        current
    }
}

#[derive(Debug, Clone)]
struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64, initial: f64) -> Self {
        let tokens = initial.min(capacity);
        Self {
            capacity,
            tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self, cost: f64) -> bool {
        self.refill();
        if self.tokens >= cost {
            self.tokens -= cost;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
            self.last_refill = Instant::now();
        }
    }
}

#[derive(Debug)]
pub enum RouteError {
    RouterShuttingDown,
}

pub struct UnifiedMessageRouter {
    queues: Vec<Arc<RwLock<VecDeque<QueuedMessage>>>>,
    notify: Arc<Notify>,
    token_buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    metrics: MetricsCollector,
    dispatcher: Mutex<Option<JoinHandle<()>>>,
    shutdown: watch::Sender<bool>,
    deliveries: broadcast::Sender<RouterDelivery>,
    config: DispatcherConfig,
}

impl UnifiedMessageRouter {
    pub fn new() -> Self {
        Self::with_config(MetricsCollector::new(), DispatcherConfig::default())
    }

    pub fn with_metrics(metrics: MetricsCollector) -> Self {
        Self::with_config(metrics, DispatcherConfig::default())
    }

    pub fn with_settings(metrics: MetricsCollector, router_config: Option<&RouterConfig>) -> Self {
        let dispatcher_config = DispatcherConfig::from_router_config(router_config);
        Self::with_config(metrics, dispatcher_config)
    }

    pub fn with_config(metrics: MetricsCollector, config: DispatcherConfig) -> Self {
        let queues = (0..PRIORITY_LEVELS)
            .map(|_| Arc::new(RwLock::new(VecDeque::new())))
            .collect();
        let notify = Arc::new(Notify::new());
        let token_buckets = Arc::new(RwLock::new(HashMap::new()));
        let (shutdown, _) = watch::channel(false);
        let (deliveries, _) = broadcast::channel(256);
        Self {
            queues,
            notify,
            token_buckets,
            metrics,
            dispatcher: Mutex::new(None),
            shutdown,
            deliveries,
            config,
        }
    }

    pub fn dispatcher_config(&self) -> DispatcherConfig {
        self.config
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RouterDelivery> {
        self.deliveries.subscribe()
    }

    pub async fn route_message(&self, msg: Message) -> Result<(), RouteError> {
        if *self.shutdown.borrow() {
            return Err(RouteError::RouterShuttingDown);
        }
        self.ensure_dispatcher_started().await;
        let queued = QueuedMessage::new(msg);
        let index = queued.effective_priority.as_index();
        let mut queue = self.queues[index].write().await;
        queue.push_back(queued);
        drop(queue);
        let depths = queue_depths(&self.queues).await;
        self.metrics.update_queue_depths(&depths);
        self.notify.notify_one();
        Ok(())
    }

    pub async fn get_pending_messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();
        for priority in (0..self.queues.len()).rev() {
            let queue = self.queues[priority].read().await;
            messages.extend(queue.iter().map(|queued| queued.message.clone()));
        }
        messages
    }

    async fn ensure_dispatcher_started(&self) {
        let mut guard = self.dispatcher.lock().await;
        if guard.is_some() {
            return;
        }
        let queues = self.queues.iter().cloned().collect::<Vec<_>>();
        let notify = Arc::clone(&self.notify);
        let token_buckets = Arc::clone(&self.token_buckets);
        let metrics = self.metrics.clone();
        let deliveries = self.deliveries.clone();
        let mut shutdown_rx = self.shutdown.subscribe();
        let config = self.config;
        let handle = tokio::spawn(async move {
            run_dispatcher(
                queues,
                notify,
                token_buckets,
                metrics,
                deliveries,
                config,
                &mut shutdown_rx,
            )
            .await;
        });
        *guard = Some(handle);
    }
}

impl Drop for UnifiedMessageRouter {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
        if let Ok(mut guard) = self.dispatcher.try_lock() {
            if let Some(handle) = guard.take() {
                handle.abort();
            }
        }
    }
}

async fn run_dispatcher(
    queues: Vec<Arc<RwLock<VecDeque<QueuedMessage>>>>,
    notify: Arc<Notify>,
    token_buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    metrics: MetricsCollector,
    deliveries: broadcast::Sender<RouterDelivery>,
    config: DispatcherConfig,
    shutdown_rx: &mut watch::Receiver<bool>,
) {
    loop {
        if *shutdown_rx.borrow() {
            break;
        }
        apply_aging(&queues, config).await;
        let mut dispatched = false;
        for priority in (0..queues.len()).rev() {
            let maybe_message = {
                let mut queue = queues[priority].write().await;
                queue.pop_front()
            };
            if let Some(mut queued) = maybe_message {
                let sender_id = queued.message.sender.clone();
                let (should_dispatch, tokens_remaining, capacity, refill_rate, last_refill_elapsed) = {
                    let mut buckets = token_buckets.write().await;
                    let bucket = buckets.entry(sender_id.clone()).or_insert_with(|| {
                        TokenBucket::new(
                            config.token_capacity,
                            config.token_refill_rate,
                            config.initial_tokens,
                        )
                    });
                    let dispatched = bucket.try_consume(queued.effective_priority.token_cost());
                    let tokens_remaining = bucket.tokens;
                    let capacity = bucket.capacity;
                    let refill_rate = bucket.refill_rate;
                    let last_refill_elapsed = bucket.last_refill.elapsed();
                    (
                        dispatched,
                        tokens_remaining,
                        capacity,
                        refill_rate,
                        last_refill_elapsed,
                    )
                };
                let now = SystemTime::now();
                let last_refill = now.checked_sub(last_refill_elapsed).unwrap_or(now);
                metrics.update_token_bucket(
                    &sender_id,
                    tokens_remaining,
                    capacity,
                    refill_rate,
                    Some(last_refill),
                );
                if !should_dispatch {
                    metrics.increment_rate_limited(&sender_id);
                    queued.record_attempt();
                    let index = queued.effective_priority.as_index();
                    let mut queue = queues[index].write().await;
                    queue.push_back(queued);
                    drop(queue);
                    let depths = queue_depths(&queues).await;
                    metrics.update_queue_depths(&depths);
                    continue;
                }
                let wait_time = queued.enqueued_at.elapsed();
                let queue_depths = queue_depths(&queues).await;
                let delivery = RouterDelivery {
                    message: queued.message.clone(),
                    effective_priority: queued.effective_priority,
                    wait_time,
                    queue_depths,
                    aging_boosts: queued.aging_boosts,
                    retry_count: queued.retry_count,
                };
                let _ = deliveries.send(delivery.clone());
                metrics.record_router_delivery(
                    queued.effective_priority,
                    wait_time,
                    &delivery.queue_depths,
                );
                metrics.update_queue_depths(&delivery.queue_depths);
                dispatched = true;
                break;
            }
        }
        if !dispatched {
            tokio::select! {
                _ = notify.notified() => {}
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
                _ = tokio::time::sleep(config.idle_backoff) => {}
            }
        }
    }
}

async fn queue_depths(queues: &[Arc<RwLock<VecDeque<QueuedMessage>>>]) -> [usize; PRIORITY_LEVELS] {
    let mut depths = [0usize; PRIORITY_LEVELS];
    for (index, queue) in queues.iter().enumerate() {
        depths[index] = queue.read().await.len();
    }
    depths
}

async fn apply_aging(queues: &[Arc<RwLock<VecDeque<QueuedMessage>>>], config: DispatcherConfig) {
    if queues.is_empty() {
        return;
    }
    for priority in 0..queues.len().saturating_sub(1) {
        let mut queue = queues[priority].write().await;
        let mut index = 0;
        while index < queue.len() {
            let should_boost = queue
                .get(index)
                .map(|queued| {
                    queued.eligible_for_boost(config.aging_threshold, config.max_aging_boosts)
                })
                .unwrap_or(false);
            if should_boost {
                if let Some(mut queued) = queue.remove(index) {
                    queued.effective_priority = queued.effective_priority.boost(1);
                    queued.aging_boosts += 1;
                    drop(queue);
                    let boosted_index = queued.effective_priority.as_index();
                    let mut boosted_queue = queues[boosted_index].write().await;
                    boosted_queue.push_back(queued);
                    drop(boosted_queue);
                    queue = queues[priority].write().await;
                    continue;
                }
            }
            index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_router_config() -> RouterConfig {
        RouterConfig {
            token_bucket_capacity: Some(512.0),
            token_bucket_refill_rate: Some(256.0),
            token_bucket_initial: Some(128.0),
            aging_threshold: Some("250ms".to_string()),
            max_aging_boosts: Some(5),
            idle_backoff: Some("15ms".to_string()),
            queue_depth_warning: Some(10),
            queue_depth_critical: Some(20),
        }
    }

    #[test]
    fn dispatcher_config_applies_overrides() {
        let overrides = build_router_config();
        let config = DispatcherConfig::from_router_config(Some(&overrides));
        assert_eq!(config.token_capacity, 512.0);
        assert_eq!(config.token_refill_rate, 256.0);
        assert_eq!(config.initial_tokens, 128.0);
        assert_eq!(config.max_aging_boosts, 5);
        assert_eq!(config.aging_threshold, Duration::from_millis(250));
        assert_eq!(config.idle_backoff, Duration::from_millis(15));
    }

    #[test]
    fn dispatcher_config_defaults_initial_tokens_to_capacity() {
        let overrides = RouterConfig {
            token_bucket_capacity: Some(300.0),
            token_bucket_refill_rate: None,
            token_bucket_initial: None,
            aging_threshold: None,
            max_aging_boosts: None,
            idle_backoff: None,
            queue_depth_warning: None,
            queue_depth_critical: None,
        };
        let config = DispatcherConfig::from_router_config(Some(&overrides));
        assert_eq!(config.token_capacity, 300.0);
        assert_eq!(config.initial_tokens, 300.0);
    }
}
