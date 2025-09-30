# Runbook: Phase 1 - Director Foundation & Claude Integration

**Epoch Goal:** Build the director orchestration layer that automates runbook execution with Claude Code CLI agents, eliminating manual terminal copy-paste workflow.

**Duration Estimate:** 4 weeks
**Parallel Capacity:** 3 agents max (comfortable), 6 agents max (peak)
**Current Session:** Late evening, laptop screen, 3 agents recommended

---

## Turn 1 — Systems Agent

**Specialist:** Systems
**Parallel Group:** N/A (Sequential - Foundation)
**Dependencies:** None

**Prompt to Delegate:**
> Build the runbook data model and Markdown parser that will serve as the foundation for automated runbook execution.
>
> **Context**: We're replacing manual copy-paste terminal workflow with automated orchestration. Runbooks are currently Markdown files in `/runbooks/` with a specific structure (see below). Your job is to create Rust data structures and a parser that can read these files and convert them into executable task graphs.
>
> **Tasks**:
> 1. Create `/liminal-v1/src-tauri/src/director/` module structure
> 2. Define data models in `director/runbook.rs`:
>    - `Runbook` (epoch_id, goal, turns, metadata)
>    - `Turn` (id, specialist role, prompt, acceptance criteria, parallel_group)
>    - `AgentRole` enum (Systems, Interface, Router, Testing, Research, Director)
>    - `TurnStatus` enum (Pending, InProgress, Completed, Failed, Blocked)
> 3. Implement Markdown parser in `director/parser.rs`:
>    - Parse `## Turn N — [Role]` headers
>    - Extract specialist from `**Specialist:** [Role]` line
>    - Extract parallel group from `**Parallel Group:** [N]` line
>    - Parse `**Prompt to Delegate:**` block quote sections (multi-line `>` quotes)
>    - Parse `**Acceptance:**` bulleted lists
>    - Build dependency graph from parallel groups (same group = parallel, different = sequential)
> 4. Add `director` module to `lib.rs` and `main.rs`
> 5. Write unit tests for parser with sample runbook snippets
>
> **Runbook Markdown Format**:
> ```markdown
> # Runbook: [Epoch Name]
> **Epoch Goal:** [Description]
>
> ## Turn 1 — Systems Agent
> **Specialist:** Systems
> **Parallel Group:** N/A (Sequential)
> **Dependencies:** None
>
> **Prompt to Delegate:**
> > [Multi-line prompt here]
> > [Can span multiple lines]
>
> **Acceptance:**
> - [Criterion 1]
> - [Criterion 2]
> ```
>
> **Implementation Notes**:
> - Use `pulldown-cmark` or similar for Markdown parsing (add to Cargo.toml if needed)
> - Keep data structures serializable (derive `Serialize`, `Deserialize`)
> - Parser should validate structure and return helpful errors
> - Dependency graph: Turns in same parallel group can run concurrently; sequential turns have implicit dependencies on all prior turns
>
> **Files to Create**:
> - `liminal-v1/src-tauri/src/director/mod.rs`
> - `liminal-v1/src-tauri/src/director/runbook.rs`
> - `liminal-v1/src-tauri/src/director/parser.rs`
> - `.uncan/systems/runbooks/impl_runbook_parser.md` (design doc)

**Acceptance:**
- ✅ Data models defined with clear documentation
- ✅ Markdown parser successfully parses this runbook file
- ✅ Unit tests pass for parser edge cases (missing fields, malformed structure)
- ✅ Can construct dependency graph from parallel groups
- ✅ `cargo fmt`, `cargo clippy`, `cargo test` all pass

---

## Turn 2 — Systems Agent

**Specialist:** Systems
**Parallel Group:** N/A (Sequential)
**Dependencies:** Turn 1

