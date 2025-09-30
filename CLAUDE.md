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

## Artifact & Output Protocol

### Code Artifacts
- Write directly to source files (edit existing or create new)
- Rust: `liminal-v1/src-tauri/src/[module]/`
- TypeScript: `liminal-v1/src/[component]/`
- Tests: Standard locations (`tests/`, `__tests__/`)

### Documentation Artifacts
- **Design docs**: `.uncan/[role]/runbooks/impl_[feature].md`
- **Reports**: `.uncan/[role]/reports/[report_name].md`
- **Session context**: `.uncan/[role]/context/session.md` (update after each Turn)

### Completion Checklist
- ✅ Run `cargo fmt` and `cargo clippy` for Rust changes
- ✅ Run `npm run lint` for TypeScript changes
- ✅ Update session context with summary
- ✅ List modified files in Turn response

### Human Director Partnership
The Human Director provides:
- Strategic vision and architectural decisions
- Quality gate and creative direction
- Final approval on integration and releases

The Director Agent (AI) handles:
- Task planning and runbook creation
- Technical execution and coordination
- Efficiency optimization and error correction
- **Authority to challenge assumptions and suggest improvements**

Both collaborate as peers with complementary strengths.

## LIMINAL Workflow Reference
Claude participates in LIMINAL Epochs as a specialist agent. Expect prompts to arrive one Turn at a time from a runbook prepared by the Director Agent.

1. Human Director and Director Agent align on the Epoch goal.
2. Director Agent produces a Markdown runbook composed of ordered Turns.
3. Human Director copies a Turn prompt and delegates it to Claude (or another specialist).
4. Claude executes the work described in the prompt.
5. Claude responds with the requested artifacts plus a concise summary of actions taken.
6. The Human Director repeats steps 3–5 until the runbook is complete.
7. Human Director reports overall results back to the Director Agent.
8. The cycle restarts for the next Epoch.

Stay within the Turn’s scope, ask for clarification when the prompt is ambiguous, and assume the Director Agent will factor your summary into the next planning cycle.

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
- Configure git once per repo:
  ```bash
  git config user.name "WAR-0"
  git config user.email "warcodes@proton.me"
  git config commit.gpgsign false
  ```
- Commit frequently using `component: action` (add scope tags when helpful).
- Keep changes atomic and self-contained; avoid bundling unrelated edits.
- Do not mention Claude or AI tools in commit messages or PR descriptions.
