# LIMINAL V1 MVP - Multi-Agent Orchestration Platform

## Critical Guidelines

- NEVER put comments in code unless explicitly requested
- ALWAYS run `cargo fmt` and `npm run lint` before considering task complete
- PREFER editing existing files over creating new ones
- MINIMIZE output tokens while maintaining accuracy
- NO Claude Code attribution in commits - author is WAR-0
- FOLLOW existing code conventions in each file

## Architecture Invariants

### Message Router
- ALL agent communication through central router only
- Priority queue with BTreeMap for O(log n) operations
- Critical messages require acknowledgment within 100ms
- No direct agent-to-agent communication allowed

### Agent Lifecycle
1. Spawn PTY subprocess with resource limits
2. Register with router (5s timeout)
3. Request territory lease
4. Process messages via router
5. Release lease on shutdown
6. Force-kill after 10s on SIGTERM

### Tauri Bridge
- Commands for synchronous RPC
- Events for real-time updates
- TypeScript types match Rust structs exactly
- Use ts-rs for type generation

## Performance Requirements
- Message routing: <10ms
- Agent spawn: <500ms
- UI updates: 60fps
- State sync: <50ms
- Memory/agent: <50MB

## Security Boundaries
- PTY sandbox for agents
- Lease system for resources
- Message validation at router
- Sanitize agent output for UI

## Writing Functions Checklist
1. Readable and followable?
2. Cyclomatic complexity <10?
3. Proper typing (no `any`)?
4. Testable without mocks?
5. Under 50 lines?

## Shortcuts

### Core Development
- `AGENT` - New agent implementation
- `ROUTER` - Message routing work
- `TERRITORY` - Lease management
- `SYNCTYPE` - Type synchronization

### Quality Assurance
- `PERFTEST` - Performance analysis
- `SAFETY` - Security verification
- `DIAGNOSE` - Debug issues

### Workflow
- `SPRINT` - Check objectives
- `QCODE` - Quick implementation
- `COMPACT` - Compress context

## Build Commands
```bash
npm run tauri dev     # Development
cargo test            # Rust tests
npm test              # React tests
cargo fmt             # Format
cargo clippy -- -W all # Lint
npm run lint          # ESLint
npm run tauri build   # Production
```

## Git Standards
- Format: `component: action`
- No AI/Claude references
- Author: WAR-0
- Atomic commits