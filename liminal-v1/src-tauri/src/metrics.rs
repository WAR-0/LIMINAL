use crate::router::Priority;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceMetrics {
    pub message_routing_latency_ms: f64,
    pub agent_spawn_time_ms: f64,
    pub lease_acquisition_time_ms: f64,
    pub total_messages_routed: u64,
    pub total_leases_acquired: u64,
    pub memory_usage_mb: f64,
    pub rate_limited_messages: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RouterSnapshot {
    pub queue_depths: BTreeMap<String, usize>,
    pub last_dispatched_priority: Option<String>,
    pub last_dispatched_at: Option<SystemTime>,
    pub rate_limited_messages: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitSnapshot {
    pub sender: String,
    pub tokens_remaining: f64,
    pub capacity: f64,
    pub refill_rate: f64,
    pub last_refill: Option<SystemTime>,
    pub rate_limit_hits: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LeaseSnapshotSummary {
    pub active_leases: usize,
    pub total_pending: usize,
    pub pending_by_resource: BTreeMap<String, usize>,
    pub deferrals: u64,
    pub overrides: u64,
    pub escalations: u64,
    pub outstanding_lease_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyLastEvent {
    pub agent_id: String,
    pub event_name: Option<String>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PtySnapshot {
    pub events_by_name: BTreeMap<String, u64>,
    pub total_events: u64,
    pub last_event: Option<PtyLastEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SystemSnapshot {
    pub memory_usage_mb: f64,
    pub last_updated: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MetricsSnapshot {
    pub performance: PerformanceMetrics,
    pub router: RouterSnapshot,
    pub rate_limits: Vec<RateLimitSnapshot>,
    pub leases: LeaseSnapshotSummary,
    pub pty: PtySnapshot,
    pub system: SystemSnapshot,
}

#[derive(Debug, Default)]
struct PerformanceState {
    message_routing_latency_ms: f64,
    total_messages_routed: u64,
    agent_spawn_time_ms: f64,
    lease_acquisition_time_ms: f64,
    lease_acquisition_samples: u64,
    total_leases_acquired: u64,
    memory_usage_mb: f64,
    rate_limited_messages: u64,
}

impl PerformanceState {
    fn to_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            message_routing_latency_ms: self.message_routing_latency_ms,
            agent_spawn_time_ms: self.agent_spawn_time_ms,
            lease_acquisition_time_ms: self.lease_acquisition_time_ms,
            total_messages_routed: self.total_messages_routed,
            total_leases_acquired: self.total_leases_acquired,
            memory_usage_mb: self.memory_usage_mb,
            rate_limited_messages: self.rate_limited_messages,
        }
    }
}

#[derive(Debug, Default)]
struct RouterState {
    queue_depths: Vec<usize>,
    last_dispatched_priority: Option<String>,
    last_dispatched_at: Option<SystemTime>,
    rate_limited_messages: u64,
}

#[derive(Debug, Default)]
struct RateLimitState {
    tokens_remaining: f64,
    capacity: f64,
    refill_rate: f64,
    last_refill: Option<SystemTime>,
    rate_limit_hits: u64,
}

#[derive(Debug, Default)]
struct LeaseState {
    active_leases: usize,
    pending_by_resource: HashMap<String, usize>,
    deferrals: u64,
    overrides: u64,
    escalations: u64,
    outstanding_leases: HashSet<u64>,
}

#[derive(Debug, Default)]
struct PtyState {
    events_by_name: HashMap<String, u64>,
    total_events: u64,
    last_event: Option<PtyLastEvent>,
}

#[derive(Debug, Default)]
struct SystemState {
    memory_usage_mb: f64,
    last_updated: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct MetricsCollector {
    performance: Arc<RwLock<PerformanceState>>,
    router: Arc<RwLock<RouterState>>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimitState>>>,
    leases: Arc<RwLock<LeaseState>>,
    pty: Arc<RwLock<PtyState>>,
    system: Arc<RwLock<SystemState>>,
    timers: Arc<RwLock<HashMap<String, Instant>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            performance: Arc::new(RwLock::new(PerformanceState::default())),
            router: Arc::new(RwLock::new(RouterState::default())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            leases: Arc::new(RwLock::new(LeaseState::default())),
            pty: Arc::new(RwLock::new(PtyState::default())),
            system: Arc::new(RwLock::new(SystemState::default())),
            timers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn start_timer(&self, timer_name: &str) {
        let mut timers = self.timers.write().unwrap();
        timers.insert(timer_name.to_string(), Instant::now());
    }

    pub fn stop_timer(&self, timer_name: &str) -> Option<Duration> {
        let mut timers = self.timers.write().unwrap();
        timers.remove(timer_name).map(|start| start.elapsed())
    }

    pub fn record_message_routing(&self, duration_ms: f64) {
        let mut performance = self.performance.write().unwrap();
        performance.total_messages_routed += 1;
        let total = performance.total_messages_routed.max(1);
        performance.message_routing_latency_ms = ((performance.message_routing_latency_ms
            * (total.saturating_sub(1) as f64))
            + duration_ms)
            / total as f64;
    }

    pub fn update_queue_depths(&self, queue_depths: &[usize]) {
        let mut router = self.router.write().unwrap();
        router.queue_depths = queue_depths.to_vec();
    }

    pub fn record_router_delivery(
        &self,
        priority: Priority,
        wait_duration: Duration,
        queue_depths: &[usize],
    ) {
        self.record_message_routing(wait_duration.as_secs_f64() * 1000.0);
        let mut router = self.router.write().unwrap();
        router.last_dispatched_priority = Some(priority.as_str().to_string());
        router.last_dispatched_at = Some(SystemTime::now());
        router.queue_depths = queue_depths.to_vec();
    }

    pub fn increment_rate_limited(&self, sender: &str) {
        {
            let mut performance = self.performance.write().unwrap();
            performance.rate_limited_messages += 1;
        }
        {
            let mut router = self.router.write().unwrap();
            router.rate_limited_messages += 1;
        }
        let mut buckets = self.rate_limits.write().unwrap();
        let entry = buckets
            .entry(sender.to_string())
            .or_insert_with(RateLimitState::default);
        entry.rate_limit_hits = entry.rate_limit_hits.saturating_add(1);
    }

    pub fn update_token_bucket(
        &self,
        sender: &str,
        tokens_remaining: f64,
        capacity: f64,
        refill_rate: f64,
        last_refill: Option<SystemTime>,
    ) {
        let mut buckets = self.rate_limits.write().unwrap();
        let entry = buckets
            .entry(sender.to_string())
            .or_insert_with(RateLimitState::default);
        entry.tokens_remaining = tokens_remaining;
        entry.capacity = capacity;
        entry.refill_rate = refill_rate;
        entry.last_refill = last_refill;
    }

    pub fn record_agent_spawn(&self, duration_ms: f64) {
        let mut performance = self.performance.write().unwrap();
        performance.agent_spawn_time_ms = duration_ms;
    }

    pub fn record_agent_event(&self, agent_id: &str, event_name: Option<&str>) {
        let mut pty = self.pty.write().unwrap();
        let key = event_name.unwrap_or("unknown").to_string();
        let entry = pty.events_by_name.entry(key).or_insert(0);
        *entry += 1;
        pty.total_events += 1;
        pty.last_event = Some(PtyLastEvent {
            agent_id: agent_id.to_string(),
            event_name: event_name.map(|value| value.to_string()),
            timestamp: SystemTime::now(),
        });
    }

    pub fn record_lease_acquisition(&self, duration_ms: f64) {
        let mut performance = self.performance.write().unwrap();
        performance.lease_acquisition_samples += 1;
        let total = performance.lease_acquisition_samples.max(1);
        performance.lease_acquisition_time_ms = ((performance.lease_acquisition_time_ms
            * (total.saturating_sub(1) as f64))
            + duration_ms)
            / total as f64;
    }

    pub fn record_lease_grant(&self) {
        let mut performance = self.performance.write().unwrap();
        performance.total_leases_acquired += 1;
    }

    pub fn record_lease_release(&self) {}

    pub fn record_lease_deferral(&self) {
        let mut leases = self.leases.write().unwrap();
        leases.deferrals = leases.deferrals.saturating_add(1);
    }

    pub fn record_lease_override(&self) {
        let mut leases = self.leases.write().unwrap();
        leases.overrides = leases.overrides.saturating_add(1);
    }

    pub fn record_lease_escalation(&self) {
        let mut leases = self.leases.write().unwrap();
        leases.escalations = leases.escalations.saturating_add(1);
    }

    pub fn update_lease_inventory(
        &self,
        active_leases: usize,
        pending_by_resource: HashMap<String, usize>,
        outstanding_leases: Vec<u64>,
    ) {
        let mut leases = self.leases.write().unwrap();
        leases.active_leases = active_leases;
        leases.pending_by_resource = pending_by_resource;
        leases.outstanding_leases = outstanding_leases.into_iter().collect();
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.update_memory_usage();
        let performance = self.performance.read().unwrap();
        performance.to_metrics()
    }

    pub fn get_snapshot(&self) -> MetricsSnapshot {
        self.update_memory_usage();
        let performance_snapshot = {
            let performance = self.performance.read().unwrap();
            performance.to_metrics()
        };

        let router_snapshot = {
            let router = self.router.read().unwrap();
            let mut depths = BTreeMap::new();
            let stored = &router.queue_depths;
            let max_index = Priority::DirectorOverride.as_index();
            for index in 0..=max_index {
                let priority = Priority::from_index(index);
                let depth = stored.get(index).copied().unwrap_or_default();
                depths.insert(priority.as_str().to_string(), depth);
            }
            RouterSnapshot {
                queue_depths: depths,
                last_dispatched_priority: router.last_dispatched_priority.clone(),
                last_dispatched_at: router.last_dispatched_at,
                rate_limited_messages: router.rate_limited_messages,
            }
        };

        let rate_limit_snapshot = {
            let buckets = self.rate_limits.read().unwrap();
            let mut entries: Vec<RateLimitSnapshot> = buckets
                .iter()
                .map(|(sender, state)| RateLimitSnapshot {
                    sender: sender.clone(),
                    tokens_remaining: state.tokens_remaining,
                    capacity: state.capacity,
                    refill_rate: state.refill_rate,
                    last_refill: state.last_refill,
                    rate_limit_hits: state.rate_limit_hits,
                })
                .collect();
            entries.sort_by(|a, b| a.sender.cmp(&b.sender));
            entries
        };

        let leases_snapshot = {
            let leases = self.leases.read().unwrap();
            let mut pending = BTreeMap::new();
            for (resource, depth) in leases.pending_by_resource.iter() {
                pending.insert(resource.clone(), *depth);
            }
            let outstanding = leases.outstanding_leases.iter().copied().collect();
            let total_pending = pending.values().copied().sum();
            LeaseSnapshotSummary {
                active_leases: leases.active_leases,
                total_pending,
                pending_by_resource: pending,
                deferrals: leases.deferrals,
                overrides: leases.overrides,
                escalations: leases.escalations,
                outstanding_lease_ids: outstanding,
            }
        };

        let pty_snapshot = {
            let pty = self.pty.read().unwrap();
            let mut counts = BTreeMap::new();
            for (name, count) in pty.events_by_name.iter() {
                counts.insert(name.clone(), *count);
            }
            PtySnapshot {
                events_by_name: counts,
                total_events: pty.total_events,
                last_event: pty.last_event.clone(),
            }
        };

        let system_snapshot = {
            let system = self.system.read().unwrap();
            SystemSnapshot {
                memory_usage_mb: system.memory_usage_mb,
                last_updated: system.last_updated,
            }
        };

        MetricsSnapshot {
            performance: performance_snapshot,
            router: router_snapshot,
            rate_limits: rate_limit_snapshot,
            leases: leases_snapshot,
            pty: pty_snapshot,
            system: system_snapshot,
        }
    }

    pub fn reset_metrics(&self) {
        *self.performance.write().unwrap() = PerformanceState::default();
        *self.router.write().unwrap() = RouterState::default();
        self.rate_limits.write().unwrap().clear();
        *self.leases.write().unwrap() = LeaseState::default();
        *self.pty.write().unwrap() = PtyState::default();
        *self.system.write().unwrap() = SystemState::default();
        self.timers.write().unwrap().clear();
    }

    fn update_memory_usage(&self) {
        let mut usage_mb = 0.0;
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("ps")
                .args(["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()
            {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if let Ok(kb) = text.trim().parse::<f64>() {
                        usage_mb = kb / 1024.0;
                    }
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            usage_mb = 0.0;
        }
        {
            let mut performance = self.performance.write().unwrap();
            performance.memory_usage_mb = usage_mb;
        }
        {
            let mut system = self.system.write().unwrap();
            system.memory_usage_mb = usage_mb;
            system.last_updated = Some(SystemTime::now());
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
