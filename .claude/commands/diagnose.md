---
name: diagnose
description: Debug system issues
---

Diagnose issue: $ARGUMENTS

Checklist:
1. PTY subprocess health
2. Router queue state
3. Territory leases
4. Tauri events
5. Performance metrics
6. Memory usage
7. CPU profiling
8. Network activity

Tools:
- RUST_LOG=trace
- Chrome DevTools
- cargo expand
- strace/dtrace
- valgrind

Common issues:
- PTY spawn failures
- Queue bottlenecks
- Lease deadlocks
- Type mismatches
- Memory leaks