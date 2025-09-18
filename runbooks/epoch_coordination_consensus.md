# Runbook: Coordination & Stability Epoch

**Epoch Goal:** Layer consensus, scheduling, and stability controls on top of the router foundation while cleaning up lingering technical debt and aligning documentation with the evolved system.

**Team Shape:** Human Director orchestrating Claude (primary specialist) with optional support agents for docs or research distillation.

**Key References:**
- `research/distributed-systems/replay-determinism-ledgers.md`
- `research/consensus-mechanisms/byzantine-fault-tolerance.md`
- `research/biological-coordination/quorum-sensing-stigmergy.md`
- `research/distributed-systems/work-stealing-scheduling.md`
- `research/phase-dynamics/critical-phenomena.md`
- `research/phase-dynamics/phase-locking-synchronization.md`
- `research/territorial-allocation/spatial-computing-models.md`
- `docs-canonical/reference/02_interaction_model.md`
- `docs-canonical/conceptual/03_lease_lifecycle.md`
- `docs-canonical/reference/03_testing_framework.md`

---

## Turn 1 — Consensus & Ledger Design Brief
**Specialist:** Claude (architecture reasoning)

**Prompt to Delegate:**
> Using the replay-determinism and BFT research, draft a 1–2 page design note covering:
> - Structure of an append-only event ledger for router/territory actions (schema, storage, retention).
> - Deterministic replay strategy (ordering guarantees, checkpoints, failure recovery).
> - Quorum/consensus signals to capture (e.g., quorum thresholds, lease agreement decisions) and how they surface in the ledger.
> - Interfaces new components expose to the rest of the system.
> Reference the relevant research docs and conclude with an implementation checklist for Turns 2–3.

**Acceptance:** Design note saved to `runbooks/tasks/consensus_ledger_design.md` with clear checklist.

---

## Turn 2 — Implement Event Ledger & Replay Hooks
**Specialist:** Claude (Rust implementation)

**Prompt to Delegate:**
> Implement the ledger per the Turn 1 checklist:
> - Add an append-only event log module (configurable path, in-memory + on-disk) that records router deliveries, lease decisions, PTY events.
> - Provide deterministic replay utilities (rebuild metrics state, verify ordering) and expose a `replay_epoch` command for testing.
> - Include checksum or hash chaining to detect tampering/corruption.
> - Extend integration tests to replay a captured scenario and assert identical outcomes.
> Run `cargo fmt`, `cargo test`, and summarize results.

**Acceptance:** Ledger module with tests demonstrating replay determinism and integrity checks.

---

## Turn 3 — Quorum & Stigmergic Consensus Signals
**Specialist:** Claude (Rust implementation + metrics)

**Prompt to Delegate:**
> Build quorum/stigmergic coordination features:
> - Introduce quorum threshold tracking (e.g., number of agents agreeing on a lease or router decision) using inspirations from quorum sensing research.
> - Add stigmergic “heat” maps for contested resources (decaying counters per file/territory) to inform lease negotiation.
> - Emit consensus/quorum events into the ledger and metrics snapshots; trigger health alerts when quorum fails or heat exceeds thresholds.
> - Update tests to cover quorum achievement, quorum failure fallback, and stigmergic decay.
> Run standard format/test commands and report.

**Acceptance:** Quorum/stigmergy signals wired into metrics + ledger, with tests verifying behavior.

---

## Turn 4 — Work-Stealing Execution Pool
**Specialist:** Claude (Rust implementation)

**Prompt to Delegate:**
> Using the work-stealing research, implement a lightweight task scheduler for internal jobs (e.g., router maintenance, clone tasks):
> - Create a work-stealing deque pool (Chase–Lev style) to run background jobs without central bottlenecks.
> - Migrate router/territory maintenance tasks (aging, heat decay, snapshots) onto the pool.
> - Expose hooks for future clone orchestration tasks.
> - Add tests validating fairness (tasks complete even under uneven load) and integration tests ensuring maintenance still runs under stress.
> Run `cargo fmt`, `cargo test`, summarize.

**Acceptance:** Work-stealing executor integrated with maintenance tasks and covered by tests.

---

## Turn 5 — Phase Stability Monitoring
**Specialist:** Claude (metrics/analysis)

**Prompt to Delegate:**
> Instrument the system with edge-of-chaos monitoring inspired by phase dynamics research:
> - Define order parameters (e.g., message variance, queue oscillation, lease churn) and compute them in metrics snapshots.
> - Detect phase transition indicators (critical thresholds) and emit alerts or annotations in the dashboard.
> - Visualize recent history in the frontend (sparkline or trend indicators for order parameters).
> - Add tests/simulations causing transitions (e.g., synthetic load pushing system near instability) and assert alerts fire.
> Run standard commands and document metrics meaning in code comments or docstrings.

**Acceptance:** Stability metrics and frontend visualization with tests demonstrating detection.

---

## Turn 6 — Territory Strategy Upgrade
**Specialist:** Claude (architecture + implementation)

**Prompt to Delegate:**
> Select and implement an improved territory allocation strategy drawing from spatial computing research:
> - Draft a short comparison (≤300 words) justifying the chosen method (e.g., CVT over embeddings, space-filling curve ranges, or stigmergic heat).
> - Implement the chosen approach (prototype acceptable) and integrate with the existing TerritoryManager.
> - Provide configuration knobs to switch strategies and emit telemetry comparing load balance.
> - Update tests covering territory assignment consistency and boundary negotiation with the new strategy.
> Run format/tests, summarize results.

**Acceptance:** Territory manager supporting the selected advanced strategy with tests and telemetry.

---

## Turn 7 — Codebase Cleanup & Lint Enablement
**Specialist:** Claude (repo hygiene)

**Prompt to Delegate:**
> Address technical debt accumulated during prior turns:
> - Remove or gate unused fields/helpers (router, territory, PTY) while preserving backward compatibility.
> - Finalize the npm lint task: add ESLint configuration (or equivalent) and update `npm run lint` to perform real checks.
> - Ensure `cargo clippy` runs clean (or document remaining justifications) and add to CI scripts if feasible.
> - Update README to describe lint/testing commands.
> Run `cargo fmt`, `cargo test`, `cargo clippy -- -D warnings`, `npm run lint`, and report.

**Acceptance:** Clean build with minimal warnings, real linting enabled, documentation updated for tooling.

---

## Turn 8 — Documentation & Knowledge Consolidation
**Specialist:** Claude (documentation)

**Prompt to Delegate:**
> Synchronize canonical docs and knowledge base with new capabilities:
> - Update relevant sections in `docs-canonical` (interaction model, lease lifecycle, testing framework) to incorporate ledger, quorum, work-stealing, and territory strategy updates.
> - Add a changelog entry or README section summarizing Coordination & Stability Epoch outcomes.
> - Append learnings or tips to the regression runbook if new scenarios emerged.
> - Ensure documentation references the new metrics/alerts vocabulary and shared temporal language.
> Validate links, run spell check if available, and provide a final summary for the Director Agent.

**Acceptance:** Documentation changes merged, README aligned, and final summary delivered.

---

## Completion
After Turn 8, compile an Epoch recap highlighting delivered features, operational status (including ledger replay results and stability metrics), open risks, and recommended focus for the next Epoch (e.g., clone orchestration, consensus hardening).
