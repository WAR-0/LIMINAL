# LIMINAL Critical Assessment & Sprint Planning
**Date**: 2025-09-29
**Prepared by**: Systems Agent
**Purpose**: Eagle-eye analysis from current state → target state with actionable sprint plan

---

## Executive Summary

You've built **world-class distributed systems infrastructure** but are missing **the automation layer** that bridges your terminal-based workflow to the RTS oversight dashboard. The backend is over-engineered for your immediate needs (BFT consensus, ledger, spatial hashing) while under-serving your core workflow (no director agent, no runbook execution, no Claude Code integration).

**The Path Forward**: 3 distinct phases over 8-12 weeks:
1. **Phase 1 (BRIDGE)**: Connect terminal workflow to application (4 weeks)
2. **Phase 2 (AUTOMATE)**: Build director orchestration layer (4 weeks)
3. **Phase 3 (VISUALIZE)**: Integrate UNCAN RTS interface (4+ weeks)

---

## I. Three-Horizon Vision Map

### Horizon 1: Terminal-Based Workflow (TODAY)
**Reality**: Human director manually delegates runbook turns to multiple Claude Code CLI instances

```
┌─────────────────────────────────────────────────────┐
│ Human Director (Terminal 1: Claude Code CLI)       │
│  - Writes/reads runbooks (Markdown files)           │
│  - Copies Turn prompts manually                     │
│  - Pastes into other terminals                      │
└─────────────┬───────────────────────────────────────┘
              │ Manual Copy/Paste
         ┌────┴────┬─────────┬──────────┐
         ▼         ▼         ▼          ▼
    Terminal 2  Terminal 3 Terminal 4  Terminal 5
    Systems     Interface  Router      Testing
    Agent       Agent      Agent       Agent
```

**Pain Points**:
- 420+ context switches/hour
- Serial execution (one Turn at a time)
- No oversight into agent progress
- Manual artifact collection
- No coordination between specialists

### Horizon 2: Application-Based Orchestration (TARGET - 12 weeks)
**Goal**: Single desktop app with director automation and multi-agent oversight

