# Testing Agent

## Purpose
Ensure LIMINAL's quality through comprehensive testing, performance validation, and regression prevention.

## Core Expertise
- **Test Methodologies**: TDD, BDD, property-based testing
- **Coverage Analysis**: Code coverage, mutation testing
- **Performance Testing**: Load testing, stress testing, benchmarking
- **Integration Testing**: End-to-end flows, system integration
- **Regression Testing**: Automated test suites, CI/CD integration

## Testing Strategies

### Cargo Test Organization
```rust
// Unit tests - colocated with source
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_handles_priority_correctly() {
        let mut router = PriorityRouter::new();
        router.route(Message::critical("test")).unwrap();
        router.route(Message::info("test")).unwrap();

        // Critical message should be returned first
        assert_eq!(
            router.next_message().unwrap().priority,
            Priority::Critical
        );
    }
}

// Integration tests - in tests/ directory
// tests/router_integration.rs
#[test]
fn test_full_message_flow() {
    let app = spawn_test_app();

    // Spawn agent
    let agent_id = app.spawn_agent("test_agent").await.unwrap();

    // Send message through router
    app.send_message(agent_id, "test_message").await.unwrap();

    // Verify agent received message
    let response = app.await_response(agent_id).await.unwrap();
    assert_eq!(response.status, "processed");
}
```

### Performance Benchmarking
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn routing_benchmark(c: &mut Criterion) {
    let mut router = PriorityRouter::new();

    c.bench_function("route_1000_messages", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                router.route(Message::random()).unwrap();
            }
            // Drain queue
            while router.next_message().is_some() {}
        });
    });
}

criterion_group!(benches, routing_benchmark);
criterion_main!(benches);
```

### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn router_never_loses_messages(
        messages in prop::collection::vec(any::<Message>(), 0..100)
    ) {
        let mut router = PriorityRouter::new();
        let msg_count = messages.len();

        // Route all messages
        for msg in messages {
            router.route(msg).unwrap();
        }

        // Extract all messages
        let mut extracted = 0;
        while router.next_message().is_some() {
            extracted += 1;
        }

        // No messages lost
        assert_eq!(extracted, msg_count);
    }
}
```

### Coverage Analysis
```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Check coverage threshold
cargo tarpaulin --fail-under 80

# Mutation testing
cargo mutants --jobs 4 --timeout 300
```

### Load Testing Script
```typescript
// tests/load-test.ts
import { spawn } from '@tauri-apps/api/shell';

async function loadTest() {
  const agents = [];
  const messageRate = 1000; // messages per second

  // Spawn 50 agents
  for (let i = 0; i < 50; i++) {
    agents.push(await spawnAgent(`agent_${i}`));
  }

  // Send messages at target rate
  const startTime = Date.now();
  let messagesSent = 0;

  while (Date.now() - startTime < 60000) { // 1 minute test
    for (const agent of agents) {
      await sendMessage(agent.id, generateMessage());
      messagesSent++;
    }

    // Maintain target rate
    const expectedMessages = ((Date.now() - startTime) / 1000) * messageRate;
    if (messagesSent > expectedMessages) {
      await sleep(10);
    }
  }

  // Verify no message loss
  const stats = await getRouterStats();
  assert(stats.processed === messagesSent);
  assert(stats.avgLatency < 10); // <10ms requirement
}
```

## Regression Test Suite
```yaml
# .github/workflows/test.yml
name: Regression Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run unit tests
        run: cargo test
      - name: Run integration tests
        run: cargo test --test '*' --features integration
      - name: Check coverage
        run: cargo tarpaulin --fail-under 80
      - name: Run benchmarks
        run: cargo bench --no-fail-fast
      - name: Load test
        run: npm run test:load
```

## Test Quality Metrics
- **Code Coverage**: Target 80% minimum
- **Mutation Score**: Target 70% killed mutants
- **Test Execution Time**: <5 minutes for full suite
- **Flakiness**: 0 flaky tests tolerated
- **Performance Regression**: <5% deviation allowed

## Shortcuts
- `QTEST` - Generate test scaffold
- `QCOV` - Run coverage analysis
- `QBENCH` - Run benchmarks
- `QLOAD` - Run load tests
- `QREG` - Run regression suite

---
*Reference `../_base.md` for shared configuration*