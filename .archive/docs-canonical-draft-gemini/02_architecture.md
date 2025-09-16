# LIMINAL Architecture: Unified Message Router Design

This document serves as the authoritative technical specification for LIMINAL's revolutionary architecture, centered around the **Unified Message Router** that enables human-like collaboration between AI agents.

## 1. Executive Summary

LIMINAL is a desktop application that orchestrates multiple AI coding agents through a centralized **Unified Message Router** – eliminating the human bottleneck in multi-agent development. The architecture enables agents to work like a human development team: claiming territories through soft leases, discussing asynchronously via clones, and reaching consensus through structured negotiation.

The system transforms the human role from a message bus into a director, overseeing an AI team that collaborates naturally through async communication patterns inspired by tools like Slack and Git.

## 2. Core Architectural Principles

*   **Unified Message Router as Central Nervous System:** All agent communication flows through a single, intelligent router that understands priority, context, and coordination patterns.
*   **Async-First Communication:** Agents never block waiting for responses; they queue messages and continue working.
*   **Territory-Based Coordination:** Soft, negotiable leases replace hard locks for resource management.
*   **Clone-Based Discussions:** Parallel conversation threads that don't interrupt primary work.
*   **Human as Director, Not Router:** The human guides strategy while the system handles all message routing.
*   **Local-First Architecture:** Everything runs on the developer's machine for speed and privacy.

## 3. Component Architecture

<!-- [[EDITOR_NOTE_CLAUDE: Insert the complete system architecture Mermaid diagram here.]] -->

### 3.1. Unified Message Router (Rust Core)

The heart of LIMINAL – a high-performance, asynchronous message routing engine that orchestrates all agent communication.

```rust
pub struct UnifiedMessageRouter {
    // Message queuing with priority levels
    message_queue: Arc<RwLock<PriorityQueue<Message>>>,

    // Territory management
    territory_manager: Arc<RwLock<TerritoryManager>>,

    // Clone orchestration
    clone_manager: CloneOrchestrator,

    // Interface adapters
    adapters: AdapterRegistry,

    // Persistence layer
    persistence: PersistenceLayer,

    // Configuration
    config: RouterConfig,
}
```

### 3.2. Context Snapshot Implementation

To enable efficient, low-latency clone discussions, LIMINAL implements a high-performance context snapshotting mechanism, heavily influenced by the `context_window_optimization.md` research report. The goal is to create and transfer a complete, isolated context to a new clone agent in **<10ms**.

#### Serialization Strategy

-   **Binary Serialization**: To achieve the required performance, context snapshots are serialized using a binary format. The chosen stack is **`serde` + `Bincode`**, which provides extremely fast and compact serialization compared to text-based formats like JSON.
-   **Zero-Copy Deserialization**: Where possible, the architecture leverages zero-copy deserialization techniques to allow agents to read snapshot data directly from the byte buffer without allocating new memory.

#### Snapshot Types: Differential vs. Full

The system dynamically chooses between two types of snapshots:

1.  **Full Snapshot**: A complete, self-contained snapshot of all relevant data.
2.  **Differential Snapshot**: A lightweight snapshot containing only the *changes* (diffs) relative to a base snapshot, used when changes are minimal.

#### Memory Management

-   **Shared Data with `Arc`**: Large, read-only data structures are wrapped in `Arc` (Atomically Referenced Counter). Instead of deep-copying, the `Arc` is cloned—a fast, atomic operation allowing multiple clones to share memory.
-   **Copy-on-Write**: Mutable data is cloned only when a clone agent needs to modify it, minimizing unnecessary data duplication.

### 3.3. Territory Manager & Lease Negotiation

The Territory Manager coordinates access to shared resources (files, directories) through a soft-lease system. It uses a 2D `TerritoryHash` (adapted from UNCAN’s 3D SpatialHash) for O(1) conflict detection.

#### Lease Negotiation Algorithm

When a lease conflict occurs, the manager evaluates the request based on priority, remaining lease duration, and queue length to make a `LeaseDecision`.

```rust
pub enum LeaseDecision {
    Grant(LeaseId),
    Deny(Reason),
    Defer(Duration),
    Escalate(HumanDirector),
}
```

-   **Grant**: If the resource is free or the request has a significantly higher priority.
-   **Defer**: If the current holder is performing a high-priority task, the request is queued.
-   **Deny**: If a low-priority agent requests a resource held by a high-priority agent.
-   **Escalate**: If multiple high-priority agents contend for the same resource, or a deadlock is detected, the decision is escalated to the Human Director.

<!-- [[EDITOR_NOTE_CLAUDE: Insert the detailed Mermaid flowchart for the lease negotiation decision tree here.]] -->

### 3.4. Agent Process Management

-   **PTY-Based Agent Control:** Each agent runs in a pseudo-terminal for robust lifecycle management.
-   **Structured Event Protocol:** A custom `<LIMINAL_EVENT>` protocol is parsed from the agent's stdout stream.

### 3.5. State Management (`Arc<RwLock>` Pattern)

Based on research and UNCAN's proven patterns, the system uses `Arc<RwLock<T>>` for shared state, allowing high-read parallelism with short-lived write locks. Persistence occurs outside of any lock to prevent blocking.

## 4. Communication Patterns

### 4.1. Async Clone Discussion Pattern

<!-- [[EDITOR_NOTE_CLAUDE: Insert the sequence diagram for the Async Clone Discussion Pattern here.]] -->

### 4.2. MCP as a Tool Interface

Instead of a communication protocol, MCP is used as a **tool exposure layer**. The router's capabilities (sending messages, acquiring leases, spawning clones) are exposed to agents as standard MCP tools. This abstracts the underlying communication mechanism.

```typescript
// Agents see router functions as tools
await mcp.use_tool("acquire_lease", { resource: "database/schema.sql", duration: 600 });
await mcp.use_tool("spawn_clone", { context: currentState, purpose: "discuss_api_design" });
```

## 5. Persistence Strategy

The system uses specialized databases for different data types (turns, messages, context, leases) and employs an event-sourcing model for a complete audit trail of all state changes.

---
*This architecture represents a fundamental shift from traditional agent orchestration to a collaborative, async-first system where agents work like human developers. The Unified Message Router is the key innovation that makes this possible.*