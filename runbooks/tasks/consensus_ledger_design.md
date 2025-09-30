# Consensus Ledger Design Note (Turn 1)

The coordination epoch needs an auditable spine that can replay router and territory behavior without uncertainty. Drawing from the replay determinism guidance in `research/distributed-systems/replay-determinism-ledgers.md` and the leaderless fault tolerance patterns in `research/consensus-mechanisms/byzantine-fault-tolerance.md`, this note specifies the append-only ledger, deterministic replay controls, and the quorum signals that translate swarm-style cues into verifiable system records. The design keeps the router as the single sequencing authority described in `docs-canonical/reference/02_interaction_model.md` while extending territory lifecycle tracking from `docs-canonical/conceptual/03_lease_lifecycle.md`. Replay tooling will integrate with the integration harness in `docs-canonical/reference/03_testing_framework.md` so we can prove the ledger closes the loop between live execution and simulation.

## Append-Only Ledger Structure
- **Ledger scope**: Capture router deliveries (enqueue → dispatch → ack), lease lifecycle decisions, PTY spawn/exit, consensus signals, and system health annotations. Event taxonomy: `RouterEvent`, `LeaseEvent`, `ConsensusEvent`, `PtyEvent`, `HealthEvent`.
- **Envelope schema**: Each entry is an `EventEnvelope` with stable ordering fields (`epoch_id`, `sequence`, `logical_ts`, `trace_id`, `agent_id`, `territory_id`, `priority`, `event_type`, `payload_digest`, `hash_chain`, `signature_opt`). `sequence` is a monotonic u64 assigned by the router thread to guarantee total order (`replay-determinism-ledgers.md`). `logical_ts` is a hybrid logical clock (HLC) to preserve causality without coordination overhead.
- **Storage layout**: Append to segment files under `ledger/{epoch_id}/segment_{n}.log` with optional mirrored sled tree for indexed lookups. Segments rotate at configurable `max_segment_size` or `max_segment_duration`. A memory-mapped reader supports sequential scanning; metadata index stores checkpoints (`sequence`, `file_offset`, `state_digest`).
- **Retention**: Keep hot segments on fast storage for the active epoch, roll cold segments to compressed archives with pluggable retention policies (`retain_epochs`, `retain_days`). Policy defaults to retaining the last 7 epochs plus any with unresolved incidents. Hash chains (`hash_chain = blake3(prev_hash || serialized_envelope)`) enforce tamper evidence as suggested in `replay-determinism-ledgers.md`.
- **Payload handling**: Payloads store minimal structured data (e.g., JSON or bincode) with explicit schema versions. Large blobs (console output) remain in PTY artifact storage; ledger stores content digests and references.

## Deterministic Replay & Recovery
- **Ordering guarantees**: The router assigns the canonical sequence before dispatch. Downstream emitters enqueue ledger entries through a buffered `LedgerWriter` so that final log order matches dispatch order even when subsystems run asynchronously. If any event originates outside the router (e.g., lease monitor), it submits via the router’s sequencing channel to avoid divergent interleavings (`replay-determinism-ledgers.md`).
- **Checkpoints**: Periodic `StateCheckpoint` events capture snapshots of router queues, active leases, PTY states, and heat map counters. Checkpoints include a `state_digest` referencing serialized state persisted alongside ledger segments. Replay can fast-forward to the latest checkpoint and then replay incremental events, matching event-sourcing recommendations from the research note.
- **Replay workflow**: `ReplayCoordinator` consumes segments in order, verifies hash chains, rebuilds metrics/territory state, and produces deterministic metrics snapshots. For failure recovery, the system rehydrates from the latest stable checkpoint and replays remaining events before accepting new work, mirroring partially synchronous recovery assumptions in BFT research.
- **Failure handling**: On detection of hash mismatch or missing segment, mark the ledger as dirty and trigger consensus escalation (described below). Recovery relies on quorum approval before truncating or resequencing entries. During live operation, flush writes with fsync batching to keep durability under 10 ms while allowing throughput.
- **Testing hooks**: The integration harness adds a `replay_epoch` scenario that ingests a captured segment and asserts identical router queue depths, lease ownership, and metrics, aligning with `docs-canonical/reference/03_testing_framework.md`.