**Prompt to Delegate:**
> Build the `ClaudeCodeAgent` wrapper that spawns and manages Claude Code CLI instances as PTY subprocesses.
>
> **Context**: We need to programmatically spawn Claude Code CLI agents with role-specific context (e.g., Systems, Interface). Each agent runs as a PTY subprocess, receives prompts via stdin, and emits structured outputs via the existing `<FORGE_EVENT>` protocol. Your job is to create a high-level wrapper that integrates with the existing `agent.rs` PTY infrastructure.
>
> **Claude Code CLI Command**:
> ```bash
> claude --dangerously-skip-permissions --verbose
> ```
>
> **Tasks**:
> 1. Create `director/claude_agent.rs` module
> 2. Implement `ClaudeCodeAgent` struct:
>    ```rust
>    pub struct ClaudeCodeAgent {
>        role: AgentRole,
>        pty_process: AgentProcess, // Reuse existing PTY from agent.rs
>        status: AgentStatus,
>        current_turn: Option<Turn>,
>        artifacts: Vec<PathBuf>,
>    }
>    ```
> 3. Implement spawning logic:
>    - Spawn `claude --dangerously-skip-permissions --verbose` as PTY subprocess
>    - Set working directory to project root
>    - Note: CLAUDE.md files are automatically read by Claude Code from directory hierarchy
> 4. Implement prompt delivery:
>    - `send_turn_prompt(&mut self, turn: &Turn) -> Result<()>`
>    - Format prompt with clear start/end markers
>    - Include turn ID, role, acceptance criteria in prompt wrapper
> 5. Implement completion detection:
>    - Parse `<FORGE_EVENT>` outputs for completion signals
>    - Monitor for explicit "Turn complete" message or timeout (30 min default)
>    - Capture stdout/stderr as artifacts
> 6. Implement artifact collection:
>    - Track files modified during turn (git status integration)
>    - Save terminal output to `.uncan/[role]/context/turn_[N]_output.log`
> 7. Add cleanup and shutdown:
>    - Graceful SIGTERM with 10s timeout
>    - Force SIGKILL if unresponsive
>
> **Integration Points**:
> - Use existing `AgentProcess` from `agent.rs` for PTY management
> - Emit `AgentEvent` for lifecycle (spawned, working, completed, failed)
> - Hook into existing `<FORGE_EVENT>` parser
>
> **Testing**:
> - Spawn a test Claude instance, send simple prompt, verify output capture
> - Test timeout and cleanup scenarios
>
> **Files to Create**:
> - `liminal-v1/src-tauri/src/director/claude_agent.rs`
> - `.uncan/systems/runbooks/impl_claude_wrapper.md` (design doc)

**Acceptance:**
- ✅ Can spawn Claude Code CLI as PTY subprocess
- ✅ Can send prompts and receive outputs via stdin/stdout
- ✅ Completion detection works (explicit signal or timeout)
- ✅ Artifacts captured to correct locations
- ✅ Graceful shutdown with cleanup
- ✅ Integration test demonstrates full lifecycle
- ✅ `cargo fmt`, `cargo clippy`, `cargo test` pass

---

## Turn 3 — Interface Agent

**Specialist:** Interface
**Parallel Group:** 1 (Parallel with Turns 4, 5)
**Dependencies:** None (UI foundation work)

