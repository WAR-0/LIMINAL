# LIMINAL Glossary

This document provides definitions for key terms and concepts used throughout the LIMINAL documentation.

---

### C

**Clone-Based Discussion**
: A mechanism where an agent spawns a lightweight, independent "clone" of itself to engage in a discussion with other agents' clones. This allows the primary agents to continue their work without being blocked, enabling asynchronous, parallel conversations for tasks like negotiation or consensus-building. The clone is created with a [**Context Snapshot**](./00_glossary.md#context-snapshot) of the parent's state.

**Context Snapshot**
: A point-in-time capture of an agent's state, used to initialize a clone for a discussion. LIMINAL uses an optimized process to create snapshots in under 10ms. It employs differential snapshots (sending only changes) for efficiency when the number of logical changes is below a certain threshold, and full snapshots otherwise.

### D

**Director Agent**
: The designated AI team lead responsible for tactical execution and low-level coordination. It breaks down high-level goals from the Human Director into executable plans, manages task assignments, and handles initial conflict resolution and escalations. It has elevated privileges but cannot issue `Critical` priority messages or unilaterally override leases.

### H

**Human Director**
: The strategic leader of the LIMINAL system (i.e., the developer). The Human Director sets the high-level vision, approves execution plans, makes critical decisions, and intervenes on escalations that the Director Agent cannot resolve. They retain ultimate authority and strategic control.

### P

**Pause Point**
: A natural, non-disruptive moment in an agent's execution cycle where it can safely receive and process incoming messages. Examples include the completion of a task, the end of a test run, or while waiting for a build process. The Unified Message Router delivers messages at these pause points to avoid interrupting an agent's "train of thought."

**Priority Levels**
: A 4+1 tier system used by the Unified Message Router to classify and prioritize messages, ensuring that urgent communications are handled promptly. The levels are:
- **Info (0):** For telemetry, logs, and non-urgent updates.
- **Coordinate (1):** For standard operations like tasking and routine coordination.
- **Blocking (2):** For messages where the sender is blocked waiting for a response.
- **Critical (3):** For system-level failures or urgent alerts that require immediate attention.
- **DirectorOverride (4):** An exclusive, absolute priority level reserved for the Director Agent for critical interventions.

### T

**Territory Leasing**
: A soft coordination mechanism for managing access to shared resources (e.g., files, directories). Instead of hard locks, agents acquire negotiable "leases" on territories. If another agent needs a leased resource, it can request a transfer, which is evaluated based on priority, progress, and other factors, potentially leading to a negotiation, deferral, or escalation. This system is managed by the **Territory Manager**.

### U

**Unified Message Router**
: The central nervous system of the LIMINAL architecture. It is a high-performance Rust engine that intelligently routes all messages between agents. It manages priority queues, detects pause points for non-blocking message delivery, and integrates with other core components like the Territory Manager and Clone Manager to orchestrate complex, asynchronous collaboration.
hestrate complex, asynchronous collaboration.
