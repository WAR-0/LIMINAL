# Runbook: Router Foundation Epoch

**Epoch Goal:** Establish production-grade primitives for LIMINAL’s backend core so future clone orchestration and advanced observability can build on a stable base.

**Team Shape:** Human Director (execution lead) working with Claude (coding specialist) and other agents as needed.

**References:**
- `docs-canonical/reference/02_interaction_model.md`
- `docs-canonical/conceptual/03_lease_lifecycle.md`
- `research/ai_implementation/ai_implementation_forge_message_priority_spec.md`
- `research/ai_implementation/ai_implementation_turn_manager_state_patterns.md`
- `research/ai_implementation/ai_implementation_rust_data_comms_design.md`
- `config/liminal.config.yaml`

---

## Turn 1 — Router Dispatch Design Sync
**Specialist:** Claude (architecture reasoning)

**Prompt to Delegate:**
> Review the current implementation of `UnifiedMessageRouter` and the canonical interaction model. Produce a concise design note covering:
> - The async dispatcher structure (task model, queue selection, pause-point placeholder)
> - Token bucket & aging mechanics to prevent starvation
> - Metadata that must be stored per message for metrics and escalation
> - Any required structural changes to router state ( structs, enums, supporting modules )
> Cap the output at 500 words and reference relevant doc sections. Close with an implementation checklist.

**Acceptance:** Design note committed to `runbooks/tasks/router_dispatch_design.md` (or similar) plus clear checklist.

---

## Turn 2 — Implement Router Dispatch & Tests
**Specialist:** Claude (Rust implementation)

**Prompt to Delegate:**
> Using the design checklist, implement the router dispatch loop in `src-tauri/src/router.rs`:
> - Add dispatcher task that drains queues by priority with token bucket enforcement and aging
> - Emit delivery events to `MetricsCollector`
> - Update or add tests under `src-tauri/tests/` to cover priority ordering, aging boost, and quota exhaustion
> Run `cargo fmt`, `cargo test`, and summarize results.

**Acceptance:** Passing tests, updated router code, and metrics hooks. Summary includes follow-up TODOs if any.

---

## Turn 3 — Territory Manager Design Update
**Specialist:** Claude (architecture reasoning)

**Prompt to Delegate:**
> Read current `territory.rs`, canonical lease lifecycle docs, and decision matrices. Draft a data model + algorithm note that specifies:
> - Lease struct fields (priority, holder progress, expiry, queue, negotiation state)
> - Spatial hash approach (cell size, hashing strategy)
> - Transfer/negotiation flow including escalation triggers from config
> - Integration points with router metrics/events
> Keep it under 500 words with diagram or table if helpful. Provide implementation checklist.

**Acceptance:** Design artifact saved alongside Turn 1’s note.

---

## Turn 4 — Implement Soft-Lease Manager
**Specialist:** Claude (Rust implementation)

**Prompt to Delegate:**
> Implement the lease system per the Turn 3 checklist:
> - Introduce spatial hash helper and lease structs
> - Support acquire, release, transfer, negotiation queueing, and escalation signaling
> - Wire config-driven thresholds from `liminal.config.yaml`
> - Extend tests covering conflict resolution, deferral, and escalation triggers
> Execute `cargo fmt` + `cargo test` and report.

**Acceptance:** New lease logic with tests, config usage, and emitted events.

---

## Turn 5 — PTY Protocol Hardening
**Specialist:** Claude (Rust implementation)

**Prompt to Delegate:**
> Update `agent.rs` to follow the structured PTY protocol research:
> - Wrap outbound structured events in `<LIMINAL_EVENT>` (or chosen tag)
> - Replace blocking reader with buffered parser that extracts tagged JSON messages
> - Surface parsed events to the main application (router/metrics) through a channel or async stream
> - Add unit/integration coverage for parsing partial tags
> Run `cargo fmt`, `cargo test`. Include any manual scenario steps.

**Acceptance:** Robust PTY communication with parsing tests and summary of integration outcomes.

---

## Turn 6 — Metrics & Observability Surface
**Specialist:** Claude (full-stack)

**Prompt to Delegate:**
> Extend metrics and UI for backend validation:
> - Expand `MetricsCollector` to track queue depths, token bucket status, lease tables, escalation counters
> - Add Tauri commands/events exposing this data
> - Update React UI to display queue depth, lease occupancy, and key counters in a simple dashboard
> - Provide quick toggles/hooks for synthetic load or lease contention simulation
> Run `cargo fmt`, `cargo test`, `npm run lint`, and capture screenshots if feasible.

**Acceptance:** Working dashboard with backend metrics and optional simulation toggles.

---

## Turn 7 — Config Wiring & Health Alerts
**Specialist:** Claude (Rust implementation)

**Prompt to Delegate:**
> Integrate `liminal.config.yaml` into runtime state:
> - Load config on startup and inject values into router/territory/token bucket initialization
> - Implement health alert hooks when thresholds breach (log + event emission)
> - Add regression tests ensuring config overrides are honored
> Run standard format/lint/test commands and summarize.

**Acceptance:** Config-driven behavior verified by tests/logs.

---

## Turn 8 — Regression Scenario & Documentation Update
**Specialist:** Claude (testing/documentation)

**Prompt to Delegate:**
> Create an end-to-end regression scenario that exercises:
> - Mixed priority traffic hitting token limits
> - Lease contention with deferral/escalation
> - PTY agents emitting structured events consumed by the router
> Document the scenario in `runbooks/tasks/router_foundation_regression.md`, update integration tests if needed, and refresh `docs-canonical` references (e.g., note implemented features). Provide a final summary for the Director Agent.

**Acceptance:** Scenario doc, passing tests, and documentation updates summarizing new capabilities.

---

## Completion
When all Turns report success, compile a short Epoch recap outlining shipped components, remaining risks, and next-sprint candidates (clone orchestration, advanced UI, etc.) before handing back to the Director Agent.
