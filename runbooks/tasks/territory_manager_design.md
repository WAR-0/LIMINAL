# Territory Manager Data & Flow Note (Turn 3)

Grounded in the lease lifecycle diagram (`docs-canonical/conceptual/03_lease_lifecycle.md`) and interaction priorities (`docs-canonical/reference/02_interaction_model.md`), the updated Territory Manager will formalize lease state as a first-class record consumed by the message router and metrics layer. We align concurrency choices with the turn manager state patterns (`research/ai_implementation/turn_manager_state_patterns.md`) and reuse router event semantics so conflict resolution mirrors message escalations.

**Lease model**
- `LeaseId` (UUID) and `resource_id` (glob path or logical territory key).
- `holder_id`, `holder_role`, `holder_priority` (captured at acquisition per interaction model privileges) plus `holder_progress` (0.0–1.0 from agent heartbeats).
- Temporal fields: `granted_at`, `expires_at`, `last_heartbeat_at`, `grace_deadline` (derived from `territory_config.auto_extend_threshold`).
- Negotiation overlay: `queue_position`, `request_priority`, `negotiation_state` (`Idle`, `Queued`, `Negotiating`, `Deferred`, `Escalating`, `Overridden`, `Expired`), and `escalation_ticket` to crosslink with router escalations.
- Accounting metadata: `conflict_attempts`, `defer_count`, `override_count`, `release_cause` enum for metrics aggregation.

**Spatial hash & lookup**
- Maintain a `SpatialGrid` keyed by integer cell indices computed via `floor(coord / cell_size)`; default `cell_size` 64 units with overrides from config (`territory_config.spatial_hash.cell_size`, to add in Turn 4/7 wiring).
- Cells map to `Vec<LeaseId>` allowing O(1) neighborhood lookups; resources without coordinates fall back to a `NonSpatial` bucket.
- Hash updates happen inside a dedicated actor task that receives `GridCommand` messages from the main manager, matching the lock-avoiding guidance in the turn manager patterns doc for write-heavy sections.

**Negotiation & escalation flow**
- On new request: read current lease via `Arc<RwLock<LeaseTable>>` (read path is cheap per state patterns). If free, grant and initialize timers. If occupied, push a `LeaseRequest` onto the resource queue sorted by request priority, then:
  - If requester priority exceeds holder by `territory_config.escalation.queue_threshold_priority` (new knob), enter `Overridden` path, emit `RouterEvent::LeaseOverrideRequested`, and notify MetricsCollector.
  - If holder progress ≥ `territory_config.fairness.priority_boost_after` proportion, mark `Deferred` with grace deadline and schedule retry using async task.
  - Otherwise keep `Negotiating`, increment `conflict_attempts`, and monitor waiting duration against `territory_config.fairness.starvation_threshold`. Breach triggers `Escalating` and emits router escalation per interaction model escalation rules.
- Heartbeat updates refresh `holder_progress` and `last_heartbeat_at`; missing heartbeat beyond `territory_config.default_lease_duration` transitions to `Expired` and starts cleanup broadcast.

**Router & metrics integration**
- Broadcast `TerritoryEvent` (`Granted`, `Deferred`, `Overridden`, `Released`, `Expired`) into the router’s delivery channel so the dispatcher can boost pending coordination messages from affected agents (`docs-canonical/reference/02_interaction_model.md#fairness`).
- Extend `MetricsCollector` with lease tables: queue depth per resource, active lease counts per priority band, override/defer rate, starvation incidents, and spatial density heatmaps aligned with health KPIs (`config/liminal.config.yaml#territory_config`).

**Implementation checklist**
- [ ] Introduce lease structs, enums, and spatial grid modules with config-ready fields.
- [ ] Migrate `TerritoryManager` to actor + RwLock hybrid per state-pattern guidance, ensuring async-safe access.
- [ ] Implement acquisition/release/heartbeat APIs with negotiation state transitions and timers.
- [ ] Add router-facing events and metrics hooks for lease lifecycle moments.
- [ ] Persist queue ordering and aging logic so deferred requests resume correctly.
- [ ] Write tests covering override, deferral, starvation escalation, and spatial collisions.
