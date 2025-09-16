# LIMINAL Integration Tests

This document outlines the initial set of integration tests designed to validate the core functionality and robustness of the LIMINAL multi-agent orchestration system.

## 1. Multi-Agent Lease Conflict Scenarios

**Objective**: Verify that the TerritoryManager correctly handles resource conflicts between multiple agents according to the defined lease negotiation algorithm.

-   **Test 1.1: High-Priority Preemption**
    -   **Scenario**: Agent A (Normal Priority) holds a lease on `src/api.ts`. Agent B (High Priority) requests the same lease.
    -   **Expected Outcome**: Agent B is granted the lease immediately. Agent A's lease is revoked, and it is notified.

-   **Test 1.2: Deferral of Low-Priority Request**
    -   **Scenario**: Agent A (High Priority) holds a lease on `src/components/Button.tsx` with 20 seconds remaining. Agent B (Low Priority) requests the lease.
    -   **Expected Outcome**: Agent B's request is deferred. It is notified that the resource will be available in approximately 20 seconds. The lease is automatically transferred to Agent B upon release by Agent A.

-   **Test 1.3: Escalation on High-Priority Conflict**
    -   **Scenario**: Agent A (High Priority) holds a lease. Agent B (High Priority) and Agent C (High Priority) both request the same lease.
    -   **Expected Outcome**: The conflict is escalated to the Human Director. The UI presents the conflict and awaits manual intervention.

-   **Test 1.4: Deadlock Detection and Escalation**
    -   **Scenario**: Agent A requests a lease held by Agent B. Simultaneously, Agent B requests a lease held by Agent A.
    -   **Expected Outcome**: The system detects the potential deadlock and immediately escalates the situation to the Human Director.

## 2. Clone Discussion Lifecycle

**Objective**: Validate the end-to-end lifecycle of a clone discussion, from spawning to consensus and merging.

-   **Test 2.1: Successful Consensus and Merge**
    -   **Scenario**: Agent A spawns a clone to discuss an API change with Agent B. The clones reach an agreement.
    -   **Expected Outcome**: The clone discussion is marked as resolved. The consensus resolution is successfully merged back into the primary agents' context without conflicts.

-   **Test 2.2: Clone Discussion Timeout**
    -   **Scenario**: Two clones are spawned for a discussion but fail to reach consensus within the specified timeout period.
    -   **Expected Outcome**: The discussion is automatically escalated to the Human Director for a decision.

-   **Test 2.3: Parent Agent Continues Work**
    -   **Scenario**: Agent A spawns a clone for a non-blocking discussion.
    -   **Expected Outcome**: Agent A continues to perform other tasks while its clone handles the discussion in parallel. The parent agent's work is not blocked.

## 3. System Recovery and Fault Tolerance

**Objective**: Ensure the system can gracefully handle agent failures.

-   **Test 3.1: Managed Agent Process Crash**
    -   **Scenario**: A subordinate agent's process is manually terminated (e.g., `kill -9`).
    -   **Expected Outcome**: The Unified Message Router detects the process failure. Any leases held by the crashed agent are immediately released. The agent is marked as `Error` in the UI, and the Human Director is notified.

-   **Test 3.2: State Recovery on Restart**
    -   **Scenario**: The main LIMINAL application is shut down and restarted during an active turn.
    -   **Expected Outcome**: Upon restart, the system correctly restores the state of all agents, leases, and ongoing discussions from the persistence layer.

## 4. Message Priority Queue Validation

**Objective**: Verify that the Unified Message Router correctly prioritizes and delivers messages based on their assigned priority level.

-   **Test 4.1: Critical Message Preemption**
    -   **Scenario**: A queue contains multiple `Info` and `Coordinate` messages. A `Critical` message is then enqueued.
    -   **Expected Outcome**: The `Critical` message is delivered to the recipient agent immediately, preempting all other messages in the queue.

-   **Test 4.2: Starvation Prevention**
    -   **Scenario**: A continuous stream of `Blocking` messages is sent to an agent, while several `Info` messages are also in the queue.
    -   **Expected Outcome**: The `Info` messages are eventually delivered, demonstrating that the starvation prevention mechanism (e.g., aging or weighted fair queuing) is working.

-   **Test 4.3: Dynamic Priority Escalation**
    -   **Scenario**: A `Blocking` message is sent but the recipient agent does not process it within the defined timeout.
    -   **Expected Outcome**: The message is automatically escalated to `Critical` priority, and an alert is raised to the Human Director.