```
┌─────────────────────────────────────────────────────┐
│ LIMINAL Desktop App (Tauri)                        │
│ ┌───────────────────────────────────────────────┐ │
│ │ Director Panel (Human + AI Director)          │ │
│ │  - Runbook editor with Turn delegation UI     │ │
│ │  - Plan approval interface                    │ │
│ │  - Escalation handling dashboard              │ │
│ └───────────────────────────────────────────────┘ │
│                                                     │
│ ┌──────────┬──────────┬──────────┬──────────┐    │
│ │ Systems  │Interface │ Router   │ Testing  │    │
│ │ Terminal │ Terminal │ Terminal │ Terminal │    │
│ │ (PTY)    │ (PTY)    │ (PTY)    │ (PTY)    │    │
│ └──────────┴──────────┴──────────┴──────────┘    │
│                                                     │
│ ┌──────────────────────────────────────────────┐  │
│ │ Unified Message Router (Backend)              │  │
│ │  - Priority queues, token buckets, aging      │  │
│ │  - Territory leasing, metrics, ledger         │  │
│ └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

**Benefits**:
- Automated runbook execution (parallel/sequential)
- Real-time progress monitoring
- Centralized artifact collection
- Cross-agent communication via router
- Human-in-the-loop at approval points

### Horizon 3: RTS Visual Swarm (ENDGAME - 6+ months)
**Vision**: UNCAN-style physics-driven visual interface

```
┌─────────────────────────────────────────────────────┐
│ LIMINAL RTS Interface (PixiJS + Rapier2D)          │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │ Territory Map (Filesystem as Treemap)       │  │
│  │  - Agents as physical entities (60fps)      │  │
│  │  - Visual lease boundaries with animations  │  │
│  │  - Conflict indicators and heat maps        │  │
│  │  - RTS controls: box select, control groups │  │
│  └─────────────────────────────────────────────┘  │
│                                                     │
│  ┌──────────────────────────────────────────┐     │
│  │ Message Router Visualization              │     │
│  │  - Live queue depths, flow animations     │     │
│  └──────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────┘
```

---

## II. Current State: Deep Dive

### A. What You've Built (Backend Infrastructure) ✅

#### 1. Production-Grade Router (`router.rs`)
- 5-level priority queues (Info, Coordinate, Blocking, Critical, DirectorOverride)
- Token bucket rate limiting with configurable refill rates
- Message aging to prevent starvation (boosts priority over time)
- Async dispatcher with <10ms target latency
- Metrics integration and ledger event emission

**Assessment**: ⭐⭐⭐⭐⭐ Solid, production-ready implementation

#### 2. Soft Territory Leasing (`territory.rs`)
- Spatial hashing for O(1) conflict detection
- Lease states: Available, Granted, InUse, Negotiating, Deferred, Overridden, Expired
- Priority-based conflict resolution with decision trees
- Heat map tracking (stigmergic coordination)
- Queue management with fairness guarantees

**Assessment**: ⭐⭐⭐⭐ Well-designed, needs real workload testing

#### 3. PTY Agent Management (`agent.rs`)
- Structured event protocol (`<FORGE_EVENT>` tags)
- Buffered parser for partial tag handling
- Non-blocking I/O with async channels
- AgentProcess spawning with resource limits

**Assessment**: ⭐⭐⭐⭐ Functional, ready for integration

#### 4. Metrics & Observability (`metrics.rs`)
- Performance tracking (routing latency, queue depths, token buckets)
- Lease lifecycle metrics (grants, deferrals, escalations)
- Agent events and health monitoring hooks
- Snapshot API for UI consumption

**Assessment**: ⭐⭐⭐⭐ Comprehensive, well-structured

#### 5. Experimental Modules (Feature-Gated)
- **Ledger**: Event sourcing, replay coordinator, checkpoint system
- **Consensus**: BFT quorum voting, ConsensusBroker
- **Health**: Threshold monitoring, alert generation

**Assessment**: ⭐⭐⭐ Scaffolded but not integrated; questionable necessity for v1

### B. What's Missing (Critical Gaps) ❌

#### 1. Director Agent (0% complete)
**Purpose**: AI agent that plans, delegates, and monitors specialist agents

**Required Components**:
```rust
// Missing: Director agent implementation
pub struct DirectorAgent {
    runbook_parser: RunbookParser,        // Parse MD → Task graph
    task_executor: TaskExecutor,          // Execute turns (parallel/sequential)
    delegation_manager: DelegationManager, // Assign turns to specialists
    escalation_handler: EscalationHandler, // Handle conflicts/errors
    progress_tracker: ProgressTracker,     // Monitor turn completion
}
```

**Current State**: Config defines director privileges (`liminal.config.yaml`) but no implementation

#### 2. Runbook Execution Engine (0% complete)
**Purpose**: Parse and execute Markdown runbooks as structured workflows

**Required Components**:
```rust
// Missing: Runbook data model and executor
pub struct Runbook {
    epoch_id: EpochId,
    goal: String,
    turns: Vec<Turn>,
    dependencies: HashMap<TurnId, Vec<TurnId>>,
}

pub struct Turn {
    id: TurnId,
    specialist_role: AgentRole, // Systems, Interface, Router, Testing
    prompt: String,
    acceptance_criteria: Vec<String>,
    parallel_group: Option<u32>,
}

pub struct RunbookExecutor {
    parser: MarkdownParser,    // MD → Runbook
    scheduler: TurnScheduler,  // Dependency resolution
    executor: TurnExecutor,    // Spawn specialist, await completion
}
```

**Current State**: Runbooks are static Markdown files in `/runbooks/` with manual copy-paste

#### 3. Claude Code CLI Integration (0% complete)
**Purpose**: Wrap Claude Code instances as managed specialist agents

**Required Components**:
```rust
// Missing: Claude Code adapter
pub struct ClaudeCodeAgent {
    pty: AgentProcess,              // Spawned PTY subprocess
    role: AgentRole,                // Systems, Interface, Router, Testing
    context_injector: ContextInjector, // Inject specialist directives
    artifact_collector: ArtifactCollector, // Capture outputs
}

