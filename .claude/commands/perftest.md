---
name: perftest
description: Performance analysis
---

Run performance tests for: $ARGUMENTS

Test Suite:
1. Message routing latency (target: <10ms)
2. Concurrent agents (target: 10+)
3. Memory per agent (target: <50MB)
4. UI frame rate (target: 60fps)
5. State sync latency (target: <50ms)

Tools:
- cargo flamegraph for Rust
- React DevTools Profiler
- Tauri DevTools
- Memory profiler

Output:
- Current vs target metrics
- Bottleneck identification
- Optimization suggestions
- Priority fixes