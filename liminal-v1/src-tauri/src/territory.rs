use crate::config::{
    parse_duration as parse_duration_str, TerritoryConfig as TerritoryConfigOverrides,
};
use crate::executor::MaintenanceExecutor;
use crate::metrics::{HeatSummary, MetricsCollector, QuorumMetricsUpdate};

#[allow(unused_imports)]
use crate::consensus::{quorum_vote, ConsensusBroker};

#[allow(unused_imports)]
use crate::ledger::{
    LeaseEscalationRecord, LeaseEvent as LedgerLeaseEvent, LeaseQueueRecord, LeaseRecord,
    LedgerEvent, LedgerWriter, QuorumVote,
};
use crate::router::Priority;
use std::collections::HashMap;
#[cfg(feature = "spatial-hash")]
use std::collections::HashSet;
#[cfg(feature = "spatial-hash")]
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, watch, Mutex, RwLock};

pub type ResourcePath = String;
pub type AgentId = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LeaseId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestId(u64);

static LEASE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

impl LeaseId {
    fn new() -> Self {
        Self(LEASE_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl RequestId {
    fn new() -> Self {
        Self(REQUEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone)]
pub struct TerritoryManager {
    state: Arc<RwLock<TerritoryState>>,
    policy: TerritoryPolicy,
    metrics: MetricsCollector,
    events: broadcast::Sender<TerritoryEvent>,
    ledger: Option<LedgerWriter>,
    consensus: Option<ConsensusBroker>,
    heat_map: Arc<Mutex<HeatMap>>,
    shutdown: watch::Sender<bool>,
    maintenance_executor: Arc<Mutex<Option<MaintenanceExecutor>>>,
    maintenance_started: Arc<AtomicBool>,
}

#[derive(Clone, Debug)]
struct TerritoryState {
    leases: HashMap<ResourcePath, Lease>,
    queues: HashMap<ResourcePath, Vec<LeaseQueueEntry>>,
    #[cfg(feature = "spatial-hash")]
    spatial: SpatialHash,
}

impl TerritoryState {
    #[cfg(feature = "spatial-hash")]
    fn new(cell_size: f64) -> Self {
        Self {
            leases: HashMap::new(),
            queues: HashMap::new(),
            spatial: SpatialHash::new(cell_size),
        }
    }

    #[cfg(not(feature = "spatial-hash"))]
    fn new(_cell_size: f64) -> Self {
        Self {
            leases: HashMap::new(),
            queues: HashMap::new(),
        }
    }

    fn total_queue_depth(&self) -> usize {
        self.queues.values().map(|entries| entries.len()).sum()
    }

    fn queue_depth(&self, resource: &ResourcePath) -> usize {
        self.queues
            .get(resource)
            .map(|entries| entries.len())
            .unwrap_or(0)
    }

    fn queue_entries_mut(&mut self, resource: &ResourcePath) -> &mut Vec<LeaseQueueEntry> {
        self.queues.entry(resource.clone()).or_default()
    }

    fn enqueue(
        &mut self,
        policy: &TerritoryPolicy,
        request: LeaseRequest,
        requested_at: Instant,
        state: NegotiationState,
        deferred_until: Option<Instant>,
    ) -> (NegotiationHandle, usize) {
        let entries = self.queue_entries_mut(&request.resource_id);
        let request_id = RequestId::new();
        let handle = NegotiationHandle {
            request_id,
            resource_id: request.resource_id.clone(),
            agent_id: request.agent_id.clone(),
            queue_position: entries.len() + 1,
        };
        let entry = LeaseQueueEntry {
            id: request_id,
            handle: handle.clone(),
            request: LeaseQueueDescriptor::from_request(&request),
            enqueued_at: requested_at,
            deferred_until,
            state,
            escalation_ticket: None,
        };
        entries.push(entry);
        Self::reindex(entries, policy);
        let position = entries
            .iter()
            .find(|entry| entry.id == request_id)
            .map(|entry| entry.handle.queue_position)
            .unwrap_or(1);
        let mut handle = handle;
        handle.queue_position = position;
        (handle, self.total_queue_depth())
    }

    fn take_next(
        &mut self,
        policy: &TerritoryPolicy,
        resource: &ResourcePath,
        now: Instant,
    ) -> Option<LeaseQueueEntry> {
        let entries = self.queues.get_mut(resource)?;
        Self::reindex(entries, policy);
        if entries.is_empty() {
            return None;
        }
        let idx = entries.iter().position(|entry| {
            entry
                .deferred_until
                .map_or(true, |deadline| deadline <= now)
        });
        idx.map(|i| entries.remove(i))
    }

    fn reindex(entries: &mut [LeaseQueueEntry], policy: &TerritoryPolicy) {
        entries.sort_by(|a, b| {
            b.request
                .priority
                .cmp(&a.request.priority)
                .then(a.enqueued_at.cmp(&b.enqueued_at))
        });
        for (index, entry) in entries.iter_mut().enumerate() {
            entry.handle.queue_position = index + 1;
            if entry.deferred_until.is_some() && entry.state == NegotiationState::Queued {
                entry.state = NegotiationState::Deferred;
            }
            if entry.state == NegotiationState::Negotiating
                && index as u32 >= policy.negotiation_max_rounds
            {
                entry.state = NegotiationState::Escalating;
            }
        }
    }
}

#[derive(Debug)]
struct HeatCell {
    value: f64,
    updated_at: Instant,
}

#[derive(Debug)]
struct HeatMap {
    decay_per_second: f64,
    increment: f64,
    max_value: f64,
    cells: HashMap<ResourcePath, HeatCell>,
}

impl HeatMap {
    fn new(decay_per_second: f64, increment: f64, max_value: f64) -> Self {
        Self {
            decay_per_second: decay_per_second.max(0.0).min(1.0),
            increment: increment.max(0.0),
            max_value: max_value.max(0.0),
            cells: HashMap::new(),
        }
    }

    fn bump(&mut self, resource: &ResourcePath, now: Instant) -> HeatSummary {
        let cell = self.cells.entry(resource.clone()).or_insert(HeatCell {
            value: 0.0,
            updated_at: now,
        });
        HeatMap::decay_cell(self.decay_per_second, cell, now);
        cell.value = (cell.value + self.increment).min(self.max_value);
        cell.updated_at = now;
        self.summary(now)
    }

    fn summary(&mut self, now: Instant) -> HeatSummary {
        let mut remove_keys = Vec::new();
        for (resource, cell) in self.cells.iter_mut() {
            HeatMap::decay_cell(self.decay_per_second, cell, now);
            if cell.value < 0.01 {
                remove_keys.push(resource.clone());
            }
        }
        for key in remove_keys {
            self.cells.remove(&key);
        }
        let mut hottest_resource = None;
        let mut hottest_score = 0.0;
        for (resource, cell) in self.cells.iter() {
            if cell.value > hottest_score {
                hottest_score = cell.value;
                hottest_resource = Some(resource.clone());
            }
        }
        HeatSummary {
            hottest_resource,
            hottest_score,
            tracked: self.cells.len(),
        }
    }

    fn decay_cell(decay_per_second: f64, cell: &mut HeatCell, now: Instant) {
        if let Some(elapsed) = now.checked_duration_since(cell.updated_at) {
            let seconds = elapsed.as_secs_f64();
            if seconds > 0.0 && decay_per_second > 0.0 {
                let base = (1.0 - decay_per_second).clamp(0.0, 1.0);
                let factor = if base == 0.0 { 0.0 } else { base.powf(seconds) };
                cell.value *= factor;
            }
        }
    }
}

#[derive(Default)]
struct LeaseInventorySnapshot {
    active: usize,
    pending: HashMap<String, usize>,
    outstanding: Vec<u64>,
}

impl LeaseInventorySnapshot {
    fn from_state(state: &TerritoryState) -> Self {
        let mut pending = HashMap::new();
        for (resource, entries) in state.queues.iter() {
            pending.insert(resource.clone(), entries.len());
        }
        let outstanding = state
            .leases
            .values()
            .map(|lease| lease.id.as_u64())
            .collect();
        Self {
            active: state.leases.len(),
            pending,
            outstanding,
        }
    }

    fn into_parts(self) -> (usize, HashMap<String, usize>, Vec<u64>) {
        (self.active, self.pending, self.outstanding)
    }
}

#[derive(Clone, Debug)]
struct Lease {
    id: LeaseId,
    resource_id: ResourcePath,
    holder_id: AgentId,
    holder_role: Option<String>,
    priority: Priority,
    granted_at: Instant,
    expires_at: Instant,
    last_heartbeat_at: Instant,
    holder_progress: f32,
    negotiation_state: NegotiationState,
    conflict_attempts: u32,
    defer_count: u32,
    override_count: u32,
    escalation_ticket: Option<String>,
    coordinates: Option<(f64, f64)>,
    #[cfg(feature = "spatial-hash")]
    cell: Option<CellIndex>,
}

impl Lease {
    fn new(request: &LeaseRequest, now: Instant, policy: &TerritoryPolicy) -> Self {
        let default_duration = policy.default_lease_duration;
        let max_duration = policy.max_lease_duration;
        let effective_duration = if default_duration < max_duration {
            default_duration
        } else {
            max_duration
        };
        Self {
            id: LeaseId::new(),
            resource_id: request.resource_id.clone(),
            holder_id: request.agent_id.clone(),
            holder_role: request.holder_role.clone(),
            priority: request.priority,
            granted_at: now,
            expires_at: now + effective_duration,
            last_heartbeat_at: now,
            holder_progress: request.progress_hint.unwrap_or(0.0).clamp(0.0, 1.0),
            negotiation_state: NegotiationState::Idle,
            conflict_attempts: 0,
            defer_count: 0,
            override_count: 0,
            escalation_ticket: None,
            coordinates: request.coordinates,
            #[cfg(feature = "spatial-hash")]
            cell: None,
        }
    }

    fn snapshot(&self) -> LeaseSnapshot {
        LeaseSnapshot {
            lease_id: self.id,
            resource_id: self.resource_id.clone(),
            holder_id: self.holder_id.clone(),
            holder_role: self.holder_role.clone(),
            priority: self.priority,
            granted_at: self.granted_at,
            expires_at: self.expires_at,
            last_heartbeat_at: self.last_heartbeat_at,
            holder_progress: self.holder_progress,
            conflict_attempts: self.conflict_attempts,
            defer_count: self.defer_count,
            override_count: self.override_count,
            escalation_ticket: self.escalation_ticket.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LeaseSnapshot {
    pub lease_id: LeaseId,
    pub resource_id: ResourcePath,
    pub holder_id: AgentId,
    pub holder_role: Option<String>,
    pub priority: Priority,
    pub granted_at: Instant,
    pub expires_at: Instant,
    pub last_heartbeat_at: Instant,
    pub holder_progress: f32,
    pub conflict_attempts: u32,
    pub defer_count: u32,
    pub override_count: u32,
    pub escalation_ticket: Option<String>,
}

#[derive(Clone, Debug)]
pub struct LeaseRequest {
    pub agent_id: AgentId,
    pub resource_id: ResourcePath,
    pub priority: Priority,
    pub holder_role: Option<String>,
    pub progress_hint: Option<f32>,
    pub coordinates: Option<(f64, f64)>,
}

impl LeaseRequest {
    pub fn new(agent_id: AgentId, resource_id: ResourcePath, priority: Priority) -> Self {
        Self {
            agent_id,
            resource_id,
            priority,
            holder_role: None,
            progress_hint: None,
            coordinates: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransferRequest {
    pub from_agent: AgentId,
    pub to_agent: AgentId,
    pub resource_id: ResourcePath,
    pub new_priority: Option<Priority>,
}

#[derive(Clone, Debug)]
pub struct NegotiationHandle {
    pub request_id: RequestId,
    pub resource_id: ResourcePath,
    pub agent_id: AgentId,
    pub queue_position: usize,
}

#[derive(Clone, Debug)]
pub enum LeaseDecision {
    Granted(LeaseSnapshot),
    Deferred {
        handle: NegotiationHandle,
        grace_deadline: Instant,
    },
    Queued(NegotiationHandle),
    Overridden {
        previous: LeaseSnapshot,
        lease: LeaseSnapshot,
    },
}

#[derive(Clone, Debug)]
pub enum TransferDecision {
    Transferred {
        previous: LeaseSnapshot,
        lease: LeaseSnapshot,
    },
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NegotiationState {
    Idle,
    Queued,
    Negotiating,
    Deferred,
    Escalating,
    Overridden,
    Expired,
}

#[derive(Clone, Debug)]
pub enum EscalationReason {
    QueueDepth,
    Starvation,
    Deadlock,
}

#[derive(Clone, Debug)]
pub enum TerritoryEvent {
    Granted(LeaseSnapshot),
    Deferred {
        handle: NegotiationHandle,
        grace_deadline: Instant,
    },
    Queued(NegotiationHandle),
    Released(LeaseSnapshot),
    Overridden {
        previous: LeaseSnapshot,
        lease: LeaseSnapshot,
    },
    Escalated {
        handle: NegotiationHandle,
        reason: EscalationReason,
    },
}

#[derive(Clone, Debug)]
pub struct TerritoryPolicy {
    pub default_lease_duration: Duration,
    pub max_lease_duration: Duration,
    pub auto_extend_threshold: Duration,
    pub negotiation_timeout: Duration,
    pub negotiation_max_rounds: u32,
    pub escalation_queue_threshold: usize,
    pub escalation_deadlock_timeout: Duration,
    pub fairness_starvation_threshold: Duration,
    pub fairness_priority_boost_after: Duration,
    pub override_priority_delta: u8,
    pub spatial_cell_size: f64,
    pub consensus_threshold: f32,
    pub heat_decay_per_second: f64,
    pub heat_increment: f64,
    pub heat_max: f64,
}

impl TerritoryPolicy {
    fn baseline() -> Self {
        Self {
            default_lease_duration: Duration::from_secs(900),
            max_lease_duration: Duration::from_secs(3600),
            auto_extend_threshold: Duration::from_secs(60),
            negotiation_timeout: Duration::from_secs(30),
            negotiation_max_rounds: 3,
            escalation_queue_threshold: 2,
            escalation_deadlock_timeout: Duration::from_secs(60),
            fairness_starvation_threshold: Duration::from_secs(600),
            fairness_priority_boost_after: Duration::from_secs(300),
            override_priority_delta: 1,
            spatial_cell_size: 64.0,
            consensus_threshold: 0.66,
            heat_decay_per_second: 0.15,
            heat_increment: 1.5,
            heat_max: 10.0,
        }
    }

    pub fn from_config(config: Option<&TerritoryConfigOverrides>) -> Self {
        let mut policy = Self::baseline();
        if let Some(overrides) = config {
            if let Some(duration) = overrides
                .default_lease_duration
                .as_deref()
                .and_then(parse_duration_str)
            {
                policy.default_lease_duration = duration;
            }
            if let Some(duration) = overrides
                .max_lease_duration
                .as_deref()
                .and_then(parse_duration_str)
            {
                policy.max_lease_duration = duration;
            }
            if let Some(duration) = overrides
                .auto_extend_threshold
                .as_deref()
                .and_then(parse_duration_str)
            {
                policy.auto_extend_threshold = duration;
            }
            if let Some(duration) = overrides
                .negotiation_timeout
                .as_deref()
                .and_then(parse_duration_str)
            {
                policy.negotiation_timeout = duration;
            }
            if let Some(rounds) = overrides.negotiation_max_rounds {
                policy.negotiation_max_rounds = rounds;
            }
            if let Some(threshold) = overrides.escalation_queue_threshold {
                policy.escalation_queue_threshold = threshold;
            }
            if let Some(duration) = overrides
                .escalation_deadlock_timeout
                .as_deref()
                .and_then(parse_duration_str)
            {
                policy.escalation_deadlock_timeout = duration;
            }
            if let Some(duration) = overrides
                .fairness_starvation_threshold
                .as_deref()
                .and_then(parse_duration_str)
            {
                policy.fairness_starvation_threshold = duration;
            }
            if let Some(duration) = overrides
                .fairness_priority_boost_after
                .as_deref()
                .and_then(parse_duration_str)
            {
                policy.fairness_priority_boost_after = duration;
            }
            if let Some(threshold) = overrides.consensus_threshold {
                policy.consensus_threshold = threshold;
            }
            if let Some(decay) = overrides.heat_decay_per_second {
                policy.heat_decay_per_second = decay.max(0.0);
            }
            if let Some(increment) = overrides.heat_increment {
                policy.heat_increment = increment.max(0.0);
            }
            if let Some(max_value) = overrides.heat_max {
                policy.heat_max = max_value.max(0.0);
            }
        }
        policy
    }
}

impl Default for TerritoryPolicy {
    fn default() -> Self {
        Self::baseline()
    }
}

#[derive(Clone, Debug)]
struct LeaseQueueDescriptor {
    agent_id: AgentId,
    priority: Priority,
    holder_role: Option<String>,
    coordinates: Option<(f64, f64)>,
}

impl LeaseQueueDescriptor {
    fn from_request(request: &LeaseRequest) -> Self {
        Self {
            agent_id: request.agent_id.clone(),
            priority: request.priority,
            holder_role: request.holder_role.clone(),
            coordinates: request.coordinates,
        }
    }
}

#[derive(Clone, Debug)]
struct LeaseQueueEntry {
    id: RequestId,
    handle: NegotiationHandle,
    request: LeaseQueueDescriptor,
    enqueued_at: Instant,
    deferred_until: Option<Instant>,
    state: NegotiationState,
    escalation_ticket: Option<String>,
}

#[cfg(feature = "spatial-hash")]
#[derive(Clone, Debug)]
struct SpatialHash {
    cell_size: f64,
    buckets: HashMap<CellIndex, HashSet<LeaseId>>,
    non_spatial: HashSet<LeaseId>,
}

#[cfg(feature = "spatial-hash")]
impl SpatialHash {
    fn new(cell_size: f64) -> Self {
        Self {
            cell_size,
            buckets: HashMap::new(),
            non_spatial: HashSet::new(),
        }
    }

    fn insert(&mut self, lease_id: LeaseId, coordinates: Option<(f64, f64)>) -> Option<CellIndex> {
        if let Some(coords) = coordinates {
            let cell = CellIndex::from_coords(coords, self.cell_size);
            self.buckets.entry(cell).or_default().insert(lease_id);
            Some(cell)
        } else {
            self.non_spatial.insert(lease_id);
            None
        }
    }

    fn remove(&mut self, lease_id: LeaseId, cell: Option<CellIndex>) {
        if let Some(cell) = cell {
            if let Some(bucket) = self.buckets.get_mut(&cell) {
                bucket.remove(&lease_id);
                if bucket.is_empty() {
                    self.buckets.remove(&cell);
                }
            }
        } else {
            self.non_spatial.remove(&lease_id);
        }
    }
}

#[cfg(feature = "spatial-hash")]
#[derive(Clone, Copy, Debug, Eq)]
struct CellIndex(i64, i64);

#[cfg(feature = "spatial-hash")]
impl CellIndex {
    fn from_coords(coords: (f64, f64), cell_size: f64) -> Self {
        let x = (coords.0 / cell_size).floor() as i64;
        let y = (coords.1 / cell_size).floor() as i64;
        Self(x, y)
    }
}

#[cfg(feature = "spatial-hash")]
impl PartialEq for CellIndex {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

#[cfg(feature = "spatial-hash")]
impl Hash for CellIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i64(self.0);
        state.write_i64(self.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> TerritoryConfigOverrides {
        TerritoryConfigOverrides {
            default_lease_duration: Some("120s".to_string()),
            max_lease_duration: Some("2h".to_string()),
            auto_extend_threshold: Some("45s".to_string()),
            negotiation_timeout: Some("15s".to_string()),
            negotiation_max_rounds: Some(5),
            escalation_queue_threshold: Some(4),
            escalation_deadlock_timeout: Some("180s".to_string()),
            fairness_starvation_threshold: Some("420s".to_string()),
            fairness_priority_boost_after: Some("120s".to_string()),
            consensus_threshold: Some(0.75),
            heat_decay_per_second: Some(0.25),
            heat_increment: Some(2.0),
            heat_max: Some(9.0),
        }
    }

    #[test]
    fn territory_policy_applies_config_overrides() {
        let config = sample_config();
        let policy = TerritoryPolicy::from_config(Some(&config));
        assert_eq!(policy.default_lease_duration, Duration::from_secs(120));
        assert_eq!(policy.max_lease_duration, Duration::from_secs(2 * 3600));
        assert_eq!(policy.auto_extend_threshold, Duration::from_secs(45));
        assert_eq!(policy.negotiation_timeout, Duration::from_secs(15));
        assert_eq!(policy.negotiation_max_rounds, 5);
        assert_eq!(policy.escalation_queue_threshold, 4);
        assert_eq!(policy.escalation_deadlock_timeout, Duration::from_secs(180));
        assert_eq!(
            policy.fairness_starvation_threshold,
            Duration::from_secs(420)
        );
        assert_eq!(
            policy.fairness_priority_boost_after,
            Duration::from_secs(120)
        );
        assert!((policy.consensus_threshold - 0.75).abs() < f32::EPSILON);
        assert!((policy.heat_decay_per_second - 0.25).abs() < f64::EPSILON);
        assert!((policy.heat_increment - 2.0).abs() < f64::EPSILON);
        assert!((policy.heat_max - 9.0).abs() < f64::EPSILON);
    }
}

impl TerritoryManager {
    pub fn new(metrics: MetricsCollector, config: Option<&TerritoryConfigOverrides>) -> Self {
        let policy = TerritoryPolicy::from_config(config);
        Self::with_policy_and_ledger(metrics, policy, None)
    }

    pub fn new_with_ledger(
        metrics: MetricsCollector,
        config: Option<&TerritoryConfigOverrides>,
        ledger: Option<LedgerWriter>,
    ) -> Self {
        let policy = TerritoryPolicy::from_config(config);
        Self::with_policy_and_ledger(metrics, policy, ledger)
    }

    pub fn with_policy(metrics: MetricsCollector, policy: TerritoryPolicy) -> Self {
        Self::with_policy_and_ledger(metrics, policy, None)
    }

    pub fn with_policy_and_ledger(
        metrics: MetricsCollector,
        policy: TerritoryPolicy,
        ledger: Option<LedgerWriter>,
    ) -> Self {
        let (events, _) = broadcast::channel(256);
        let state = TerritoryState::new(policy.spatial_cell_size);
        let consensus = ledger.as_ref().map(|writer| {
            ConsensusBroker::new(
                Some(writer.clone()),
                metrics.clone(),
                policy.consensus_threshold,
            )
        });
        let (shutdown, _) = watch::channel(false);
        let heat_map = Arc::new(Mutex::new(HeatMap::new(
            policy.heat_decay_per_second,
            policy.heat_increment,
            policy.heat_max,
        )));
        Self {
            state: Arc::new(RwLock::new(state)),
            policy,
            metrics,
            events,
            ledger,
            consensus,
            heat_map,
            shutdown,
            maintenance_executor: Arc::new(Mutex::new(None)),
            maintenance_started: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TerritoryEvent> {
        self.events.subscribe()
    }

    pub fn policy(&self) -> &TerritoryPolicy {
        &self.policy
    }

    pub async fn set_maintenance_executor(&self, executor: MaintenanceExecutor) {
        let mut guard = self.maintenance_executor.lock().await;
        *guard = Some(executor);
        drop(guard);
        self.start_maintenance_if_needed().await;
    }

    pub async fn maintenance_executor(&self) -> Option<MaintenanceExecutor> {
        self.maintenance_executor.lock().await.clone()
    }

    pub async fn acquire_lease(&self, request: LeaseRequest) -> LeaseDecision {
        self.start_maintenance_if_needed().await;
        let now = Instant::now();
        let requester_id = request.agent_id.clone();
        let requester_priority = request.priority;
        let mut guard = self.state.write().await;
        if let Some(active) = guard.leases.get_mut(&request.resource_id) {
            let priority_delta =
                request.priority.as_index() as i32 - active.priority.as_index() as i32;
            let mut quorum_votes = vec![
                quorum_vote(
                    &active.holder_id,
                    (active.priority.as_index() + 1) as f32,
                    false,
                ),
                quorum_vote(
                    &requester_id,
                    (requester_priority.as_index() + 1) as f32,
                    true,
                ),
            ];
            let mut quorum_reason = String::from("maintain");
            if priority_delta >= self.policy.override_priority_delta as i32 {
                let resource_key = request.resource_id.clone();
                quorum_reason = String::from("override");
                #[cfg(feature = "spatial-hash")]
                let (lease_id, pending_coords, previous_snapshot, snapshot) = {
                    let active_ref = active;
                    let lease_id = active_ref.id;
                    let mut pending_coords = None;
                    if active_ref.coordinates != request.coordinates {
                        pending_coords = Some((active_ref.cell, request.coordinates));
                        active_ref.coordinates = request.coordinates;
                        active_ref.cell = None;
                    }
                    let previous_snapshot = active_ref.snapshot();
                    active_ref.holder_id = request.agent_id.clone();
                    active_ref.holder_role = request.holder_role.clone();
                    active_ref.priority = request.priority;
                    active_ref.granted_at = now;
                    active_ref.expires_at = now + self.policy.default_lease_duration;
                    active_ref.last_heartbeat_at = now;
                    active_ref.holder_progress =
                        request.progress_hint.unwrap_or(0.0).clamp(0.0, 1.0);
                    active_ref.override_count += 1;
                    let snapshot = active_ref.snapshot();
                    (lease_id, pending_coords, previous_snapshot, snapshot)
                };
                #[cfg(not(feature = "spatial-hash"))]
                let (previous_snapshot, snapshot) = {
                    let active_ref = active;
                    active_ref.coordinates = request.coordinates;
                    let previous_snapshot = active_ref.snapshot();
                    active_ref.holder_id = request.agent_id.clone();
                    active_ref.holder_role = request.holder_role.clone();
                    active_ref.priority = request.priority;
                    active_ref.granted_at = now;
                    active_ref.expires_at = now + self.policy.default_lease_duration;
                    active_ref.last_heartbeat_at = now;
                    active_ref.holder_progress =
                        request.progress_hint.unwrap_or(0.0).clamp(0.0, 1.0);
                    active_ref.override_count += 1;
                    let snapshot = active_ref.snapshot();
                    (previous_snapshot, snapshot)
                };
                #[cfg(feature = "spatial-hash")]
                if let Some((old_cell, coords)) = pending_coords {
                    guard.spatial.remove(lease_id, old_cell);
                    let new_cell = guard.spatial.insert(lease_id, coords);
                    if let Some(updated) = guard.leases.get_mut(&resource_key) {
                        updated.cell = new_cell;
                    }
                }
                let inventory = LeaseInventorySnapshot::from_state(&guard);
                let (active, pending, outstanding) = inventory.into_parts();
                drop(guard);
                self.bump_heat_map(&resource_key).await;
                self.record_quorum_decision(&resource_key, quorum_votes, &quorum_reason)
                    .await;
                self.metrics.record_lease_override();
                self.metrics
                    .update_lease_inventory(active, pending, outstanding);
                self.emit_event(TerritoryEvent::Overridden {
                    previous: previous_snapshot.clone(),
                    lease: snapshot.clone(),
                })
                .await;
                return LeaseDecision::Overridden {
                    previous: previous_snapshot,
                    lease: snapshot,
                };
            }
            let time_left = active
                .expires_at
                .checked_duration_since(now)
                .unwrap_or_default();
            let (handle, _total_depth, decision_state) =
                if time_left <= self.policy.auto_extend_threshold {
                    active.defer_count += 1;
                    let (handle, total) = guard.enqueue(
                        &self.policy,
                        request,
                        now,
                        NegotiationState::Deferred,
                        Some(now + self.policy.auto_extend_threshold),
                    );
                    let handle_for_decision = handle.clone();
                    (
                        handle,
                        total,
                        LeaseDecision::Deferred {
                            handle: handle_for_decision,
                            grace_deadline: now + self.policy.auto_extend_threshold,
                        },
                    )
                } else {
                    active.conflict_attempts += 1;
                    let (handle, total) =
                        guard.enqueue(&self.policy, request, now, NegotiationState::Queued, None);
                    let handle_for_decision = handle.clone();
                    (handle, total, LeaseDecision::Queued(handle_for_decision))
                };
            match &decision_state {
                LeaseDecision::Deferred { .. } => {
                    quorum_reason = String::from("defer");
                }
                LeaseDecision::Queued(_) => {
                    quorum_reason = String::from("queue");
                }
                _ => {}
            }
            let entries = guard.queue_entries_mut(&handle.resource_id);
            for entry in entries.iter() {
                if entry.handle.agent_id != requester_id {
                    quorum_votes.push(quorum_vote(
                        &entry.handle.agent_id,
                        (entry.request.priority.as_index() + 1) as f32,
                        false,
                    ));
                }
            }
            let should_escalate = entries.len() >= self.policy.escalation_queue_threshold
                || entries.iter().any(|entry| {
                    now.duration_since(entry.enqueued_at)
                        >= self.policy.fairness_starvation_threshold
                });
            if should_escalate {
                self.metrics.record_lease_escalation();
                quorum_reason = String::from("escalate");
                self.emit_event(TerritoryEvent::Escalated {
                    handle: handle.clone(),
                    reason: if entries.len() >= self.policy.escalation_queue_threshold {
                        EscalationReason::QueueDepth
                    } else {
                        EscalationReason::Starvation
                    },
                })
                .await;
            }
            let inventory = LeaseInventorySnapshot::from_state(&guard);
            let (active, pending, outstanding) = inventory.into_parts();
            let heat_resource = handle.resource_id.clone();
            drop(guard);
            if matches!(
                decision_state,
                LeaseDecision::Deferred { .. } | LeaseDecision::Queued(_)
            ) {
                self.metrics.record_lease_deferral();
            }
            self.bump_heat_map(&heat_resource).await;
            self.record_quorum_decision(&heat_resource, quorum_votes, &quorum_reason)
                .await;
            self.metrics
                .update_lease_inventory(active, pending, outstanding);
            match decision_state.clone() {
                LeaseDecision::Deferred {
                    handle,
                    grace_deadline,
                } => {
                    self.emit_event(TerritoryEvent::Deferred {
                        handle,
                        grace_deadline,
                    })
                    .await;
                }
                LeaseDecision::Queued(handle) => {
                    self.emit_event(TerritoryEvent::Queued(handle)).await;
                }
                _ => {}
            }
            return decision_state;
        }
        #[cfg(feature = "spatial-hash")]
        let mut lease = Lease::new(&request, now, &self.policy);
        #[cfg(not(feature = "spatial-hash"))]
        let lease = Lease::new(&request, now, &self.policy);
        #[cfg(feature = "spatial-hash")]
        {
            lease.cell = guard.spatial.insert(lease.id, lease.coordinates);
        }
        let snapshot = lease.snapshot();
        guard.leases.insert(request.resource_id.clone(), lease);
        let inventory = LeaseInventorySnapshot::from_state(&guard);
        let (active, pending, outstanding) = inventory.into_parts();
        drop(guard);
        self.metrics.record_lease_grant();
        self.metrics
            .update_lease_inventory(active, pending, outstanding);
        self.publish_heat_summary().await;
        self.emit_event(TerritoryEvent::Granted(snapshot.clone()))
            .await;
        LeaseDecision::Granted(snapshot)
    }

    pub async fn release_lease(
        &self,
        agent_id: &AgentId,
        resource: &ResourcePath,
    ) -> Option<LeaseSnapshot> {
        self.start_maintenance_if_needed().await;
        let now = Instant::now();
        let mut guard = self.state.write().await;
        let lease = guard.leases.get(resource)?;
        if lease.holder_id != *agent_id {
            return None;
        }
        let lease = guard.leases.remove(resource)?;
        #[cfg(feature = "spatial-hash")]
        guard.spatial.remove(lease.id, lease.cell);
        let snapshot = lease.snapshot();
        let next_entry = guard.take_next(&self.policy, resource, now);
        let mut granted_snapshot: Option<LeaseSnapshot> = None;
        if let Some(entry) = next_entry {
            let request = LeaseRequest {
                agent_id: entry.request.agent_id.clone(),
                resource_id: resource.clone(),
                priority: entry.request.priority,
                holder_role: entry.request.holder_role.clone(),
                progress_hint: None,
                coordinates: entry.request.coordinates,
            };
            let mut lease = Lease::new(&request, now, &self.policy);
            #[cfg(feature = "spatial-hash")]
            {
                lease.cell = guard.spatial.insert(lease.id, lease.coordinates);
            }
            granted_snapshot = Some(lease.snapshot());
            guard.leases.insert(resource.clone(), lease);
        }
        let inventory = LeaseInventorySnapshot::from_state(&guard);
        let (active, pending, outstanding) = inventory.into_parts();
        drop(guard);
        self.metrics
            .update_lease_inventory(active, pending, outstanding);
        self.publish_heat_summary().await;
        self.emit_event(TerritoryEvent::Released(snapshot.clone()))
            .await;
        if let Some(granted) = granted_snapshot.clone() {
            self.metrics.record_lease_grant();
            self.emit_event(TerritoryEvent::Granted(granted)).await;
        }
        Some(snapshot)
    }

    pub async fn transfer_lease(&self, request: TransferRequest) -> TransferDecision {
        let now = Instant::now();
        let mut guard = self.state.write().await;
        let lease = guard.leases.get_mut(&request.resource_id);
        if lease.is_none() {
            return TransferDecision::Rejected;
        }
        let lease = lease.unwrap();
        if lease.holder_id != request.from_agent {
            return TransferDecision::Rejected;
        }
        let previous_snapshot = lease.snapshot();
        lease.holder_id = request.to_agent.clone();
        if let Some(priority) = request.new_priority {
            lease.priority = priority;
        }
        lease.granted_at = now;
        lease.expires_at = now + self.policy.default_lease_duration;
        lease.last_heartbeat_at = now;
        lease.override_count += 1;
        let snapshot = lease.snapshot();
        let inventory = LeaseInventorySnapshot::from_state(&guard);
        let (active, pending, outstanding) = inventory.into_parts();
        drop(guard);
        self.metrics.record_lease_override();
        self.metrics
            .update_lease_inventory(active, pending, outstanding);
        self.emit_event(TerritoryEvent::Overridden {
            previous: previous_snapshot.clone(),
            lease: snapshot.clone(),
        })
        .await;
        TransferDecision::Transferred {
            previous: previous_snapshot,
            lease: snapshot,
        }
    }

    pub async fn current_lease(&self, resource: &ResourcePath) -> Option<LeaseSnapshot> {
        let guard = self.state.read().await;
        guard.leases.get(resource).map(|lease| lease.snapshot())
    }

    pub async fn update_progress(
        &self,
        resource: &ResourcePath,
        agent_id: &AgentId,
        progress: f32,
    ) -> Option<LeaseSnapshot> {
        let mut guard = self.state.write().await;
        let lease = guard.leases.get_mut(resource)?;
        if lease.holder_id != *agent_id {
            return None;
        }
        lease.holder_progress = progress.clamp(0.0, 1.0);
        lease.last_heartbeat_at = Instant::now();
        Some(lease.snapshot())
    }

    pub async fn queue_depth(&self, resource: &ResourcePath) -> usize {
        let guard = self.state.read().await;
        guard.queue_depth(resource)
    }

    async fn record_quorum_decision(
        &self,
        resource: &ResourcePath,
        votes: Vec<QuorumVote>,
        reason: &str,
    ) {
        if votes.is_empty() {
            return;
        }
        if let Some(broker) = &self.consensus {
            broker.record_quorum(resource, votes, reason).await;
        } else {
            let total: f32 = votes.iter().map(|vote| vote.weight.max(0.0)).sum();
            let agree: f32 = votes
                .iter()
                .filter(|vote| vote.vote)
                .map(|vote| vote.weight.max(0.0))
                .sum();
            let threshold = self.policy.consensus_threshold;
            let achieved = if total > f32::EPSILON {
                (agree / total) >= threshold
            } else {
                false
            };
            self.metrics.record_quorum_metrics(QuorumMetricsUpdate {
                resource_id: resource.clone(),
                achieved,
                threshold,
                reason: reason.to_string(),
            });
        }
    }

    async fn bump_heat_map(&self, resource: &ResourcePath) {
        let summary = {
            let mut heat = self.heat_map.lock().await;
            heat.bump(resource, Instant::now())
        };
        self.metrics.update_heat_summary(summary);
    }

    async fn publish_heat_summary(&self) {
        let summary = {
            let mut heat = self.heat_map.lock().await;
            heat.summary(Instant::now())
        };
        self.metrics.update_heat_summary(summary);
    }

    pub async fn heat_snapshot(&self) -> HeatSummary {
        self.start_maintenance_if_needed().await;
        let summary = {
            let mut heat = self.heat_map.lock().await;
            heat.summary(Instant::now())
        };
        self.metrics.update_heat_summary(summary.clone());
        summary
    }

    async fn emit_event(&self, event: TerritoryEvent) {
        let ledger_payload = self.ledger.as_ref().and_then(|writer| {
            ledger_event_from_territory(&event).map(|payload| (writer.clone(), payload))
        });
        let _ = self.events.send(event);
        if let Some((ledger_writer, payload)) = ledger_payload {
            let start = Instant::now();
            if ledger_writer
                .append_async(LedgerEvent::Lease(payload))
                .await
                .is_ok()
            {
                self.metrics.record_ledger_append(start.elapsed());
            } else {
                self.metrics.record_ledger_error();
            }
        }
    }

    async fn start_maintenance_if_needed(&self) {
        if self.maintenance_started.load(Ordering::SeqCst) {
            return;
        }
        let executor = {
            let guard = self.maintenance_executor.lock().await;
            guard.clone()
        };
        if let Some(executor) = executor {
            if self
                .maintenance_started
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                self.launch_maintenance_tasks(executor).await;
            }
        }
    }

    async fn launch_maintenance_tasks(&self, executor: MaintenanceExecutor) {
        let manager = self.clone();
        let mut shutdown_rx = self.shutdown.subscribe();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(120));
            loop {
                tokio::select! {
                    result = shutdown_rx.changed() => {
                        match result {
                            Ok(_) => {
                                if *shutdown_rx.borrow() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    _ = ticker.tick() => {
                        let executor = executor.clone();
                        let manager = manager.clone();
                        executor.spawn(async move {
                            manager.publish_heat_summary().await;
                        });
                    }
                }
            }
        });
    }
}

impl Drop for TerritoryManager {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
    }
}

fn ledger_event_from_territory(event: &TerritoryEvent) -> Option<LedgerLeaseEvent> {
    match event {
        TerritoryEvent::Granted(snapshot) => {
            Some(LedgerLeaseEvent::Granted(lease_record_from(snapshot)))
        }
        TerritoryEvent::Deferred {
            handle,
            grace_deadline,
        } => Some(LedgerLeaseEvent::Deferred(queue_record_from(
            handle,
            Some(*grace_deadline),
        ))),
        TerritoryEvent::Queued(handle) => {
            Some(LedgerLeaseEvent::Deferred(queue_record_from(handle, None)))
        }
        TerritoryEvent::Released(snapshot) => {
            Some(LedgerLeaseEvent::Released(lease_record_from(snapshot)))
        }
        TerritoryEvent::Overridden { previous, lease } => Some(LedgerLeaseEvent::Overridden {
            previous: lease_record_from(previous),
            lease: lease_record_from(lease),
        }),
        TerritoryEvent::Escalated { handle, reason } => Some(LedgerLeaseEvent::Escalated(
            escalation_record_from(handle, reason),
        )),
    }
}

fn lease_record_from(snapshot: &LeaseSnapshot) -> LeaseRecord {
    LeaseRecord {
        lease_id: snapshot.lease_id.as_u64(),
        resource_id: snapshot.resource_id.clone(),
        holder_id: snapshot.holder_id.clone(),
        priority: snapshot.priority.as_str().to_string(),
    }
}

fn queue_record_from(
    handle: &NegotiationHandle,
    grace_deadline: Option<Instant>,
) -> LeaseQueueRecord {
    LeaseQueueRecord {
        request_id: format!("{}:{}", handle.agent_id, handle.queue_position),
        agent_id: handle.agent_id.clone(),
        resource_id: handle.resource_id.clone(),
        queue_position: handle.queue_position,
        grace_deadline_ms: grace_deadline.map(instant_to_epoch_ms),
    }
}

fn escalation_record_from(
    handle: &NegotiationHandle,
    reason: &EscalationReason,
) -> LeaseEscalationRecord {
    let reason_str = match reason {
        EscalationReason::QueueDepth => "queueDepth",
        EscalationReason::Starvation => "starvation",
        EscalationReason::Deadlock => "deadlock",
    };
    LeaseEscalationRecord {
        agent_id: handle.agent_id.clone(),
        resource_id: handle.resource_id.clone(),
        reason: reason_str.to_string(),
    }
}

fn instant_to_epoch_ms(target: Instant) -> u64 {
    let now = Instant::now();
    let system_now = SystemTime::now();
    let target_system = if target <= now {
        system_now
    } else {
        let delta = target.duration_since(now);
        system_now.checked_add(delta).unwrap_or(system_now)
    };
    target_system
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
