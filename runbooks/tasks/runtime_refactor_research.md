# Runtime Refactor Research & Development Runbook

## Context
The LIMINAL V1 MVP currently has a stopgap solution for Tokio runtime initialization. The MaintenanceExecutor creates its own runtime when none exists, which works but introduces architectural complexity with multiple independent runtimes.

## Problem Statement
- MaintenanceExecutor spawns workers at construction time (synchronous main)
- Tauri initializes its async runtime later in the lifecycle
- Current fix: Executor owns a dedicated runtime if no ambient one exists
- Risk: Multiple runtimes complicate shutdown and resource management

## Research Objectives
1. Determine optimal runtime initialization strategy
2. Minimize runtime duplication
3. Ensure clean shutdown semantics
4. Maintain compatibility with both dev and test environments

## Investigation Tasks

### Task 1: Map Current Runtime Lifecycle
**Goal**: Document exact initialization sequence

1. Trace main() execution path
   - [ ] Document when MaintenanceExecutor::new() is called
   - [ ] Identify all tokio::spawn calls before Tauri::Builder::run()
   - [ ] Map which components depend on early async initialization

2. Analyze Tauri runtime bootstrap
   - [ ] Find where Tauri creates its runtime (likely in Builder::build or run)
   - [ ] Check if Tauri exposes runtime handle before run()
   - [ ] Document Tauri v2 async context guarantees

3. Test environment analysis
   - [ ] Verify #[tokio::test] runtime behavior
   - [ ] Confirm integration tests still work with changes

**Deliverable**: Sequence diagram of runtime initialization

### Task 2: Evaluate Refactor Option A - Centralized Runtime
**Goal**: Single runtime owned by main()

1. Prototype implementation
   ```rust
   fn main() {
       let runtime = tokio::runtime::Runtime::new().unwrap();
       let _guard = runtime.enter();

       runtime.block_on(async {
           let executor = MaintenanceExecutor::new();
           let router = UnifiedMessageRouter::new(executor.clone());
           // ... rest of initialization

           tauri::Builder::default()
               .manage(executor)
               .run(tauri::generate_context!())
               .expect("error running tauri");
       });
   }
   ```

2. Test points
   - [ ] Verify Tauri works inside block_on
   - [ ] Check if Tauri tries to create nested runtime
   - [ ] Measure startup performance impact

3. Risk assessment
   - [ ] Document Tauri version constraints
   - [ ] Test on all target platforms (macOS priority)

**Deliverable**: Working prototype or failure analysis

### Task 3: Evaluate Refactor Option B - Delayed Initialization
**Goal**: Initialize executor after Tauri setup

1. Investigate Tauri setup hook
   ```rust
   tauri::Builder::default()
       .setup(|app| {
           // Async context should be available here
           let executor = MaintenanceExecutor::new();
           app.manage(executor);
           Ok(())
       })
   ```

2. Dependency analysis
   - [ ] Can router/territory wait for setup phase?
   - [ ] Do any commands need executor before first invoke?
   - [ ] Check initialization order constraints

3. Implementation approach
   - [ ] Move executor creation to setup closure
   - [ ] Use OnceCell/Lazy for late initialization if needed
   - [ ] Ensure all State<> accesses are safe

**Deliverable**: Feasibility report with code samples

### Task 4: Evaluate Refactor Option C - Native Tokio Primitives
**Goal**: Replace custom executor with Tokio built-ins

1. Analyze MaintenanceExecutor requirements
   - [ ] List all work-stealing queue features used
   - [ ] Document priority/scheduling requirements
   - [ ] Check if tokio::task::JoinSet suffices

2. Migration path
   - [ ] Replace custom queue with tokio::sync::mpsc
   - [ ] Use tokio::select! for priority handling
   - [ ] Consider tokio::task::LocalSet for pinned work

3. Performance comparison
   - [ ] Benchmark message throughput
   - [ ] Measure scheduling latency
   - [ ] Profile memory usage

**Deliverable**: Recommendation with benchmarks

## Implementation Plan

### Phase 1: Research (2-3 hours)
1. Complete Task 1 - Map lifecycle
2. Quick spike on most promising option
3. Document findings

### Phase 2: Prototype (3-4 hours)
1. Implement chosen approach
2. Update all dependent systems
3. Ensure tests pass

### Phase 3: Validation (1-2 hours)
1. Run integration tests
2. Test npm run tauri dev
3. Verify no regressions

### Phase 4: Cleanup (1 hour)
1. Remove old runtime creation code
2. Update documentation
3. Add lifecycle tests

## Success Criteria
- [ ] Single runtime instance (or well-documented multi-runtime design)
- [ ] Clean shutdown without hanging threads
- [ ] All tests pass
- [ ] npm run tauri dev works reliably
- [ ] No performance regression

## Risk Mitigation
- Keep current working solution as fallback branch
- Test each option in isolation first
- Document Tauri version dependencies
- Add runtime lifecycle tests

## Decision Matrix

| Option | Complexity | Risk | Performance | Maintainability |
|--------|------------|------|-------------|-----------------|
| A: Centralized | High | Medium | Good | Good |
| B: Delayed | Medium | Low | Good | Excellent |
| C: Native | Low | Low | Excellent | Excellent |
| Current (stopgap) | Low | Low | Good | Poor |

## Recommended Approach
Start with Option C (Native Tokio) as it has lowest risk and best long-term maintainability. Fall back to Option B if custom scheduling is truly needed.

## Commands for Testing
```bash
# After each refactor attempt
cd /Users/grey/War/projects/LIMINAL/liminal-v1
cargo test
npm run tauri dev
cargo test --test integration_test

# Check for runtime panics
RUST_BACKTRACE=1 npm run tauri dev
```

## Notes
- Current stopgap works; this is about architectural cleanliness
- Coordinate with router/territory refactoring if pursuing
- Consider impact on future consensus/ledger integration