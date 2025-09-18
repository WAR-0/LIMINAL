# Router Dispatch Design Note (Turn 1)

Aligned with the canonical interaction model (docs-canonical/reference/02_interaction_model.md) and the message priority research (research/ai_implementation/forge_message_priority_spec.md), the router will own a dedicated asynchronous dispatcher. `UnifiedMessageRouter::start` will spawn a `tokio::task::JoinHandle<DispatcherOutcome>` that loops on `dispatch_once`, pulling from prioritized queues and pausing between iterations on an awaitable `DispatcherGate`. The gate exposes a placeholder for recipient pause-points by deferring final delivery through a `DeliveryResolver` trait that can be wired to PTY pause hooks described in turn_manager_state_patterns (research/ai_implementation/turn_manager_state_patterns.md).

**Dispatcher structure**
- Maintain five priority queues (Info → DirectorOverride) stored in an array of `Arc<RwLock<VecDeque<QueuedMessage>>>` for lock-sharded pushes while the dispatcher reads via a single mutable view (docs-canonical/reference/02_interaction_model.md#priority-levels).
- Each pass applies `aging_index` updates before selecting the highest non-empty queue; if all token buckets are exhausted the dispatcher sleeps on a configurable backoff sourced from `liminal.config.yaml`.
- Delivery emits a `RouterEvent::Dispatched` through `MetricsCollector`, capturing latency and queue depth so health KPIs (config/liminal.config.yaml#health_monitoring_kpis) can track routing targets.

**Token bucket and aging mechanics**
- Per-sender `TokenBucket` (burst, refill_per_second, last_refill_at) enforces quotas; Director overrides inherit the unlimited multiplier from `agent_privileges.director` while clones are clamped as in the interaction spec.
- Starvation prevention uses a dual strategy: `AgingTracker` increments wait time; thresholds elevate the effective priority (Info→Coordinate, Coordinate→Blocking) as the spec prescribes; weighted slice counters guarantee at least one lower-tier dequeue every N passes if aging has not triggered (docs-canonical/reference/02_interaction_model.md#fairness, research/ai_implementation/forge_message_priority_spec.md#router-mechanics).
- Token exhaustion records a `RateLimited` outcome per message for escalation monitoring and re-queues the envelope after refilling instead of dropping outright.

**Per-message metadata**
- Extend `QueuedMessage` with `uuid`, `trace_id`, original `Priority`, `effective_priority`, `enqueued_at`, `last_attempt_at`, `retry_count`, `sender_kind`, `token_snapshot`, and `aging_boosts` to support escalation triggers (docs-canonical/reference/02_interaction_model.md#escalation) and downstream analytics.
- Store `pause_hint` (safe interruption marker) and `delivery_deadline` so the dispatcher can surface impending SLA breaches to the Director.
- Metrics emission packages these fields into `DispatchSample` for time-series logging and alerting.

**Router state changes**
- Introduce `struct UnifiedMessageRouter` → composite of `DispatcherConfig`, `TokenBucketTable`, `AgingTracker`, `MetricsCollector`, and queue array; expose `RouterHandle` for enqueue, drain, shutdown, and manual wake.
- Add `enum RouterEvent` for `Enqueued`, `Dispatched`, `Deferred`, `Escalated`, and `RateLimited` so observers (metrics, leases, PTY bridge) can subscribe through a broadcast channel described in rust_data_comms_design.md.
- Split supporting modules: `router::throttle` (token buckets), `router::aging`, `router::event`, keeping the public API under `router.rs` while tests target each module plus integration coverage in `src-tauri/tests/router_dispatch_test.rs`.

**Implementation checklist**
- [ ] Add DirectorOverride priority and queue scaffolding.
- [ ] Define `QueuedMessage`, `TokenBucket`, `AgingTracker`, and dispatcher config structures.
- [ ] Implement enqueue validation, token consumption, and wake logic.
- [ ] Build async dispatcher loop with pause-point placeholder and metrics emission.
- [ ] Emit router events for dispatch, deferral, rate limiting, and escalation triggers.
- [ ] Write integration tests for priority ordering, aging promotion, and quota exhaustion.
- [ ] Document configuration knobs and wire defaults from `liminal.config.yaml`.
