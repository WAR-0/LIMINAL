# LIMINAL Integration Test Specification

This document defines the high-level integration test plan for validating LIMINAL's multi-agent orchestration system. The focus is on end-to-end scenarios that test the interaction between core components.

## 1. Core Performance Tests

### 1.1. Message Throughput & Latency
-   **Objective:** Validate the router can handle 1000+ messages per second with a p99 latency under 1ms.
-   **Method:** Generate a high volume of messages with varied priorities and measure routing time.

### 1.2. Clone Spawn Performance
-   **Objective:** Validate that clone creation, including context snapshotting, consistently meets the **<10ms** performance target.
-   **Method:** Repeatedly spawn clones with typical context sizes and measure the end-to-end time from request to clone readiness.

<!-- [[EDITOR_NOTE_CLAUDE: Insert the detailed test case for the Spatial Hash Efficiency Test, including setup and assertions for O(1) lookups.]] -->

## 2. Multi-Agent Coordination Tests

### 2.1. Lease Conflict Resolution
-   **Objective:** Test the full range of the lease negotiation algorithm, including deferral, escalation, and priority inheritance.
-   **Scenarios:**
    -   A high-priority agent requests a lease from a low-priority agent.
    -   Multiple agents queue for the same resource, triggering an escalation.
    -   A high-priority agent depends on a resource held by a low-priority agent, testing priority inheritance.

<!-- [[EDITOR_NOTE_CLAUDE: Insert the detailed test case for the full Clone Discussion Lifecycle, from initiation through consensus to merging the resolution.]] -->

## 3. System Recovery and Fault Tolerance Tests

### 3.1. Agent Process Crash Recovery
-   **Objective:** Ensure the system gracefully handles an unexpected agent process termination.
-   **Method:** Manually kill an agent process mid-task.
-   **Expected Outcome:** The agent's leases are released, its clones are terminated, and the Director is notified.

### 3.2. Deadlock Detection and Resolution
-   **Objective:** Verify that the system can detect and escalate circular lease dependencies.
-   **Method:** Create a scenario where Agent A waits for a resource from Agent B, and Agent B waits for a resource from Agent A.
-   **Expected Outcome:** The deadlock is detected within a short timeout, and the conflict is escalated to the Director for resolution.

## 4. Message Priority Queue Tests

### 4.1. Priority Queue Validation
-   **Objective:** Test that the router correctly handles messages according to their priority.
-   **Scenarios:**
    -   **Strict Ordering:** Ensure `DirectorOverride` and `Critical` messages are always processed first.
    -   **Starvation Prevention:** Verify that a continuous stream of high-priority messages does not completely starve low-priority `Info` messages.
    -   **Token Bucket:** Test that the rate limiting for `Critical` messages works correctly, downgrading excessive messages to `Blocking`.

## 5. End-to-End Scenarios

### 5.1. Complete Development Cycle
-   **Objective:** Test a full feature implementation from goal definition to completion.
-   **Method:** Provide a high-level goal (e.g., "Add user authentication") and allow the AI team to execute the full cycle: planning, approval, parallel execution, clone discussions, lease conflicts, and final integration.
-   **Success Criteria:** The feature is implemented correctly, and key performance metrics (e.g., parallelism, clone spawn time) are met.

<!-- [[EDITOR_NOTE_CLAUDE: Insert the detailed test case for the Stress Test scenario, involving 20+ parallel agents and graceful degradation under overload.]] -->

---
These integration tests will ensure that LIMINAL meets its core requirements for performance, reliability, and functionality.