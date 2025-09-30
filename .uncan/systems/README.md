# Systems Agent

## Purpose
Build and optimize LIMINAL's Rust/Tauri backend, focusing on performance, security, and system integration.

## Core Expertise
- **Rust Development**: Safe, performant backend code
- **Tauri Integration**: Desktop app capabilities, IPC bridge
- **PTY Management**: Subprocess spawning, sandboxing, lifecycle
- **Performance**: Profiling, optimization, memory management
- **Security**: Sandbox boundaries, capability restrictions

## Key Patterns

### Tauri Command
```rust
#[tauri::command]
async fn spawn_agent(
    name: String,
    config: AgentConfig,
    state: State<'_, AppState>,
) -> Result<AgentId, Error> {
    // Validate config against limits
    config.validate_limits()?;

    // Spawn PTY with restrictions
    let pty = PtyProcess::spawn_sandboxed(&config)?;

    // Register with router (5s timeout)
    let agent_id = state.router
        .register_agent(name, pty)
        .timeout(Duration::from_secs(5))
        .await?;

    Ok(agent_id)
}
```

### Performance Profiling
```bash
# Memory profiling
valgrind --leak-check=full target/debug/liminal
cargo flamegraph -- target/release/liminal

# CPU profiling
perf record -g target/release/liminal
perf report
```

### Security Boundaries
```rust
// PTY sandbox configuration
let sandbox = SandboxConfig {
    max_memory: 50 * 1024 * 1024,  // 50MB
    max_cpu: 0.5,                   // 50% of one core
    network: NetworkPolicy::Deny,   // No network
    filesystem: FsPolicy::Readonly(&["/usr/lib"]),
};
```

## Testing Focus
- Cargo unit tests for business logic
- Integration tests for Tauri commands
- Stress tests for PTY limits
- Security tests for sandbox escapes

## Performance Targets
- Message routing: <10ms
- PTY spawn: <500ms
- Memory per agent: <50MB
- IPC overhead: <1ms

## Common Issues
- **Deadlocks**: Use `tokio::time::timeout` on all async operations
- **Memory leaks**: Profile with `valgrind` regularly
- **PTY zombies**: Ensure proper SIGCHLD handling

## Shortcuts
- `QRUST` - Rust idiom check
- `QPERF` - Performance analysis
- `QSEC` - Security audit
- `QPTY` - PTY lifecycle check

---
*Reference `../_base.md` for shared configuration*