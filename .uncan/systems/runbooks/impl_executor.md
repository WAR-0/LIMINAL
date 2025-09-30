# Runbook Executor Implementation

## Overview
The RunbookExecutor provides a high-level API for executing runbooks end-to-end, with real-time progress events for UI consumption.

## Architecture

### Core Components
1. **RunbookExecutor**: Main entry point, wraps DirectorAgent
2. **ExecutionEvent**: Real-time event stream via broadcast channel
3. **ExecutionSummary**: Final results after completion
4. **Cancellation**: Graceful and force shutdown support

### Event Flow
```
RunbookExecutor
  └─> DirectorAgent (execution loop)
       └─> ClaudeCodeAgent (per role)
            └─> PTY Process (Claude CLI)

Events flow back through broadcast channel:
  RunbookStarted
  └─> TurnStarted
       └─> TurnProgress (optional)
       └─> TurnCompleted | TurnFailed
  └─> RunbookCompleted | RunbookFailed
```

## ExecutionEvent Types
- **RunbookStarted** { epoch_id, total_turns }
- **TurnStarted** { turn_id, specialist, timestamp }
- **TurnProgress** { turn_id, progress_pct } (future)
- **TurnCompleted** { turn_id, duration_ms, artifacts }
- **TurnFailed** { turn_id, error_message }
- **RunbookCompleted** { total_duration_ms, completed_turns, failed_turns }
- **RunbookFailed** { error_message }

## API Design

### RunbookExecutor
```rust
pub struct RunbookExecutor {
    orchestrator: DirectorAgent,
    max_parallel: usize,
    execution_handle: Option<JoinHandle<Result<ExecutionSummary>>>,
    event_tx: broadcast::Sender<ExecutionEvent>,
}

impl RunbookExecutor {
    pub fn new(working_dir: PathBuf, metrics: MetricsCollector, router: UnifiedMessageRouter) -> Self;
    pub fn with_max_parallel(mut self, max: usize) -> Self;
    pub fn subscribe(&self) -> broadcast::Receiver<ExecutionEvent>;
    pub async fn load_runbook(&self, path: &Path) -> Result<RunbookSummary>;
    pub async fn execute(&mut self) -> Result<ExecutionSummary>;
    pub async fn cancel(&self, force: bool) -> Result<()>;
}
```

### ExecutionSummary
```rust
pub struct ExecutionSummary {
    pub epoch_id: String,
    pub total_turns: usize,
    pub completed_turns: usize,
    pub failed_turns: usize,
    pub total_duration_ms: u64,
    pub turn_summaries: Vec<TurnSummary>,
}

pub struct TurnSummary {
    pub turn_id: usize,
    pub specialist: AgentRole,
    pub status: TurnStatus,
    pub duration_ms: u64,
    pub artifacts: Vec<PathBuf>,
}
```

## Implementation Strategy

### Phase 1: Event Infrastructure
1. Add broadcast channel to RunbookExecutor
2. Define ExecutionEvent enum
3. Hook into DirectorAgent execution loop

### Phase 2: Execution Loop Enhancement
1. Wrap DirectorAgent's start_execution
2. Monitor execution via polling get_turn_status
3. Emit events on status changes
4. Build ExecutionSummary on completion

### Phase 3: Cancellation
1. Graceful: call orchestrator.pause() then shutdown()
2. Force: call orchestrator.shutdown() immediately

### Phase 4: Logging
1. Log all events to tracing
2. Save turn outputs to `.uncan/director/sessions/{session_id}/`
3. Integrate with existing metrics collector

## Testing Strategy

### Unit Tests
- Event emission on turn state changes
- Cancellation (graceful vs force)
- ExecutionSummary building

### Integration Test
1. Create minimal test runbook (2 sequential turns)
2. Mock ClaudeCodeAgent to respond immediately
3. Verify:
   - Events emitted in correct order
   - Session state saved
   - Artifacts collected
   - Summary accurate

## File Structure
```
src-tauri/src/director/
├── mod.rs              (add executor export)
├── executor.rs         (new: RunbookExecutor + ExecutionEvent)
├── orchestrator.rs     (existing: DirectorAgent)
├── claude_agent.rs     (existing: ClaudeCodeAgent)
├── parser.rs           (existing: RunbookParser)
├── runbook.rs          (existing: Runbook, Turn)
└── session.rs          (existing: Session)

tests/
└── runbook_execution_test.rs  (new: integration test)
```

## Open Questions
1. Should we add TurnProgress events? (requires agent output streaming)
2. Should cancellation wait for in-progress turns? (current: yes for graceful)
3. How to handle partial failures? (current: mark turn as failed, continue)

## Success Criteria
- ✅ Can execute runbooks end-to-end
- ✅ Events emitted in real-time
- ✅ Cancellation works (graceful and force)
- ✅ Session artifacts saved
- ✅ Integration test passes
- ✅ All existing tests still pass