use crate::config::LedgerConfig;
use crate::metrics::{
    ConsensusSnapshot, HeatSnapshot, LeaseSnapshotSummary, MetricsSnapshot, RouterSnapshot,
};
use crate::router::Priority;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::broadcast;

const DEFAULT_BROADCAST_CAPACITY: usize = 512;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("ledger I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ledger serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("ledger task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

pub type LedgerResult<T> = Result<T, LedgerError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogicalClock {
    pub wall_millis: u64,
    pub counter: u32,
}

impl LogicalClock {
    fn now(clock: &mut HybridLogicalClock) -> Self {
        clock.tick(SystemTime::now())
    }
}

#[derive(Debug, Default)]
struct HybridLogicalClock {
    last_wall: u64,
    counter: u32,
}

impl HybridLogicalClock {
    fn tick(&mut self, now: SystemTime) -> LogicalClock {
        let wall_millis = now
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        if wall_millis > self.last_wall {
            self.last_wall = wall_millis;
            self.counter = 0;
        } else {
            self.counter = self.counter.saturating_add(1);
        }
        LogicalClock {
            wall_millis: self.last_wall,
            counter: self.counter,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EventMetadata {
    pub trace_id: Option<String>,
    pub agent_id: Option<String>,
    pub territory_id: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventEnvelope {
    pub epoch_id: String,
    pub sequence: u64,
    pub logical_clock: LogicalClock,
    pub metadata: EventMetadata,
    pub payload_digest: String,
    pub hash_chain: String,
    pub event: LedgerEvent,
}

impl EventEnvelope {
    fn without_hash(&self) -> serde_json::Value {
        let mut value = serde_json::to_value(self).unwrap_or_default();
        if let serde_json::Value::Object(map) = &mut value {
            map.remove("hashChain");
        }
        value
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LedgerEvent {
    Router(RouterEvent),
    Lease(LeaseEvent),
    Consensus(ConsensusEvent),
    Pty(PtyEvent),
    Health(HealthEvent),
    Checkpoint(StateCheckpoint),
}

impl LedgerEvent {
    fn metadata(&self) -> EventMetadata {
        match self {
            LedgerEvent::Router(event) => event.metadata(),
            LedgerEvent::Lease(event) => event.metadata(),
            LedgerEvent::Consensus(event) => event.metadata(),
            LedgerEvent::Pty(event) => event.metadata(),
            LedgerEvent::Health(event) => event.metadata(),
            LedgerEvent::Checkpoint(event) => event.metadata(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RouterEvent {
    Dispatched(RouterDispatchRecord),
    RateLimited(RateLimitedRecord),
}

impl RouterEvent {
    fn metadata(&self) -> EventMetadata {
        match self {
            RouterEvent::Dispatched(record) => EventMetadata {
                agent_id: Some(record.sender.clone()),
                priority: Some(record.effective_priority.clone()),
                trace_id: record.message_id.clone(),
                territory_id: None,
            },
            RouterEvent::RateLimited(record) => EventMetadata {
                agent_id: Some(record.sender.clone()),
                priority: Some(record.priority.clone()),
                trace_id: None,
                territory_id: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouterDispatchRecord {
    pub message_id: Option<String>,
    pub content_digest: Option<String>,
    pub sender: String,
    pub recipient: String,
    pub priority: String,
    pub effective_priority: String,
    pub wait_time_ms: u64,
    pub queue_depths: Vec<usize>,
    pub aging_boosts: u8,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitedRecord {
    pub sender: String,
    pub priority: String,
    pub tokens_remaining: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumVote {
    pub agent_id: String,
    pub weight: f32,
    pub vote: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuorumVector {
    pub resource_id: String,
    pub threshold: f32,
    pub total_weight: f32,
    pub agree_weight: f32,
    pub achieved: bool,
    pub reason: String,
    pub votes: Vec<QuorumVote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LeaseEvent {
    Granted(LeaseRecord),
    Released(LeaseRecord),
    Deferred(LeaseQueueRecord),
    Escalated(LeaseEscalationRecord),
    Overridden {
        previous: LeaseRecord,
        lease: LeaseRecord,
    },
}

impl LeaseEvent {
    fn metadata(&self) -> EventMetadata {
        match self {
            LeaseEvent::Granted(record)
            | LeaseEvent::Released(record)
            | LeaseEvent::Overridden { lease: record, .. } => EventMetadata {
                agent_id: Some(record.holder_id.clone()),
                territory_id: Some(record.resource_id.clone()),
                priority: Some(record.priority.clone()),
                trace_id: Some(format!("lease-{}", record.lease_id)),
            },
            LeaseEvent::Deferred(record) => EventMetadata {
                agent_id: Some(record.agent_id.clone()),
                territory_id: Some(record.resource_id.clone()),
                priority: None,
                trace_id: Some(format!("lease-queue-{}", record.request_id)),
            },
            LeaseEvent::Escalated(record) => EventMetadata {
                agent_id: Some(record.agent_id.clone()),
                territory_id: Some(record.resource_id.clone()),
                priority: None,
                trace_id: Some(format!("lease-escalation-{}", record.reason)),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ConsensusEvent {
    #[default]
    Idle,
    Proposal(ConsensusSignal),
    Vote(ConsensusSignal),
    Commit(ConsensusSignal),
}

impl ConsensusEvent {
    fn metadata(&self) -> EventMetadata {
        match self {
            ConsensusEvent::Proposal(signal)
            | ConsensusEvent::Vote(signal)
            | ConsensusEvent::Commit(signal) => {
                let territory = signal
                    .vector
                    .as_ref()
                    .map(|vec| vec.resource_id.clone())
                    .or_else(|| signal.territory_id.clone());
                EventMetadata {
                    trace_id: Some(signal.topic.clone()),
                    agent_id: signal.agent_id.clone(),
                    territory_id: territory,
                    priority: None,
                }
            }
            ConsensusEvent::Idle => EventMetadata::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsensusSignal {
    pub topic: String,
    pub phase: String,
    pub agent_id: Option<String>,
    pub territory_id: Option<String>,
    pub quorum_threshold: Option<f32>,
    pub payload_digest: Option<String>,
    pub vector: Option<QuorumVector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyEvent {
    pub agent_id: String,
    pub event_name: Option<String>,
    pub timestamp_ms: u64,
}

impl PtyEvent {
    fn metadata(&self) -> EventMetadata {
        EventMetadata {
            agent_id: Some(self.agent_id.clone()),
            trace_id: Some(format!("pty-{}", self.timestamp_ms)),
            territory_id: None,
            priority: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthEvent {
    pub severity: String,
    pub message: String,
    pub timestamp_ms: u64,
}

impl HealthEvent {
    fn metadata(&self) -> EventMetadata {
        EventMetadata {
            trace_id: Some(format!("health-{}", self.timestamp_ms)),
            agent_id: None,
            territory_id: None,
            priority: Some(self.severity.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseRecord {
    pub lease_id: u64,
    pub resource_id: String,
    pub holder_id: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseQueueRecord {
    pub request_id: String,
    pub agent_id: String,
    pub resource_id: String,
    pub queue_position: usize,
    pub grace_deadline_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseEscalationRecord {
    pub agent_id: String,
    pub resource_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StateCheckpoint {
    pub checkpoint_id: String,
    pub captured_at_ms: u64,
    pub router: RouterReplayState,
    pub leases: LeaseReplayState,
    pub metrics: MetricsSnapshot,
}

impl StateCheckpoint {
    fn metadata(&self) -> EventMetadata {
        EventMetadata {
            trace_id: Some(self.checkpoint_id.clone()),
            agent_id: None,
            territory_id: None,
            priority: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RouterReplayState {
    pub total_dispatched: u64,
    pub last_priority: Option<String>,
    pub queue_depths: Vec<usize>,
}

impl RouterReplayState {
    pub fn apply_dispatch(&mut self, record: &RouterDispatchRecord) {
        self.total_dispatched = self.total_dispatched.saturating_add(1);
        self.last_priority = Some(record.effective_priority.clone());
        self.queue_depths = record.queue_depths.clone();
    }

    pub fn to_snapshot(&self) -> RouterSnapshot {
        let mut depth_map = BTreeMap::new();
        for (idx, depth) in self.queue_depths.iter().enumerate() {
            let label = Priority::from_index(idx).as_str().to_string();
            depth_map.insert(label, *depth);
        }
        RouterSnapshot {
            queue_depths: depth_map,
            last_dispatched_priority: self.last_priority.clone(),
            last_dispatched_at: None,
            rate_limited_messages: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LeaseReplayState {
    pub active: HashMap<String, LeaseRecord>,
    pub deferrals: u64,
    pub overrides: u64,
    pub escalations: u64,
}

impl LeaseReplayState {
    pub fn apply(&mut self, event: &LeaseEvent) {
        match event {
            LeaseEvent::Granted(record) => {
                self.active
                    .insert(record.resource_id.clone(), record.clone());
            }
            LeaseEvent::Released(record) => {
                self.active.remove(&record.resource_id);
            }
            LeaseEvent::Deferred(_) => {
                self.deferrals = self.deferrals.saturating_add(1);
            }
            LeaseEvent::Escalated(_) => {
                self.escalations = self.escalations.saturating_add(1);
            }
            LeaseEvent::Overridden { lease, .. } => {
                self.overrides = self.overrides.saturating_add(1);
                self.active.insert(lease.resource_id.clone(), lease.clone());
            }
        }
    }

    pub fn to_summary(&self) -> LeaseSnapshotSummary {
        let pending = BTreeMap::new();
        LeaseSnapshotSummary {
            active_leases: self.active.len(),
            total_pending: pending.values().copied().sum(),
            pending_by_resource: pending,
            deferrals: self.deferrals,
            overrides: self.overrides,
            escalations: self.escalations,
            outstanding_lease_ids: self.active.values().map(|record| record.lease_id).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ReplayOutcome {
    pub router: RouterReplayState,
    pub leases: LeaseReplayState,
    pub metrics: Option<MetricsSnapshot>,
    pub checkpoints: Vec<StateCheckpoint>,
    pub last_sequence: Option<u64>,
    pub tail_hash: Option<String>,
}

impl ReplayOutcome {
    fn update_from_checkpoint(&mut self, checkpoint: &StateCheckpoint) {
        self.metrics = Some(checkpoint.metrics.clone());
        self.router = checkpoint.router.clone();
        self.leases = checkpoint.leases.clone();
    }
}

#[derive(Clone)]
pub struct LedgerWriter {
    inner: Arc<LedgerInner>,
}

struct LedgerInner {
    config: LedgerRuntimeConfig,
    root: PathBuf,
    epoch_id: String,
    state: Mutex<WriterState>,
    clock: Mutex<HybridLogicalClock>,
    broadcaster: broadcast::Sender<EventEnvelope>,
}

#[derive(Clone)]
pub struct LedgerReader {
    root: PathBuf,
}

pub struct ReplayCoordinator {
    reader: LedgerReader,
}

#[derive(Debug)]
struct LedgerRuntimeConfig {
    segment_size_bytes: u64,
    segment_duration: Duration,
}

impl From<&LedgerConfig> for LedgerRuntimeConfig {
    fn from(config: &LedgerConfig) -> Self {
        Self {
            segment_size_bytes: config.segment_size_bytes,
            segment_duration: Duration::from_secs(config.segment_duration_secs.max(1)),
        }
    }
}

struct WriterState {
    file: BufWriter<File>,
    sequence: u64,
    prev_hash: String,
    segment_index: u32,
    bytes_written: u64,
    segment_opened_at: SystemTime,
}

impl WriterState {
    fn new(file: BufWriter<File>, now: SystemTime) -> Self {
        Self {
            file,
            sequence: 0,
            prev_hash: String::from("0"),
            segment_index: 0,
            bytes_written: 0,
            segment_opened_at: now,
        }
    }

    fn should_rotate(&self, now: SystemTime, config: &LedgerRuntimeConfig) -> bool {
        let size_exceeded = self.bytes_written >= config.segment_size_bytes;
        let time_exceeded = now
            .duration_since(self.segment_opened_at)
            .unwrap_or_default()
            >= config.segment_duration;
        size_exceeded || time_exceeded
    }
}

impl LedgerWriter {
    pub fn new(config: &LedgerConfig) -> LedgerResult<Self> {
        let root = config.root_path.clone();
        let epoch_id = config
            .current_epoch
            .clone()
            .unwrap_or_else(|| current_epoch_id());
        let runtime = LedgerRuntimeConfig::from(config);
        fs::create_dir_all(root.join(&epoch_id))?;
        let initial_file = open_segment(&root, &epoch_id, 0)?;
        let state = WriterState::new(initial_file, SystemTime::now());
        let (tx, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        Ok(Self {
            inner: Arc::new(LedgerInner {
                config: runtime,
                root,
                epoch_id,
                state: Mutex::new(state),
                clock: Mutex::new(HybridLogicalClock::default()),
                broadcaster: tx,
            }),
        })
    }

    pub fn epoch_id(&self) -> String {
        self.inner.epoch_id.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.inner.broadcaster.subscribe()
    }

    pub async fn append_async(&self, event: LedgerEvent) -> LedgerResult<EventEnvelope> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || inner.append(event)).await?
    }

    pub fn append_blocking(&self, event: LedgerEvent) -> LedgerResult<EventEnvelope> {
        self.inner.append(event)
    }

    pub async fn record_checkpoint(
        &self,
        checkpoint: StateCheckpoint,
    ) -> LedgerResult<EventEnvelope> {
        self.append_async(LedgerEvent::Checkpoint(checkpoint)).await
    }

    pub async fn flush(&self) -> LedgerResult<()> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || inner.flush()).await??;
        Ok(())
    }
}

impl LedgerInner {
    fn append(&self, event: LedgerEvent) -> LedgerResult<EventEnvelope> {
        let mut state = self.state.lock().unwrap();
        let mut clock = self.clock.lock().unwrap();
        let now = SystemTime::now();
        if state.should_rotate(now, &self.config) {
            state.segment_index = state.segment_index.saturating_add(1);
            state.bytes_written = 0;
            state.segment_opened_at = now;
            state.file = open_segment(&self.root, &self.epoch_id, state.segment_index)?;
        }
        let metadata = event.metadata();
        let logical_clock = LogicalClock::now(&mut clock);
        let payload_bytes = serde_json::to_vec(&event)?;
        let payload_digest = blake3::hash(&payload_bytes).to_hex().to_string();
        state.sequence = state.sequence.saturating_add(1);
        let mut envelope = EventEnvelope {
            epoch_id: self.epoch_id.clone(),
            sequence: state.sequence,
            logical_clock,
            metadata,
            payload_digest,
            hash_chain: String::new(),
            event,
        };
        let value = envelope.without_hash();
        let serialized_without_hash = serde_json::to_vec(&value)?;
        let mut hasher = Hasher::new();
        hasher.update(state.prev_hash.as_bytes());
        hasher.update(&serialized_without_hash);
        let hash_chain = hasher.finalize().to_hex().to_string();
        state.prev_hash = hash_chain.clone();
        envelope.hash_chain = hash_chain;
        let serialized = serde_json::to_vec(&envelope)?;
        state.file.write_all(&serialized)?;
        state.file.write_all(b"\n")?;
        state.file.flush()?;
        state.bytes_written = state
            .bytes_written
            .saturating_add(serialized.len() as u64 + 1);
        let _ = self.broadcaster.send(envelope.clone());
        Ok(envelope)
    }

    fn flush(&self) -> LedgerResult<()> {
        let mut state = self.state.lock().unwrap();
        state.file.flush()?;
        Ok(())
    }
}

impl LedgerReader {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn read_epoch(&self, epoch_id: &str) -> LedgerResult<Vec<EventEnvelope>> {
        let mut entries = Vec::new();
        let epoch_path = self.root.join(epoch_id);
        if !epoch_path.exists() {
            return Ok(entries);
        }
        let mut segments = collect_segments(&epoch_path)?;
        segments.sort();
        for segment in segments {
            let file = File::open(&segment)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                let envelope: EventEnvelope = serde_json::from_str(&line)?;
                entries.push(envelope);
            }
        }
        Ok(entries)
    }

    pub fn verify_epoch(&self, epoch_id: &str) -> LedgerResult<bool> {
        let events = self.read_epoch(epoch_id)?;
        let mut prev_hash = String::from("0");
        for event in events {
            let value = event.without_hash();
            let serialized_without_hash = serde_json::to_vec(&value)?;
            let mut hasher = Hasher::new();
            hasher.update(prev_hash.as_bytes());
            hasher.update(&serialized_without_hash);
            let expected = hasher.finalize().to_hex().to_string();
            if expected != event.hash_chain {
                return Ok(false);
            }
            prev_hash = event.hash_chain;
        }
        Ok(true)
    }
}

impl ReplayCoordinator {
    pub fn new(reader: LedgerReader) -> Self {
        Self { reader }
    }

    pub fn replay_epoch(&self, epoch_id: &str) -> LedgerResult<ReplayOutcome> {
        let events = self.reader.read_epoch(epoch_id)?;
        let mut outcome = ReplayOutcome::default();
        for envelope in events.iter() {
            match &envelope.event {
                LedgerEvent::Router(event) => match event {
                    RouterEvent::Dispatched(record) => outcome.router.apply_dispatch(record),
                    RouterEvent::RateLimited(_) => {}
                },
                LedgerEvent::Lease(event) => outcome.leases.apply(event),
                LedgerEvent::Consensus(_) => {}
                LedgerEvent::Pty(_) => {}
                LedgerEvent::Health(_) => {}
                LedgerEvent::Checkpoint(checkpoint) => {
                    outcome.checkpoints.push(checkpoint.clone());
                    outcome.update_from_checkpoint(checkpoint);
                }
            }
            outcome.last_sequence = Some(envelope.sequence);
            outcome.tail_hash = Some(envelope.hash_chain.clone());
        }
        if outcome.metrics.is_none() {
            let router_snapshot = outcome.router.to_snapshot();
            let lease_summary = outcome.leases.to_summary();
            let metrics = MetricsSnapshot {
                performance: Default::default(),
                router: router_snapshot,
                rate_limits: vec![],
                leases: lease_summary,
                pty: Default::default(),
                system: Default::default(),
                ledger: Default::default(),
                consensus: ConsensusSnapshot::default(),
                heat: HeatSnapshot::default(),
            };
            outcome.metrics = Some(metrics);
        }
        Ok(outcome)
    }
}

fn open_segment(root: &Path, epoch_id: &str, index: u32) -> LedgerResult<BufWriter<File>> {
    let dir = root.join(epoch_id);
    fs::create_dir_all(&dir)?;
    let file_path = dir.join(format!("segment_{index:04}.log"));
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(file_path)?;
    Ok(BufWriter::new(file))
}

fn collect_segments(epoch_path: &Path) -> LedgerResult<Vec<PathBuf>> {
    let mut segments = Vec::new();
    if epoch_path.is_dir() {
        for entry in fs::read_dir(epoch_path)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("segment_"))
                .unwrap_or(false)
            {
                segments.push(path);
            }
        }
    }
    Ok(segments)
}

fn current_epoch_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("epoch-{now}")
}
