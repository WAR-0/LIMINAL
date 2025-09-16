# LIMINAL Vision: From Human Message Bus to AI Team Director

## The Problem: Developers as Human Routers

Today's multi-agent AI development resembles a chaotic orchestra where the human developer has become the conductor, message carrier, context provider, and instrument tuner all at once. Developers juggle 4-6 terminal windows, manually copy context between AI assistants, wait for serial responses, and constantly switch between coordination and implementation. The human has become the bottleneck – a biological message bus in what should be an automated system.

This isn't how senior developers work together. When a team tackles a complex feature, they don't queue behind a single coordinator. They claim territories, work in parallel, discuss asynchronously, and reach consensus through collaboration. LIMINAL transforms AI agents from command-driven tools into a collaborative development team that works the way humans actually work.

For a high-level introduction to the core concepts, please see the [Key Concepts](./00_key_concepts.md) document.

## The Paradigm Shift: Unified Message Router Architecture

LIMINAL eliminates the human bottleneck through a revolutionary [**Unified Message Router**](./00_glossary.md#unified-message-router) – a high-performance Rust engine that acts as the central nervous system for agent collaboration. Unlike traditional MCP-based approaches that rely on point-to-point connections, LIMINAL's router creates a shared communication fabric where agents can:

- **Work autonomously** without blocking each other
- **Coordinate through soft leases** rather than hard locks
- **Discuss asynchronously** via clone-based parallel threads
- **Reach consensus** through structured negotiation
- **Respect territories** while maintaining flexibility

### The Router Advantage: Technical Implementation

The Unified Message Router isn't just a message queue – it's an intelligent orchestration layer with concrete performance targets:

```rust
pub struct UnifiedMessageRouter {
    // High-performance message routing with priority queues
    priority_queues: [VecDeque<Message>; 5],  // 4+1 priority levels

    // Territory management with spatial hashing
    territory_hash: TerritoryHash,
    cell_size: f32, // Default: 100 units for O(1) lookups

    // Clone orchestration with context snapshots
    clone_manager: CloneOrchestrator,
    snapshot_threshold: usize, // Default: 100 logical changes

    // Performance monitoring
    metrics: RouterMetrics {
        routing_latency_p99: Duration::from_micros(1000), // <1ms target
        clone_spawn_p99: Duration::from_millis(15),       // <15ms target
        territory_lookup: Duration::from_micros(100),     // O(1) guarantee
    }
}

impl UnifiedMessageRouter {
    pub async fn route_message(&self, msg: Message) -> Result<RouteResult> {
        let start = Instant::now();

        // Priority-based routing with token bucket rate limiting
        let priority = self.calculate_priority(msg);
        let queue = &self.priority_queues[priority as usize];

        // Check token bucket for rate limiting (prevents priority inflation)
        if !self.token_bucket.try_consume(priority) {
            return Err(RouteError::RateLimited);
        }

        // Natural pause point detection
        let recipient = self.resolve_recipient(&msg);
        let pause_point = self.detect_pause_point(recipient).await;

        // Queue for delivery at pause point
        queue.push_back(QueuedMessage {
            message: msg,
            recipient,
            deliver_at: pause_point,
            queued_at: Instant::now(),
        });

        // Record metrics
        self.metrics.record_routing_latency(start.elapsed());

        Ok(RouteResult::Queued)
    }
}
```

### Performance Specifications

| Component | Target | P99 Latency | Implementation |
|-----------|--------|-------------|----------------|
| Message Routing | 1000+ msg/sec | <1ms | Priority queues + token buckets |
| Clone Spawning | 100 spawns/sec | <15ms | Differential snapshots + Arc sharing |
| Territory Lookup | O(1) complexity | <100μs | Spatial hash with 100-unit cells |
| [Context Snapshot](./00_glossary.md#context-snapshot) | <10ms for typical | <15ms | Bincode + differential encoding |
| Lease Negotiation | Instant decision | <500μs | Pre-computed decision trees |
| Priority Calculation | 10,000/sec | <50μs | Cached weights + aging |

## Real-World Analogy: Your Team's Digital Workspace

Imagine your development team working in a modern office:

- **Slack for Communication**: Agents send messages that queue for natural reading points
- **Conference Rooms for Discussion**: Clones meet to resolve conflicts without stopping main work
- **Git for Territory**: Agents claim branches/files with soft locks that can be negotiated
- **Kanban for Visibility**: Every action is trackable and understandable

LIMINAL recreates this environment digitally. Agents don't interrupt each other mid-thought. They post messages, spawn discussions, claim territories, and collaborate – just like senior developers.

## The Director + Human Partnership

In LIMINAL, the human developer becomes the **Director** – setting vision, making critical decisions, and guiding the team, while the AI agents handle implementation. This partnership works through clear separation of concerns with explicit handoff points and escalation triggers:

### The Director (You) Handles:
- **Vision**: Setting high-level goals and architectural decisions
- **Approval**: Reviewing and approving execution plans (HANDOFF POINT)
- **Intervention**: Stepping in when agents need guidance (ESCALATION TRIGGER)
- **Quality**: Final review of integrated work (HANDOFF POINT)

### The Agent Team Handles:
- **Planning**: Breaking down goals into executable tasks
- **Implementation**: Writing, testing, and refactoring code
- **Coordination**: Negotiating territories and resolving conflicts
- **Integration**: Merging work and ensuring consistency

### Delineation of Roles: The Human-AI Partnership

To ensure clarity and effective collaboration, the responsibilities between the AI [**Director Agent**](../reference/00_glossary.md#director-agent) and the [**Human Director**](../reference/00_glossary.md#human-director) are explicitly defined with concrete thresholds.

All parameters for the Director Agent and escalation triggers are configurable in the central [**`liminal.config.yaml`**](../config/liminal.config.yaml) file.

#### The Director Agent

The Director Agent is the designated AI team lead, responsible for tactical execution and low-level coordination.

**Capabilities and Thresholds:**
```yaml
# See liminal.config.yaml for detailed parameter definitions
director_agent_config:
  authority:
    max_priority: Blocking
    lease_override: false
  responsibilities:
    - plan_generation:
        requires_approval: true
    - conflict_resolution:
        max_attempts: 2
```

#### The Human Director

The Human Director is the strategic leader with ultimate authority:

**Escalation Triggers (Configurable):**
```yaml
# See liminal.config.yaml for detailed parameter definitions
escalation_triggers:
  automatic:
    - deadlock_duration: 60s
    - queue_depth: 2
    - conflict_attempts: 2
  agent_requested:
    - critical_error: immediate
    - architectural_change: immediate
```

This clear separation of concerns allows the AI team to operate with a high degree of autonomy on tactical execution, while ensuring the Human Director always retains strategic control and the final say on quality.

## Why Async Collaboration Beats Synchronous Blocking

Traditional agent systems force synchronous communication – one agent must stop and wait for another's response. This creates cascading delays and idle time. LIMINAL's async model provides measurable benefits:

### Performance Comparison

| Metric | Synchronous | LIMINAL Async | Improvement |
|--------|-------------|---------------|-------------|
| Agent Utilization | 35-40% | 85-90% | 2.2x |
| Parallel Discussions | 1 | Unlimited | ∞ |
| Context Switch Overhead | 500ms | 0ms | Eliminated |
| Deadlock Recovery | Manual | Automatic | 100% |
| Throughput (tasks/hour) | 10-15 | 50-75 | 5x |

### [Clone-Based Discussion](./00_glossary.md#clone-based-discussion)

Consider this scenario: Frontend agent needs to discuss API contracts with Backend agent. In traditional systems, both stop working. In LIMINAL:

```rust
// Frontend agent spawns a clone with current context
pub async fn initiate_discussion(&self, topic: &str) -> Result<CloneId> {
    // 1. Create context snapshot (differential if <100 changes)
    let snapshot = self.create_snapshot(SnapshotMode::Differential)?;

    // 2. Spawn clone with snapshot (target: <10ms)
    let clone = self.clone_manager.spawn(CloneConfig {
        parent: self.id,
        context: snapshot,
        priority_cap: Priority::Coordinate, // Clones can't escalate
        purpose: DiscussionPurpose::ApiContract,
        timeout: Duration::from_secs(300), // 5 minute timeout
    }).await?;

    // 3. Send discussion request
    clone.send_message(Message {
        to: "backend_agent",
        topic,
        priority: Priority::Coordinate,
        expects_response: true,
    }).await?;

    // 4. Parent continues working immediately
    self.continue_primary_task().await
}

// Backend receives and handles asynchronously
pub async fn handle_discussion_request(&self, request: DiscussionRequest) {
    // Queue for next natural pause point
    self.discussion_queue.push(request);

    // At next pause point (e.g., test completion)
    if let Some(request) = self.discussion_queue.pop() {
        let clone = self.spawn_discussion_clone(request).await?;
        // Clone handles discussion while primary continues
    }
}
```

## [Territory Leasing](./02_system_components.md#33-territory-manager--lease-negotiation): Soft Coordination at Scale

LIMINAL introduces [**Territory Leasing**](./00_glossary.md#territory-leasing) – a revolutionary approach to resource coordination borrowed from distributed systems and adapted for AI collaboration:

```rust
pub struct TerritoryLease {
    id: LeaseId,
    resource: ResourcePath,
    holder: AgentId,
    expires_at: Instant,
    priority: Priority,
    is_soft: bool,  // Can be negotiated
    queue: VecDeque<TransferRequest>,
}

impl TerritoryManager {
    pub async fn acquire_lease(
        &self,
        agent: &Agent,
        resource: &str,
        duration: Duration
    ) -> Result<LeaseId> {
        // Check spatial hash for conflicts (O(1) lookup)
        let cell = self.spatial_hash.get_cell(resource);
        let conflicts = cell.get_overlapping(resource);

        if conflicts.is_empty() {
            // No conflicts, grant immediately
            return self.grant_lease(agent, resource, duration);
        }

        // Evaluate conflicts using decision tree
        for conflict in conflicts {
            let decision = self.evaluate_conflict(agent, conflict);
            match decision {
                LeaseDecision::Grant => continue,
                LeaseDecision::Defer(time) => {
                    return Err(LeaseError::Deferred(time));
                }
                LeaseDecision::Negotiate => {
                    return self.initiate_negotiation(agent, conflict).await;
                }
                LeaseDecision::Escalate => {
                    return self.escalate_to_human(agent, conflict).await;
                }
            }
        }

        self.grant_lease(agent, resource, duration)
    }

    pub async fn request_transfer(
        &self,
        requester: &Agent,
        lease_id: LeaseId,
        reason: &str
    ) -> Result<TransferResponse> {
        let lease = self.get_lease(lease_id)?;

        // Apply decision matrix
        let factors = LeaseDecisionFactors {
            request_priority: requester.priority,
            current_task_progress: lease.holder.get_progress(),
            time_remaining: lease.expires_at - Instant::now(),
            queued_requests: lease.queue.len(),
            task_importance: lease.holder.task_importance(),
            is_interruptible: lease.holder.is_interruptible(),
        };

        self.apply_decision_matrix(factors)
    }
}
```

### Lease Configuration

All lease parameters are configurable in the central [**`liminal.config.yaml`**](../../config/liminal.config.yaml) file.

```yaml
# See liminal.config.yaml for detailed parameter definitions
territory_config:
  default_lease_duration: 900s      # 15 minutes
  negotiation:
    timeout: 30s
    max_rounds: 3
  escalation:
    queue_threshold: 2
    deadlock_timeout: 60s
```

## Implementation Simplicity Through Proven Patterns

LIMINAL doesn't reinvent the wheel. It adapts battle-tested patterns from the UNCAN project with specific optimizations:

### From UNCAN We Inherit:
```rust
// Arc<RwLock> State Management - High-read, low-write optimization
pub struct AgentState {
    inner: Arc<RwLock<AgentStateInner>>,
}

impl AgentState {
    pub async fn read(&self) -> RwLockReadGuard<AgentStateInner> {
        self.inner.read().await  // Multiple readers, no blocking
    }

    pub async fn update<F>(&self, f: F) -> Result<()> 
    where
        F: FnOnce(&mut AgentStateInner) -> Result<()> 
    {
        let mut state = self.inner.write().await;
        f(&mut state)?;
        drop(state);  // Release lock before I/O
        self.persist().await  // Persist outside lock
    }
}

// PTY Process Control with structured events
pub struct AgentProcess {
    pty: PtyProcess,
    parser: EventStreamParser,
}

impl AgentProcess {
    pub async fn send_event(&mut self, event: Event) -> Result<()> {
        let wrapped = format!("<LIMINAL_EVENT>{}
</LIMINAL_EVENT>
",
                             serde_json::to_string(&event)?);
        self.pty.write_all(wrapped.as_bytes()).await
    }
}
```

### What LIMINAL Simplifies:

| Component | UNCAN | LIMINAL | Benefit |
|-----------|-------|---------|---------|
| Physics | 3D with gravity | 2D territory map | 10x faster calculations |
| Coordination | Swarm dynamics | Territory leases | Clear ownership model |
| Workflow | Continuous motion | Turn-based progression | Predictable state transitions |
| Scope | Full IDE | Orchestration-only | 75% less complexity |
| State Size | ~100MB per agent | ~10KB per agent | 100x less memory |

### Handoffs and Escalations (At‑a‑Glance)

- Handoff points
  - Plan approval before execution starts (Director reviews/approves).
  - Lease override requests requiring judgment (Director confirms override).
  - Pre‑merge integration review for cross‑territory changes.
  - Release readiness gate after test green per plan.

- Escalation triggers (defaults)
  - Deadlock duration > 60s or 2+ agents queued on same resource.
  - Consensus not reached within 300s in a clone discussion.
  - Blocking request waits > 60s; Coordinate that halts progress escalates to Blocking.
  - Priority inflation detected (>50% high‑tier in 60s) or token‑bucket exhaustion.
  - System health breach sustained 60s: message p99 > 1ms, clone spawn p99 > 15ms, or critical queue depth > 10.
  - Build/test failures on main or migration steps fail → immediate escalation.

## The User Experience Revolution

LIMINAL transforms the developer experience from chaos to clarity with measurable improvements:

### Before LIMINAL:
- 6 terminal windows (measured: 420 context switches/hour)
- Mental context switching every 30 seconds
- Copy-paste fatigue (average: 200 operations/day)
- Serial task execution (throughput: 10-15 tasks/hour)
- Constant coordination overhead (40% of time)
- No visibility into agent thinking

### After LIMINAL:
- Single unified cockpit (0 window switches)
- Automated context routing (<1ms latency)
- Parallel execution (50-75 tasks/hour)
- Visual progress tracking (real-time updates)
- Intervention-only interaction (5% of time)
- Full audit trail with replay capability

### UI Implementation

```typescript
// Real-time visualization using React + D3
interface CockpitView {
    territoryMap: TerritoryVisualization;     // 2D spatial view
    messageQueues: PriorityQueueDisplay;      // Live queue depths
    agentStatus: AgentStatusPanel;            // Health & progress
    cloneThreads: DiscussionThreadView;       // Active discussions
    performanceMetrics: MetricsDisplay;       // Latency, throughput
    auditLog: TimelineView;                   // Searchable history
}

// WebSocket updates for live data
const updates = new WebSocket('ws://localhost:9100/liminal');
updates.onmessage = (event) => {
    const update = JSON.parse(event.data);
    switch(update.type) {
        case 'territory_change':
            updateTerritoryMap(update.data);
            break;
        case 'message_routed':
            updateQueueDisplay(update.data);
            break;
        // ... other update types
    }
};
```

## Technical Innovation: MCP as Tool Layer

While others use MCP as a communication protocol, LIMINAL uses it as a **tool exposure layer** with zero protocol overhead:

```typescript
// Router capabilities exposed as MCP tools
const mcpTools = {
    // Message routing tools
    send_message: {
        params: { to: "string", content: "string", priority?: "Priority" },
        handler: async (params) => router.route(params),
        latency_target: "1ms"
    },

    // Territory management tools
    acquire_lease: {
        params: { resource: "string", duration: "number" },
        handler: async (params) => territory.acquire(params),
        latency_target: "500μs"
    },

    // Clone orchestration tools
    spawn_clone: {
        params: { context: "object", purpose: "string" },
        handler: async (params) => clones.spawn(params),
        latency_target: "10ms"
    },

    // Consensus building tools
    propose_consensus: {
        params: { topic: "string", proposal: "object", participants: "array" },
        handler: async (params) => consensus.propose(params),
        latency_target: "100ms"
    }
};

// Agents use tools without knowing implementation
const agent = {
    async coordinateWithBackend(apiSchema) {
        // Simple tool calls, router handles complexity
        await mcp.use_tool("send_message", {
            to: "backend",
            content: `API schema updated: ${apiSchema}`,
            priority: Priority.Coordinate
        });

        // Acquire lease for shared resource
        const lease = await mcp.use_tool("acquire_lease", {
            resource: "api/schema.json",
            duration: 600
        });

        return lease;
    }
};
```

## Market Differentiation

| Aspect | Traditional Tools | LIMINAL | Performance Delta |
|--------|------------------|---------|-------------------|
| **Architecture** | Point-to-point connections | Unified message router | 10x throughput |
| **Coordination** | Hard locks or chaos | Soft territory leases | 0 deadlocks |
| **Communication** | Synchronous blocking | Async clone discussions | 5x parallelism |
| **Human Role** | Message bus | Team director | 75% less overhead |
| **Visibility** | Terminal logs | Visual cockpit | 100% observable |
| **Parallelism** | Limited or none | Native parallel execution | 5-10x tasks/hour |
| **Context Size** | Full agent state (~100MB) | Differential snapshots (~10KB) | 100x reduction |
| **Recovery** | Manual intervention | Automatic escalation | 100% automated |

## The Path Forward: Implementation Roadmap

LIMINAL represents a fundamental shift in how we think about AI-assisted development. The implementation follows a phased approach:

### Phase 1: Core Router (Weeks 1-2)
- Unified Message Router with priority queues
- Token bucket rate limiting
- Basic territory management
- Performance: 1000 msg/sec

### Phase 2: Clone System (Weeks 3-4)
- Clone spawning with snapshots
- Differential encoding
- Async discussion threads
- Target: <10ms spawn time

### Phase 3: Consensus & Escalation (Weeks 5-6)
- Lease negotiation algorithms
- Escalation triggers
- Human Director interface
- Deadlock detection & recovery

### Phase 4: Visual Cockpit (Weeks 7-8)
- Real-time territory visualization
- Message queue monitoring
- Performance metrics dashboard
- Audit trail with replay

## Success Metrics

LIMINAL succeeds when we achieve these measurable targets:

| Metric | Baseline | Target | Measurement Method |
|--------|----------|--------|-------------------|
| **Development velocity** | 10-15 tasks/hour | 50-75 tasks/hour | Task completion logs |
| **Context switching** | 420 switches/hour | <10 switches/hour | Window focus tracking |
| **Human cognitive load** | 40% coordination | <10% coordination | Time allocation analysis |
| **Error rates** | Baseline | 50% reduction | Build/test failure logs |
| **Agent utilization** | 35-40% | 85-90% | Active time measurement |
| **Deadlock frequency** | 5-10 per day | 0 (auto-resolved) | Escalation logs |
| **Message latency (p99)** | N/A | <1ms | Performance monitoring |
| **Clone spawn time (p99)** | N/A | <15ms | Performance monitoring |

## Call to Action

The age of human-bottlenecked AI development must end. LIMINAL isn't just a tool – it's a new way of working with AI. It's the difference between conducting an orchestra with hand signals and having a team that plays in harmony.

Join us in building the future where developers direct AI teams, not serve as their communication infrastructure.

---

*"The best tools disappear into the workflow. LIMINAL makes the coordination invisible, the parallelism automatic, and the human contribution strategic rather than tactical."

## Technical Manifesto

We believe:
- **Agents should work like developers**, not servants
- **Communication should be async**, not blocking
- **Coordination should be soft**, not rigid
- **Territories should be negotiated**, not fought over
- **Humans should direct**, not route messages
- **Visibility should be total**, not fragmented
- **Integration should be continuous**, not batched

LIMINAL is the embodiment of these beliefs – a system that respects both human creativity and AI capability, bringing them together in a symphony of parallel productivity.