**Prompt to Delegate:**
> Extract and adapt the UI theme, color palette, and core components from the UNCAN project for use in LIMINAL.
>
> **Context**: The UNCAN project (`/Users/grey/War/projects/uncan`) has a polished React + PixiJS interface with excellent color schemes, menus, and visual effects. We want to reuse these design elements as the foundation for LIMINAL's director dashboard. Your job is to create a theme system and component library that captures UNCAN's visual language.
>
> **Tasks**:
> 1. Analyze UNCAN UI implementation:
>    - Review `/Users/grey/War/projects/uncan/src/ui/` components
>    - Extract color palette from existing styles
>    - Identify reusable patterns (buttons, panels, status indicators)
> 2. Create LIMINAL theme system in `liminal-v1/src/`:
>    - `src/theme/colors.ts` - Color constants from UNCAN
>    - `src/theme/styles.ts` - Base styles (fonts, spacing, shadows)
>    - `src/theme/index.ts` - Theme provider/context
> 3. Port key components:
>    - `src/components/Panel.tsx` - Draggable/resizable panels
>    - `src/components/StatusBadge.tsx` - Agent status indicators
>    - `src/components/Button.tsx` - Primary/secondary button styles
>    - `src/components/Terminal.tsx` - Terminal-style output view
> 4. Consider shadcn/ui + Tailwind:
>    - Evaluate if shadcn components fit the aesthetic
>    - If yes, install and configure with UNCAN color palette
>    - If no, pure CSS/styled-components with UNCAN styles
> 5. Create UI playground/storybook:
>    - `src/playground/ComponentShowcase.tsx`
>    - Demo all theme colors and components
>    - Accessible via dev route `/playground`
> 6. Document design system:
>    - `.uncan/interface/runbooks/design_system.md`
>    - Color usage guidelines
>    - Component usage examples
>
> **UNCAN References**:
> - Colors: Look for agent role colors (Explorer=Cyan, Builder=Green, etc.)
> - Effects: Pulse animations, glow effects, border styles
> - Layout: Panel systems, grid arrangements
>
> **Design Philosophy**:
> - High contrast for readability
> - Subtle animations for state changes
> - Terminal/code aesthetic (monospace, dark theme)
> - Status colors: Green=success, Yellow=warning, Red=error, Blue=info
>
> **Files to Create**:
> - `liminal-v1/src/theme/` directory with theme system
> - `liminal-v1/src/components/` directory with base components
> - `liminal-v1/src/playground/ComponentShowcase.tsx`
> - `.uncan/interface/runbooks/design_system.md`

**Acceptance:**
- ✅ Theme system captures UNCAN's visual style
- ✅ At least 5 reusable components ported/created
- ✅ Component playground accessible and functional
- ✅ Design system documented with examples
- ✅ Colors and styles feel cohesive with UNCAN
- ✅ `npm run lint` passes

---

## Turn 4 — Router Agent

**Specialist:** Router
**Parallel Group:** 1 (Parallel with Turns 3, 5)
**Dependencies:** None (simplification work)

**Prompt to Delegate:**
> Simplify the Territory Manager for Phase 1 by removing spatial hashing and deferring complex features to Phase 3.
>
> **Context**: The current `territory.rs` implementation includes spatial hashing (O(1) conflict detection) and stigmergic heat maps designed for hundreds of agents. For Phase 1, we only have 4-6 local Claude instances. A simple `HashMap<ResourcePath, Lease>` is sufficient. We'll re-enable spatial hashing in Phase 3 when integrating UNCAN's RTS visualization.
>
> **Tasks**:
> 1. Review current `territory.rs` implementation
> 2. Create simplified version:
>    - Remove `SpatialHash` struct and related logic
>    - Replace with `HashMap<ResourcePath, Lease>` for lease tracking
>    - Keep core lease states (Available, Granted, InUse, Negotiating, Deferred, Overridden, Expired)
>    - Keep priority-based conflict resolution (decision matrix)
>    - Keep heat map tracking (defer visualization to Phase 3)
> 3. Update `TerritoryManager` API:
>    - `acquire_lease()` - Check HashMap for conflicts
>    - `release_lease()` - Remove from HashMap
>    - `check_conflicts()` - Iterate HashMap (O(n) is fine for 4-6 agents)
>    - Keep existing lease negotiation flows
> 4. Add feature gate for spatial hashing:
>    - `#[cfg(feature = "spatial-hash")]` for complex code
>    - Document that spatial hash will be re-enabled for Phase 3
> 5. Update tests:
>    - Ensure existing tests still pass with simplified implementation
>    - Add comment noting spatial hash tested separately under feature flag
> 6. Document simplification rationale:
>    - Write design note explaining Phase 1 vs Phase 3 tradeoffs
>    - Note performance characteristics (HashMap O(n) fine for <10 agents)
>
> **Files to Modify**:
> - `liminal-v1/src-tauri/src/territory.rs`
> - `liminal-v1/src-tauri/Cargo.toml` (add spatial-hash feature flag)
> - `.uncan/router/runbooks/territory_simplification.md` (design doc)