impl ClaudeCodeAgent {
    pub fn spawn_specialist(
        role: AgentRole,
        turn_prompt: &str,
        context: &Context,
    ) -> Result<Self> {
        // 1. Spawn Claude Code CLI as PTY subprocess
        // 2. Inject role-specific directives (e.g., CLAUDE.md for Systems)
        // 3. Send Turn prompt via stdin
        // 4. Parse <FORGE_EVENT> outputs
        // 5. Collect artifacts on completion
    }
}
```

**Current State**: PTY infrastructure exists, but no Claude Code CLI binding

#### 4. Director Dashboard UI (10% complete)
**Purpose**: Human Director interface for runbook management and oversight

**Required Components**:
```typescript
// Missing: Director UI components
interface DirectorDashboard {
    RunbookEditor: React.FC;        // Create/edit runbooks
    TurnDelegator: React.FC;        // Assign turns to specialists
    AgentTerminals: React.FC;       // Multi-pane terminal view
    ProgressTracker: React.FC;      // Epoch/turn completion status
    EscalationHandler: React.FC;    // Approve overrides, resolve conflicts
    ArtifactViewer: React.FC;       // View agent outputs
}
```

**Current State**: Basic MVP UI exists (agent status, message log) but no director features

#### 5. Integrated Testing (30% complete)
**Purpose**: End-to-end scenarios proving the full workflow

**Current Tests**:
- ✅ Basic router dispatch
- ✅ Territory lease acquisition/release
- ✅ PTY event parsing
- ❌ Runbook execution
- ❌ Multi-agent coordination
- ❌ Director escalation flows
- ❌ Claude Code integration

---

## III. Architecture Critique: Over-Engineering vs Under-Delivery

### Over-Engineered Components (For Current Use Case)

#### 1. Byzantine Fault Tolerance & Consensus
**Why it exists**: `consensus.rs` implements BFT quorum voting for multi-node scenarios

**Reality**: Your use case is **single-machine, single-user** with multiple local Claude instances. There's no distributed trust problem. If a Claude instance crashes, restart it.

**Recommendation**: Remove or defer consensus module. Replace with simple "retry on failure" logic.

#### 2. Ledger with Replay & Event Sourcing
**Why it exists**: `ledger` module for deterministic replay and audit trails

**Reality**: Useful for debugging, but not critical path for v1. Your terminal workflow doesn't need replay—you can re-run the runbook.

**Recommendation**: Keep basic event logging, defer replay coordinator to Phase 3.

#### 3. Spatial Hashing for Territory Lookups
**Why it exists**: O(1) conflict detection using grid cells

**Reality**: You have 4-6 agents working on a small project. A `HashMap<ResourcePath, Lease>` is sufficient.

**Recommendation**: Keep for UNCAN integration (needed for RTS visualization), but overkill for Phase 1.

### Under-Engineered Components (For Immediate Needs)

#### 1. Runbook Parsing & Execution
**What's missing**: No data model or executor for Markdown runbooks

**Why it's critical**: This is the **core workflow** you want to automate. Without this, the app is just a fancy terminal multiplexer.

**Priority**: 🔥 **Highest**

#### 2. Claude Code Process Management
**What's missing**: No adapter to spawn/manage Claude Code instances

**Why it's critical**: Agents are the **workers**. Without Claude integration, you can't delegate turns.

**Priority**: 🔥 **Highest**

#### 3. Director Agent Logic
**What's missing**: No planning, delegation, or progress monitoring

**Why it's critical**: The **automation layer** that eliminates manual copy-paste.

**Priority**: 🔥 **High**

---

## IV. Sprint Plan: 3 Phases to RTS Oversight

### Phase 1: Terminal-to-App Bridge (4 weeks)
**Goal**: Replace manual terminal workflow with single-app execution

#### Sprint 1.1: Runbook Engine (1 week)
**Deliverables**:
- Markdown parser: `Runbook` data model with Turns, dependencies
- Task graph builder: detect parallel vs sequential turns
- Basic executor: spawn turns in order, wait for completion

**Test Scenario**: Parse `epoch_router_foundation.md`, execute Turns 1-2 sequentially

#### Sprint 1.2: Claude Code Integration (1 week)
**Deliverables**:
- `ClaudeCodeAgent`: spawn Claude as PTY subprocess
- Context injection: load specialist directives (`.uncan/systems/CLAUDE.md`)
- Prompt delivery: send Turn prompt via stdin
- Artifact capture: collect outputs to runbook results dir

**Test Scenario**: Spawn Systems agent, delegate Turn 1, capture design note output

#### Sprint 1.3: Director Agent MVP (1 week)
**Deliverables**:
- `DirectorAgent`: basic orchestrator that executes runbooks
- Turn scheduler: resolve dependencies, spawn agents
- Session state: track turn completion, artifacts
- Escalation stubs: log errors, await human resolution

**Test Scenario**: Execute full `epoch_router_foundation.md` (8 turns) with simulated agents

#### Sprint 1.4: Director Dashboard v1 (1 week)
**Deliverables**:
- Runbook selector: dropdown of available runbooks
- Turn progress: list view showing status (pending/in_progress/completed)
- Agent terminals: grid of 4 PTY terminals (Systems, Interface, Router, Testing)
- Start/Stop controls

**Test Scenario**: Load runbook, start execution, monitor agent terminals in real-time

**Phase 1 Exit Criteria**:
✅ Can load a runbook from filesystem
✅ Can execute all turns automatically
✅ Can view agent outputs in app
✅ Can collect artifacts to disk

---

### Phase 2: Director Orchestration & Collaboration (4 weeks)
**Goal**: Enable parallel execution, cross-agent messaging, and human escalation

#### Sprint 2.1: Parallel Turn Execution (1 week)
**Deliverables**:
- Parallel group support: spawn multiple agents simultaneously
- Dependency graph validation: prevent cycles
- Resource contention: use territory leases to prevent conflicts

**Test Scenario**: Execute turns with `parallel_group: 1`, verify simultaneous execution

#### Sprint 2.2: Cross-Agent Communication (1 week)
**Deliverables**:
- Route agent messages through UnifiedMessageRouter
- Parse `<FORGE_EVENT>` messages from Claude outputs
- Deliver messages to recipient agents via stdin
- Message log view in UI

**Test Scenario**: Agent A sends lease request to Agent B, B responds, router delivers

#### Sprint 2.3: Human Escalation Flow (1 week)
**Deliverables**:
- Escalation UI: modal dialog for approval/denial
- Pause execution: block turn until human decision
- Resolution injection: send human choice back to agent

**Test Scenario**: Agent encounters conflict, escalates to human, director resolves, agent continues

#### Sprint 2.4: Artifact & Context Management (1 week)
**Deliverables**:
- Artifact viewer: display agent outputs (code, docs, logs)
- Context snapshots: save/restore agent state between turns
- Session replay: reload previous runbook execution

**Test Scenario**: Complete runbook, review all artifacts, replay session

**Phase 2 Exit Criteria**:
✅ Can execute parallel turns without deadlocks
✅ Agents communicate via router
✅ Human can intervene at escalation points
✅ All artifacts saved and reviewable

---

### Phase 3: UNCAN RTS Visualization (4+ weeks)
**Goal**: Integrate visual swarm interface for territory oversight

#### Sprint 3.1: Territory Visualization (1 week)
**Deliverables**:
- Port UNCAN territory mapper (treemap layout)
- Render filesystem as spatial territories
- Color-code by lease status (available/in-use/contested)

**Test Scenario**: Load project, see files as territories with lease overlays

#### Sprint 3.2: Agent Physics & Rendering (1 week)
**Deliverables**:
- Port UNCAN physics engine (Rapier2D)
- Render agents as entities with movement
- Sync agent positions with lease acquisitions

**Test Scenario**: Agent moves to territory when lease granted

#### Sprint 3.3: RTS Controls (1 week)
**Deliverables**:
- Box selection, control groups (Ctrl+1-9)
- Right-click commanding (move, claim, release)
- Camera controls (WASD, zoom, pan)

**Test Scenario**: Select agents, command them to claim territories, observe lease negotiation

#### Sprint 3.4: Visual Effects & Polish (1 week)
**Deliverables**:
- Lease boundary animations (dashed borders, pulse effects)
- Conflict indicators (warning triangles)
- Message flow visualization (animated arcs between agents)
- Statistics overlay (queue depths, lease counts)

**Test Scenario**: Run full runbook in RTS view, observe all interactions visually

**Phase 3 Exit Criteria**:
✅ Can see filesystem as navigable territories
✅ Agents rendered as physical entities
✅ Can command agents with RTS controls
✅ Lease negotiations visible in real-time

---

## V. Immediate Next Steps (This Week)

### Critical Path: Runbook Engine Foundation

#### Task 1: Define Runbook Data Model
```rust
// Create: liminal-v1/src-tauri/src/director/runbook.rs
pub struct Runbook {
    pub id: String,
    pub epoch_id: String,
    pub goal: String,
    pub turns: Vec<Turn>,
}

