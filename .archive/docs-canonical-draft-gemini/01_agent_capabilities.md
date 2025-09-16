# Agent Capabilities Guide

This document defines the agent-side logic and decision-making patterns for effectively leveraging LIMINAL's Unified Message Router. It provides concrete heuristics, decision trees, and implementation guidelines for agent developers.

## 1. Lease Management Heuristics

Agents must intelligently decide when to grant, deny, or defer lease transfer requests. The decision should be based on the requesting agent's priority, the agent's own current task progress, and the number of other agents waiting.

<!-- [[EDITOR_NOTE_CLAUDE: Insert the detailed decision matrix and Mermaid flowchart for lease negotiation here. This should include specific thresholds for time remaining, queue length, and task progress.]] -->

### Proactive Lease Management

-   **Planning:** Request leases just-in-time before they are needed.
-   **Releasing:** Release leases immediately when a task is complete.
-   **Extending:** Monitor task progress and extend leases proactively if more time is needed.

## 2. Intelligent Tool Usage: Clones vs. Direct Messages

The choice between spawning a clone for an asynchronous discussion and sending a direct message is critical for system efficiency.

-   **Spawn a Clone when:**
    -   The topic is complex and requires negotiation or consensus.
    -   Multiple agents are involved.
    -   The discussion is long-running and should not block the primary agent's work.
-   **Send a Direct Message when:**
    -   The communication is a simple, one-way notification.
    -   The information is urgent and requires immediate attention.

<!-- [[EDITOR_NOTE_CLAUDE: Insert the Mermaid flowchart illustrating the decision process for choosing between a clone and a direct message.]] -->

### Context Snapshot Strategy

Agents must be strategic about the context they provide to clones to stay within the **<10ms** spawn target.

-   **Minimalism:** Provide only the essential context for the specific discussion.
-   **Differential Snapshots:** When possible, send only the changes from a previous snapshot.

## 3. Message Priority Determination

Agents are responsible for assigning the correct priority to their outgoing messages.

-   **Critical:** Reserved for system failures or issues that completely block all work.
-   **Blocking:** For messages that require a response before the sender can proceed (e.g., lease negotiations, consensus requests).
-   **Coordinate:** For standard task handoffs and status updates.
-   **Info:** For background information, progress updates, and telemetry.

### Dynamic Priority Adjustment

The system includes mechanisms for dynamic priority adjustment, such as escalating a message's priority if it waits in a queue for too long. Agents should be aware of these mechanisms but should focus on assigning the correct initial priority.

## 4. Error Recovery

Agents must be able to recover from common errors gracefully.

<!-- [[EDITOR_NOTE_CLAUDE: Insert the Mermaid flowchart for the error recovery decision tree, covering recoverable, resource, communication, and fatal errors.]] -->

## 5. Director Agent Special Capabilities

The AI Director Agent has unique capabilities for orchestrating the team, including handling escalations from other agents and monitoring overall system health. It serves as the first line of conflict resolution before escalating to the Human Director.

---
This guide provides agents with the decision-making framework needed to operate effectively within LIMINAL. The key principle is that agents should behave like considerate team members: respecting others' work, communicating asynchronously when possible, and escalating to the AI Director or Human Director only when necessary.