**Acceptance:**
- ✅ Spatial hash logic removed from main code path
- ✅ HashMap-based implementation functional
- ✅ All existing tests pass
- ✅ Feature flag `spatial-hash` defined for future use
- ✅ Performance remains <10ms for lease operations
- ✅ Design doc explains simplification rationale
- ✅ `cargo fmt`, `cargo clippy`, `cargo test` pass

---

## Turn 5 — Systems Agent

**Specialist:** Systems
**Parallel Group:** 1 (Parallel with Turns 3, 4 - but has dependencies, start after Turn 2 complete)
**Dependencies:** Turns 1, 2

**Prompt to Delegate:**
> Build the `DirectorAgent` orchestrator that manages runbook execution, delegates turns to Claude agents, and handles session state.
>
> **Context**: This is the core orchestration logic. The DirectorAgent reads a runbook, spawns specialist agents, delegates turn prompts, monitors progress, and handles escalations. It's the brain of the automation system.
>
> **Tasks**:
> 1. Create `director/orchestrator.rs` module
> 2. Implement `DirectorAgent` struct:
>    ```rust
>    pub struct DirectorAgent {
>        current_runbook: Option<Runbook>,
>        agents: HashMap<AgentRole, ClaudeCodeAgent>,
>        turn_status: HashMap<TurnId, TurnStatus>,
>        session_id: String,
>        metrics: MetricsCollector,
>        router: UnifiedMessageRouter,
>    }
>    ```
> 3. Implement core methods:
>    - `load_runbook(path: &Path) -> Result<()>` - Parse and validate runbook
>    - `start_execution() -> Result<()>` - Begin turn sequence
>    - `delegate_turn(turn: &Turn) -> Result<()>` - Spawn agent and send prompt
>    - `monitor_progress() -> Vec<TurnUpdate>` - Poll agent status
>    - `handle_completion(turn_id: TurnId) -> Result<()>` - Process turn results
>    - `handle_escalation(escalation: Escalation) -> Result<()>` - Stub for Phase 2
> 4. Implement dependency resolution:
>    - Detect parallel groups
>    - Spawn parallel turns simultaneously (max 3 default, configurable)
>    - Wait for all turns in a parallel group to complete before proceeding
>    - Sequential turns wait for all prior turns
> 5. Implement session state management:
>    - Save session to `.uncan/director/sessions/[session_id].json`
>    - Track: runbook path, turn statuses, artifacts, timestamps
>    - Resume capability (load existing session)
> 6. Integrate with existing systems:
>    - Use `MetricsCollector` to track execution metrics
>    - Emit `TerritoryEvent` when agents acquire/release files
>    - Log to `LedgerWriter` if enabled
> 7. Add Tauri commands for UI:
>    - `load_runbook(path: String) -> Result<RunbookSummary>`
>    - `start_runbook() -> Result<()>`
>    - `get_turn_status() -> Vec<TurnStatus>`
>    - `pause_execution() -> Result<()>`
>    - `resume_execution() -> Result<()>`
>
> **Error Handling**:
> - Agent spawn failures: Retry once, then escalate
> - Turn timeout (default 30 min): Mark as failed, notify human
> - Parse errors in agent output: Log and continue (best-effort)
>
> **Files to Create**:
> - `liminal-v1/src-tauri/src/director/orchestrator.rs`
> - `liminal-v1/src-tauri/src/director/session.rs` (session state)
> - `.uncan/systems/runbooks/impl_director_orchestrator.md` (design doc)