pub struct Turn {
    pub id: u32,
    pub specialist: AgentRole,
    pub prompt: String,
    pub acceptance: Vec<String>,
    pub parallel_group: Option<u32>,
}

pub enum AgentRole {
    Systems,
    Interface,
    Router,
    Testing,
    Research,
}
```

#### Task 2: Implement Markdown Parser
```rust
// Create: liminal-v1/src-tauri/src/director/parser.rs
pub struct RunbookParser;

impl RunbookParser {
    pub fn parse(content: &str) -> Result<Runbook> {
        // Parse ## headers as turn boundaries
        // Extract ## Turn N — Role sections
        // Extract > block quotes as prompts
        // Parse **Acceptance** sections
    }
}
```

#### Task 3: Build Basic Executor
```rust
// Create: liminal-v1/src-tauri/src/director/executor.rs
pub struct RunbookExecutor {
    router: UnifiedMessageRouter,
    agents: HashMap<AgentRole, ClaudeCodeAgent>,
}

impl RunbookExecutor {
    pub async fn execute(&mut self, runbook: Runbook) -> Result<()> {
        for turn in runbook.turns {
            let agent = self.get_or_spawn_agent(turn.specialist).await?;
            agent.send_prompt(&turn.prompt).await?;
            agent.await_completion().await?;
        }
    }
}
```

#### Task 4: Create Director Module Structure
```bash
mkdir -p liminal-v1/src-tauri/src/director
touch liminal-v1/src-tauri/src/director/{mod.rs,runbook.rs,parser.rs,executor.rs,agent_manager.rs}
```

---

## VI. Decision Points & Trade-offs

### A. Simplification Decisions

#### Decision 1: Defer Consensus Module
**Rationale**: Single-machine use case doesn't need BFT. Failure = restart agent.
**Impact**: Remove `consensus.rs` dependency, simplify error handling
**Savings**: 20% less code complexity, faster development

#### Decision 2: Minimal Ledger for v1
**Rationale**: Event replay is nice-to-have, not critical path
**Impact**: Keep event logging, remove replay coordinator
**Savings**: 15% less code complexity

#### Decision 3: Spatial Hash Only for RTS
**Rationale**: HashMap is sufficient for 4-6 agents
**Impact**: Simplify territory manager for Phase 1-2, re-enable for Phase 3
**Savings**: Easier testing, clearer code

### B. Architecture Pivots

#### Pivot 1: Claude Code as Primary Agent Type
**From**: Generic "agent" abstraction
**To**: `ClaudeCodeAgent` with role specialization
**Why**: Your workflow is Claude-centric; don't over-abstract

#### Pivot 2: Runbook-First Execution
**From**: General-purpose task system
**To**: Markdown runbook executor
**Why**: You already have runbooks; leverage existing investment

#### Pivot 3: Human-in-Loop at Approval Points
**From**: Fully autonomous multi-agent
**To**: Director approval before plan execution
**Why**: Matches your actual workflow; humans direct, agents execute

---

## VII. Risk Assessment & Mitigation

### Risk 1: Claude Code Integration Complexity
**Probability**: Medium
**Impact**: High
**Mitigation**:
- Start with simple echo test (spawn Claude, send prompt, read output)
- Incremental: Add context injection, then artifact capture, then message routing
- Fallback: Manual terminal mode if PTY integration fails

### Risk 2: Runbook Parser Brittleness
**Probability**: High
**Impact**: Medium
**Mitigation**:
- Define strict Markdown schema for runbooks
- Add validation and helpful error messages
- Build parser test suite with all existing runbooks

### Risk 3: Performance Bottlenecks (Message Router)
**Probability**: Low
**Impact**: Medium
**Mitigation**:
- Already have <10ms routing target
- Load testing with synthetic messages
- Async dispatcher prevents blocking

### Risk 4: Scope Creep (UNCAN Integration Too Early)
**Probability**: High
**Impact**: High
**Mitigation**:
- **Strict phase gates**: Phase 3 only starts after Phase 2 exit criteria met
- RTS visualization is bonus, not blocker

---

## VIII. Success Metrics

### Phase 1 Metrics
- ✅ Runbook execution time: manual (30 min) → automated (<5 min)
- ✅ Context switches: 420/hr → 0/hr
- ✅ Agent spawn time: <2s
- ✅ Turn completion accuracy: 100% (all turns execute)

### Phase 2 Metrics
- ✅ Parallel speedup: 2-4x (depending on dependency graph)
- ✅ Escalation response time: <60s (human notified + responds)
- ✅ Message routing latency: <10ms p99
- ✅ Artifact collection: 100% (no lost outputs)

### Phase 3 Metrics
- ✅ UI framerate: 60fps steady
- ✅ Territory render time: <100ms for 500+ files
- ✅ RTS command latency: <50ms (click → agent response)
- ✅ Visual polish: smooth animations, no jank

---

## IX. Recommendations Summary

### A. Immediate Actions (Week 1)
1. ✅ Create `/director` module with data models
2. ✅ Implement Markdown→Runbook parser
3. ✅ Build basic sequential executor
4. ✅ Test with existing runbooks

### B. Phase 1 Focus (Weeks 1-4)
- **Priority 1**: Runbook engine + Claude Code integration
- **Priority 2**: Director agent MVP
- **Priority 3**: Basic UI for execution monitoring
- **Defer**: Ledger replay, consensus, spatial hashing optimizations

### C. Phase 2 Focus (Weeks 5-8)
- **Priority 1**: Parallel execution + cross-agent messaging
- **Priority 2**: Human escalation flows
- **Priority 3**: Artifact management
- **Defer**: Advanced UI features, visualizations

### D. Phase 3 Focus (Weeks 9-12+)
- **Priority 1**: Port UNCAN territory mapper
- **Priority 2**: Integrate physics engine
- **Priority 3**: RTS controls + visual polish

---

## X. Final Thoughts

You've built **exceptional infrastructure** (router, territory, metrics) but it's **solving the wrong problem first**. Your docs describe a distributed multi-agent platform, but your use case is **local orchestration of Claude Code instances**.

**The Good News**: All the hard stuff (async routing, PTY management, lease coordination) is done. The missing pieces are **high-level glue code**: runbook parsing, agent spawning, UI wiring.

**The Path to Success**:
1. **Phase 1**: Bridge terminal→app (prove value immediately)
2. **Phase 2**: Add collaboration features (parallel execution, messaging)
3. **Phase 3**: Polish with UNCAN visuals (delight factor)

**Key Insight**: Don't let UNCAN's visual appeal distract from Phase 1-2 fundamentals. The RTS interface is **icing on the cake**—but first you need to bake the cake (director orchestration).

---

## Appendix A: File Structure Changes

```
liminal-v1/src-tauri/src/
├── director/                 # NEW: Director agent module
│   ├── mod.rs
│   ├── runbook.rs           # Data models (Runbook, Turn, AgentRole)
│   ├── parser.rs            # Markdown → Runbook parser
│   ├── executor.rs          # Turn executor with dependency resolution
│   ├── agent_manager.rs     # Spawn/manage ClaudeCodeAgent instances
│   └── escalation.rs        # Human escalation handling
├── claude/                  # NEW: Claude Code integration
│   ├── mod.rs
│   ├── agent.rs             # ClaudeCodeAgent wrapper
│   ├── context.rs           # Context injection (directives, state)
│   └── artifacts.rs         # Output capture and storage
├── router.rs               # EXISTING: Keep as-is
├── territory.rs            # SIMPLIFY: Remove spatial hash for Phase 1
├── metrics.rs              # EXISTING: Keep as-is
├── agent.rs                # EXISTING: PTY base, keep as-is
├── consensus.rs            # REMOVE: Defer to Phase 3+
├── ledger.rs               # SIMPLIFY: Basic logging only for Phase 1
└── health.rs               # EXISTING: Keep as-is
```

## Appendix B: Runbook Schema Proposal

```markdown
# Runbook: [Epoch Name]

**Epoch Goal:** [One-sentence description]

## Turn 1 — [Agent Role]

**Specialist:** [Systems|Interface|Router|Testing|Research]
**Parallel Group:** [Optional: 1-N for parallel execution]

**Prompt to Delegate:**
> [Full Turn prompt here]
> [Can be multi-line]

**Acceptance:**
- [Criterion 1]
- [Criterion 2]

---

## Turn 2 — [Agent Role]
[Repeat structure]
```

**Parser Logic**:
- Each `## Turn N` header starts a new Turn
- Extract role from `— [Role]` suffix
- `**Prompt to Delegate:**` begins prompt block
- `>` quote lines are the prompt body
- `**Acceptance:**` begins acceptance criteria list