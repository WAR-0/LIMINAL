# LIMINAL Development Base Configuration

## Git Configuration (ALL AGENTS)
```bash
git config user.name "WAR-0"
git config user.email "warcodes@proton.me"
git config commit.gpgsign false
```

## Context Persistence (ALL AGENTS)
After each turn, write to `./context/session.md` (relative to your agent directory):
```markdown
## [Timestamp] - Turn Summary
- **Decisions**: [Key decisions made]
- **Changes**: [Files modified/created]
- **Issues**: [Problems encountered]
- **Next**: [What the next agent needs to know]
```

## Core Architecture (ALL AGENTS MUST RESPECT)
- **Performance**: routing <10ms, UI 60fps, memory <50MB/agent
- **Security**: PTY sandbox, lease system, message validation
- **Communication**: ALL through central router, NO direct agent-to-agent
- **Tech Stack**: Vite + React 18 + TypeScript + Tauri 2 + Rust

## Build Commands
```bash
cd liminal-v1
npm run tauri dev     # Development
cargo test            # Rust tests
cargo fmt             # Format
npm run build         # Production
```

## Turn Execution Protocol
1. Receive turn specification from Director
2. Execute ONLY within turn scope
3. Write context summary before ending
4. No autonomous chaining beyond turn boundary

## Best Practices Format
Use BP-1/C-1/T-1 numbering for all best practices documentation.

## NO TIME ESTIMATES
Focus on complexity and optimization priorities, never temporal estimates.