**Acceptance:**
- ✅ Can load and parse runbooks
- ✅ Can spawn multiple agents in parallel groups
- ✅ Dependency resolution works correctly
- ✅ Session state persists across restarts
- ✅ Tauri commands expose functionality to UI
- ✅ Integration with router, metrics, ledger
- ✅ `cargo fmt`, `cargo clippy`, `cargo test` pass

---

## Turn 6 — Systems Agent

**Specialist:** Systems
**Parallel Group:** N/A (Sequential)
**Dependencies:** Turn 5

**Prompt to Delegate:**
> Build the runbook executor that integrates all components and provides the main execution loop.
>
> **Context**: Now that we have the parser, Claude wrapper, and orchestrator, we need to tie it all together into a cohesive execution engine. This is the "main loop" that the UI will invoke to run runbooks.
>
> **Tasks**:
> 1. Create `director/executor.rs` module
> 2. Implement `RunbookExecutor`:
>    ```rust
>    pub struct RunbookExecutor {
>        orchestrator: DirectorAgent,
>        max_parallel: usize,
>        execution_handle: Option<JoinHandle<Result<ExecutionSummary>>>,
>    }
>    ```
> 3. Implement execution loop:
>    ```rust
>    pub async fn execute(&mut self) -> Result<ExecutionSummary> {
>        // 1. Validate runbook (all dependencies resolvable)
>        // 2. For each turn or parallel group:
>        //    a. Check dependencies satisfied
>        //    b. Spawn agents (up to max_parallel)
>        //    c. Send turn prompts
>        //    d. Monitor for completion
>        //    e. Collect artifacts
>        //    f. Update session state
>        // 3. Handle failures and escalations
>        // 4. Return summary
>    }
>    ```
> 4. Implement real-time progress updates:
>    - Use `tokio::sync::broadcast` to emit turn events
>    - Events: TurnStarted, TurnProgress(%), TurnCompleted, TurnFailed
>    - UI subscribes to these events for live updates
> 5. Implement cancellation:
>    - Graceful shutdown: finish current turns, stop new turns
>    - Force shutdown: SIGTERM all agents, cleanup
> 6. Add comprehensive logging:
>    - Log turn starts, completions, failures
>    - Log agent outputs to session directory
>    - Integration with existing ledger if enabled
> 7. Write integration test:
>    - Create minimal test runbook (2 turns)
>    - Execute end-to-end with mock agents
>    - Verify session state, artifacts, events
>
> **Files to Create**:
> - `liminal-v1/src-tauri/src/director/executor.rs`
> - `liminal-v1/src-tauri/tests/runbook_execution_test.rs` (integration test)
> - `.uncan/systems/runbooks/impl_executor.md` (design doc)

**Acceptance:**
- ✅ Can execute runbooks end-to-end
- ✅ Parallel execution respects max_parallel limit
- ✅ Real-time events emitted for UI consumption
- ✅ Cancellation works gracefully
- ✅ Integration test passes with mock agents
- ✅ Session artifacts saved correctly
- ✅ `cargo fmt`, `cargo clippy`, `cargo test` pass

---

## Turn 7 — Interface Agent

**Specialist:** Interface
**Parallel Group:** 2 (Parallel with Turn 8)
**Dependencies:** Turns 3, 6 (needs theme + executor)

