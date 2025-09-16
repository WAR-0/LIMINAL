# LIMINAL Integration Test Scenarios

This document outlines high-value integration tests for LIMINAL’s multi-agent orchestration stack. Tests focus on end-to-end behaviors across the Unified Message Router, TerritoryManager, Clone Orchestrator, PTY process management, and MCP tool interface.

## Test Harness Assumptions
- Local-first, single-machine runtime
- Mock or real agents connected via PTY or MCP
- Deterministic seedable timing for reproducible runs
- Structured events use `<LIMINAL_EVENT>` tags

## Scenarios

### 1) Multi-Agent Lease Conflicts and Resolution
- Setup:
  - Backend Agent holds lease on `api/users.ts` (remaining ~45s)
  - Frontend Agent requests transfer with priority=Coordinate
  - Testing Agent requests same resource later (queue length grows)
- Expected:
  - TerritoryManager applies policy defaults:
    - Defer if `time_remaining < DEFAULT_DEFER_THRESHOLD` or progress > 80%
    - Escalate if `queuedRequests > DEFAULT_QUEUE_ESCALATE`
  - Frontend receives `Defer(eta)`, Testing triggers escalation when queue > 2
  - Human Director prompt raised on escalation; override path validated
  - Leases auto-release on completion; next-in-queue notified
- Metrics:
  - Transfer decision latency < 100ms
  - No deadlocks; fairness maintained

### 2) Clone Discussion Lifecycle (spawn → consensus → merge → terminate)
- Setup:
  - Frontend and Backend need API contract agreement
  - Frontend spawns clone with context snapshot (Full or Differential)
  - Router delivers message at natural pause point; Backend spawns clone
- Expected:
  - Snapshot generation < 10ms; delivery < 50ms total
  - Discussion thread records proposal, counter, agreement
  - Consensus Engine returns Approved; StateMerger applies diff atomically
  - Parent agents continue primary work unblocked; clones terminated post-merge
- Metrics:
  - Zero parent blocking during discussion
  - Merge conflict rate ~0 in clean scenario; rollback verified on injected conflict

### 3) Managed Agent Process Crash and Recovery
- Setup:
  - Spawn agent under PTY; simulate crash mid-task
  - In-flight messages and leases exist
- Expected:
  - Router detects PTY termination; emits Critical alert
  - Leases released or marked reclaimable after grace period
  - Message retries or rerouting per policy; persistent state reloaded on restart
  - Human Director notified with actionable recovery options
- Metrics:
  - No message loss; persisted audit trail complete
  - Recovery within configured timeout; system stabilizes

### 4) Message Priority Queue Under Mixed Load
- Setup:
  - Mix of Info, Coordinate, Blocking, Critical messages across 5–10 agents
  - Enable aging and dynamic escalation (Blocking→Critical on timeout)
- Expected:
  - Strict priority dispatch honored; starvation prevented (aging/weighted RR)
  - Cooperative preemption respected at agent natural pause points
  - Director Override bypass verified (rare, audited)
- Metrics:
  - Routing latency: p95 < 100ms, p99 < 200ms
  - Throughput: >1000 messages/sec sustained without loss

## Instrumentation & Assertions
- Event log verification for every state transition (event-sourced audit)
- Timestamped spans: snapshot, route, deliver, process, merge
- Queue depth tracking per priority; aging promotions recorded
- Lease decision traces with inputs and chosen outcome

## Failure Injection Matrix
- PTY partial/malformed `<LIMINAL_EVENT>` frames
- Delayed agent pause points; long atomic sections
- Territory conflict storms (>100 concurrent requests)
- Storage failures (simulate transient DB write errors)

## Exit Criteria (MVP)
- All scenarios pass under stress for 1 hour
- Zero deadlocks; bounded latencies; stable memory (<500MB typical session)
- Complete audit trail and recovery correctness