## Quorum & Consensus Signal Capture
- **Quorum metadata**: Every lease negotiation records `quorum_vector` containing participating agents, decision weights, and quorum threshold (default `2/3` for critical leases per `byzantine-fault-tolerance.md`). Router overrides and Director interventions log explicit quorum bypass tags for audit.
- **Consensus envelopes**: Introduce `ConsensusEvent::Proposal`, `::Vote`, `::Commit`, mirroring leaderless BFT phases (HoneyBadger-style committee aggregation from the research doc). Each includes randomness seeds or view numbers when applicable. Commit events embed the resulting decision hash that downstream modules verify during replay.
- **Stigmergic signals**: Inspired by `research/biological-coordination/quorum-sensing-stigmergy.md`, contested territories record `heat_score` samples in `LeaseEvent` payloads alongside decay parameters. Ledger readers can reconstruct heat maps to inform future scheduling and detect hotspots.
- **Alerting**: Ledger entries mark consensus failure conditions (`timeout`, `conflicting_commit`, `insufficient_quorum`). Replay uses the same definitions to reproduce alerts, guaranteeing accountability when consensus breaks down.

## Component Interfaces & Integration Points
- **LedgerWriter API**: Synchronous interface for critical path (`append_blocking(event)`) plus async batcher (`append_async(event)`). Accepts structured events and handles serialization, hash chaining, and segment rotation. Provides `flush()` for tests and shutdown.
- **LedgerReader API**: Streaming iterator (`stream(range)`) and indexed lookup (`get(sequence)`). Exposes `verify_segment(segment_id)` for integrity checks. Offers tailing subscription for UI visualizations or metrics streaming.
- **ReplayCoordinator**: Consumes a `ReplayPlan { epoch_id, start_sequence, end_sequence_opt, apply_actions }`, rebuilds state structs (`RouterState`, `LeaseState`, `HeatMapState`), and emits `ReplayOutcome` metrics. Command exposed through Tauri (`cmd::ledger_replay(plan)`) and CLI (`cargo run -- ledger replay --epoch ...`).
- **ConsensusBroker**: New hub that ingests quorum votes, enforces thresholds, and appends `ConsensusEvent`s. Exposes `register_quorum(kind, threshold, participants)` for territory and router modules. Publishes summarized metrics to existing telemetry collectors.
- **Metrics/Observability**: Ledger integrates with Prometheus exporters to emit `ledger_append_latency`, `segment_durable_lag`, `quorum_failure_total`, and `heat_score_percentile`. Frontend can subscribe to `LedgerReader::stream_tail()` for visualization.
- **Configuration**: Extend `liminal.config.yaml` under `ledger` with `segment_size_bytes`, `segment_duration_secs`, `retain_epochs`, `checkpoint_interval`, `quorum_thresholds`, and `heat_decay`. Defaults chosen to meet replay <500 ms for 10k events and retention aligned with operational needs.

## Implementation Checklist (Turns 2–3)
- [ ] Scaffold `ledger` crate/module with `EventEnvelope`, event enums, serialization, and hash chaining.
- [ ] Implement `LedgerWriter` + `LedgerReader`, segment management, and configuration wiring.
- [ ] Integrate router dispatch, lease lifecycle manager, PTY orchestrator, and health monitor to emit structured ledger events.
- [ ] Add checkpoint writer and replay utilities (`ReplayCoordinator`, deterministic state rebuild) plus CLI/Tauri entry points.
- [ ] Extend integration tests to capture an epoch, replay it, and assert identical router/lease/metrics state.
- [ ] Implement quorum tracking (`ConsensusBroker`), capture quorum vectors, and log consensus failure modes.
- [ ] Wire stigmergic heat map updates and decay parameters into lease events and metrics snapshots.
- [ ] Emit Prometheus metrics and UI tail subscription using the ledger reader.
- [ ] Document new configuration knobs and ledger operations in developer docs.