**Prompt to Delegate:**
> Build the Director Dashboard UI that provides runbook management, agent oversight, and execution control.
>
> **Context**: The UI is the human director's command center. It needs to display runbook status, live agent terminals, turn progress, and provide controls to start/stop/pause execution. Use the theme system from Turn 3 and integrate with the Tauri backend from Turn 6.
>
> **Tasks**:
> 1. Create main dashboard layout:
>    - `src/views/DirectorDashboard.tsx`
>    - Top bar: Runbook selector, Start/Pause/Stop buttons, overall progress
>    - Left panel: Turn list with status indicators
>    - Main area: Agent terminal grid (2x2 or 3x2 layout)
>    - Right panel: Artifacts viewer (collapsible)
> 2. Implement runbook selector:
>    - `src/components/RunbookSelector.tsx`
>    - Dropdown listing available runbooks from `/runbooks/`
>    - Display epoch goal and turn count
>    - Load button triggers `load_runbook` Tauri command
> 3. Implement turn progress list:
>    - `src/components/TurnList.tsx`
>    - Display all turns with status badges (Pending, InProgress, Completed, Failed)
>    - Color code by status (gray, yellow, green, red)
>    - Show specialist role and parallel group
>    - Click to view turn details
> 4. Implement agent terminal grid:
>    - `src/components/AgentTerminalGrid.tsx`
>    - Grid of terminal views (reuse `Terminal.tsx` from Turn 3)
>    - Each terminal shows one agent's output
>    - Auto-scroll to latest output
>    - Color-coded header by agent role
> 5. Implement execution controls:
>    - `src/components/ExecutionControls.tsx`
>    - Start button: Trigger `start_runbook()` command
>    - Pause button: Trigger `pause_execution()` command
>    - Stop button: Trigger graceful shutdown
>    - Progress bar showing % complete
> 6. Implement real-time updates:
>    - Subscribe to turn events via Tauri event system
>    - Update turn statuses in real-time
>    - Stream agent outputs to terminals
>    - Notifications for turn completions/failures
> 7. State management (Zustand):
>    - `src/stores/runbookStore.ts`
>    - Store: current runbook, turn statuses, agent outputs
>    - Actions: loadRunbook, startExecution, updateTurnStatus
> 8. Styling:
>    - Use theme from Turn 3
>    - Responsive layout (min width 1280px)
>    - Dark theme with high contrast
>
> **Files to Create**:
> - `liminal-v1/src/views/DirectorDashboard.tsx`
> - `liminal-v1/src/components/RunbookSelector.tsx`
> - `liminal-v1/src/components/TurnList.tsx`
> - `liminal-v1/src/components/AgentTerminalGrid.tsx`
> - `liminal-v1/src/components/ExecutionControls.tsx`
> - `liminal-v1/src/stores/runbookStore.ts`
> - `.uncan/interface/runbooks/impl_dashboard.md` (design doc)

**Acceptance:**
- ✅ Dashboard layout functional and responsive
- ✅ Can load and display runbooks
- ✅ Can start/pause/stop execution
- ✅ Turn statuses update in real-time
- ✅ Agent terminal outputs stream correctly
- ✅ Theme consistent with UNCAN aesthetic
- ✅ `npm run lint` passes
- ✅ Manual smoke test successful

---

## Turn 8 — Testing Agent

**Specialist:** Testing
**Parallel Group:** 2 (Parallel with Turn 7)
**Dependencies:** Turn 6 (needs executor)

**Prompt to Delegate:**
> Create comprehensive integration tests for the runbook execution system.
>
> **Context**: We need automated tests that verify the end-to-end workflow: parse runbook → spawn agents → execute turns → collect artifacts. These tests will serve as regression protection and documentation.
>
> **Tasks**:
> 1. Create test runbooks in `liminal-v1/src-tauri/tests/fixtures/`:
>    - `simple_sequential.md` - 2 sequential turns
>    - `parallel_group.md` - 3 turns in parallel group
>    - `mixed_workflow.md` - Mix of sequential and parallel
>    - `error_handling.md` - Turn that intentionally fails
> 2. Create integration test suite:
>    - `liminal-v1/src-tauri/tests/integration_runbook_execution.rs`
> 3. Test scenarios:
>    - **Test 1: Simple Sequential**
>      - Load `simple_sequential.md`
>      - Execute with mock agents
>      - Verify turns execute in order
>      - Verify artifacts collected
>    - **Test 2: Parallel Group**
>      - Load `parallel_group.md`
>      - Verify 3 agents spawn simultaneously
>      - Verify all complete before next turn
>    - **Test 3: Dependency Resolution**
>      - Load `mixed_workflow.md`
>      - Verify sequential turns wait for parallel group completion
>    - **Test 4: Error Handling**
>      - Load `error_handling.md` with failing turn
>      - Verify execution stops gracefully
>      - Verify error logged to session
>    - **Test 5: Session Persistence**
>      - Start execution, pause mid-way
>      - Load session from disk
>      - Resume and verify state intact
> 4. Create mock agent for testing:
>    - `tests/mock_claude_agent.rs`
>    - Responds to prompts with canned outputs
>    - Configurable delay and success/failure
> 5. Add performance benchmarks:
>    - Measure agent spawn time
>    - Measure turn completion time
>    - Verify <500ms spawn, <30s simple turn
> 6. Document test strategy:
>    - `.uncan/testing/runbooks/integration_test_plan.md`
>    - Test coverage map
>    - How to add new tests
>
> **Files to Create**:
> - `liminal-v1/src-tauri/tests/integration_runbook_execution.rs`
> - `liminal-v1/src-tauri/tests/fixtures/` (test runbooks)
> - `liminal-v1/src-tauri/tests/mock_claude_agent.rs`
> - `.uncan/testing/runbooks/integration_test_plan.md`

