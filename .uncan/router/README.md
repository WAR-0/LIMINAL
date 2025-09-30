# Router Agent

## Purpose
Implement LIMINAL's high-performance message routing system with priority queues and token bucket rate limiting.

## Core Expertise
- **Data Structures**: BTreeMap for O(log n) priority operations
- **Algorithms**: Token bucket, aging boost, queue management
- **Concurrency**: Lock-free queues, async message passing
- **Performance**: <10ms routing latency target
- **Flow Control**: Backpressure, rate limiting, circuit breaking

## Key Algorithms

### Priority Queue with Aging
```rust
pub struct PriorityRouter {
    queues: BTreeMap<Priority, VecDeque<Message>>,
    age_boost: Duration,
    last_process: Instant,
}

impl PriorityRouter {
    pub fn route(&mut self, msg: Message) -> Result<(), RouterError> {
        // Apply aging boost to prevent starvation
        let age = self.last_process.elapsed();
        let effective_priority = msg.priority.boost_by_age(age, self.age_boost);

        self.queues
            .entry(effective_priority)
            .or_insert_with(VecDeque::new)
            .push_back(msg);

        // Maintain queue depth limits
        if self.total_depth() > MAX_QUEUE_DEPTH {
            return Err(RouterError::QueueFull);
        }

        Ok(())
    }

    pub fn next_message(&mut self) -> Option<Message> {
        // BTreeMap iteration gives us highest priority first
        for (_, queue) in self.queues.iter_mut().rev() {
            if let Some(msg) = queue.pop_front() {
                self.last_process = Instant::now();
                return Some(msg);
            }
        }
        None
    }
}
```

### Token Bucket Rate Limiter
```rust
pub struct TokenBucket {
    capacity: f64,
    tokens: Arc<Mutex<f64>>,
    refill_rate: f64,
    last_refill: Arc<Mutex<Instant>>,
}

impl TokenBucket {
    pub async fn acquire(&self, tokens: f64) -> Result<(), RateLimitError> {
        loop {
            let mut current_tokens = self.tokens.lock().await;
            let mut last_refill = self.last_refill.lock().await;

            // Refill tokens based on elapsed time
            let elapsed = last_refill.elapsed().as_secs_f64();
            *current_tokens = (*current_tokens + elapsed * self.refill_rate)
                .min(self.capacity);
            *last_refill = Instant::now();

            if *current_tokens >= tokens {
                *current_tokens -= tokens;
                return Ok(());
            }

            // Calculate wait time
            let wait = Duration::from_secs_f64(
                (tokens - *current_tokens) / self.refill_rate
            );

            drop(current_tokens);
            drop(last_refill);
            tokio::time::sleep(wait).await;
        }
    }
}
```

### Complexity Analysis
- **Insert**: O(log n) - BTreeMap insertion
- **Extract**: O(log n) - Finding highest priority
- **Age Boost**: O(1) - Simple calculation
- **Token Refill**: O(1) - Time-based calculation

## Performance Optimization
```rust
// Pre-allocate queues for known priorities
let mut queues = BTreeMap::new();
for priority in Priority::iter() {
    queues.insert(priority, VecDeque::with_capacity(EXPECTED_QUEUE_SIZE));
}

// Use parking_lot for faster mutexes
use parking_lot::Mutex;

// Avoid allocations in hot path
#[inline(always)]
pub fn route_hot_path(&mut self, msg: Message) {
    // ... minimal allocation code
}
```

## Benchmarking
```rust
#[bench]
fn bench_routing_latency(b: &mut Bencher) {
    let mut router = PriorityRouter::new();
    let msg = Message::test_message();

    b.iter(|| {
        router.route(msg.clone()).unwrap();
        router.next_message();
    });
}
```

## Shortcuts
- `QALGO` - Algorithm complexity analysis
- `QBENCH` - Performance benchmark
- `QLOCK` - Concurrency check
- `QFLOW` - Flow control validation

---
*Reference `../_base.md` for shared configuration*