**Acceptance:**
- ✅ All 5 test scenarios pass
- ✅ Mock agent correctly simulates Claude behavior
- ✅ Session persistence verified
- ✅ Error handling tested
- ✅ Performance benchmarks within targets
- ✅ Test documentation complete
- ✅ `cargo test` passes all integration tests

---

## Completion Criteria

Phase 1 is complete when:

✅ **Functional**:
- Can load any runbook from `/runbooks/` directory
- Can execute runbooks with sequential and parallel turns
- Can spawn Claude Code CLI agents automatically
- Can view agent outputs in real-time UI
- Can collect artifacts to session directory

✅ **Quality**:
- All unit tests pass (`cargo test`, `npm test`)
- Integration tests pass (Turn 8)
- No clippy warnings (`cargo clippy`)
- No lint errors (`npm run lint`)
- Code formatted (`cargo fmt`, `npm run lint`)

✅ **Documentation**:
- Design docs in `.uncan/[role]/runbooks/impl_*.md`
- Session context updated in `.uncan/[role]/context/session.md`
- Artifact protocol followed consistently

✅ **Demonstration**:
- Can execute this runbook (self-referential test)
- Can execute existing `epoch_router_foundation.md` runbook
- Human Director can load, start, monitor, and review results via UI

---

## Execution Notes

**Recommended Sequence** (for 3-agent capacity):

1. **Session 1** (Sequential):
   - Delegate Turn 1 to Systems Agent
   - Wait for completion (~2-3 hours)

2. **Session 2** (Sequential):
   - Delegate Turn 2 to Systems Agent
   - Wait for completion (~2-3 hours)

3. **Session 3** (Parallel x3):
   - Delegate Turn 3 to Interface Agent
   - Delegate Turn 4 to Router Agent
   - Delegate Turn 5 to Systems Agent (start after Turn 2 complete)
   - Wait for all three (~3-4 hours)

4. **Session 4** (Sequential):
   - Delegate Turn 6 to Systems Agent
   - Wait for completion (~2-3 hours)

5. **Session 5** (Parallel x2):
   - Delegate Turn 7 to Interface Agent
   - Delegate Turn 8 to Testing Agent
   - Wait for both (~3-4 hours)

**Total Estimated Time**: 15-20 hours of agent work, spread across 5 sessions

**Human Oversight**: Review artifacts after each session, provide feedback if needed, approve before next session.

---

## Session Context Template

After each Turn, update `.uncan/[role]/context/session.md`:

```markdown
## [Timestamp] - Turn [N]: [Title]

### Completed
- Files: [list modified/created files]
- Features: [what was built]
- Tests: [what was tested]

### Issues Encountered
- [Problem] - [Resolution]

### Handoff Notes
- Next agent needs: [context for next Turn]
```

---

**Human Director**: Review this runbook. Any questions or adjustments needed before we begin execution?