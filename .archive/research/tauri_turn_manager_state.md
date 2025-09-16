# Managing Shared State in a Tauri Turn Manager

Developing a Turn Manager that is safe under asynchronous Tauri commands and WebSocket handlers requires careful choice of concurrency pattern. Two common Rust patterns for shared, mutable state are:
- Using shared state with locks (e.g. `Arc<Mutex<T>>` or `Arc<RwLock<T>>`), and
- Using a dedicated task/actor with channels to serialize access.

Each approach has trade-offs in a high-read, low-write scenario. Below, we compare these patterns and discuss best practices for your use case.

## Arc/Arc – Shared-State Concurrency

Using an `Arc<Mutex<T>>` or `Arc<RwLock<T>>` is the straightforward way to share a state object across threads/tasks in Rust. For a Turn Manager, you might keep an `Arc<RwLock<TurnsState>>` globally (or one per turn) and clone the Arc into each Tauri command or event handler that needs access. The read-heavy, write-light nature of your workload suggests using a read–write lock: an `RwLock` allows multiple concurrent readers while still ensuring exclusive access for writes. This means many UI queries can check the state in parallel without blocking each other, unlike a `Mutex` which only permits one lock at a time. Indeed, if you used a single-threaded manager (actor), every read would queue behind the previous one – an `RwLock` avoids that by design.

### Best Practices with Arc
Keep lock sections short and never hold a lock across an `.await`. Acquire the lock, copy or inspect the needed data, then release it before doing any slow or blocking work (like I/O or DB writes). This prevents blocking other tasks and avoids deadlocks. If a long operation is needed based on the state, extract the data under lock and perform the operation after unlocking. In your case, a Tauri command can grab a read lock to get the latest turn status and immediately drop it, ensuring minimal locking time.

### High Read, Low Write Optimization
The Tokio `RwLock` is fair (FIFO) so writers won’t starve under heavy reads. Given writes (state transitions) are infrequent (only on major events like `PLAN_APPROVED`), using a single `RwLock` should work well – readers won’t block each other, and occasional writes get priority to avoid starvation. If you expect different turns to be updated truly independently, you could even use finer-grained locks (e.g. one lock per turn or a concurrent map) to eliminate any false sharing. However, with only ~5 handlers total, a single lock guarding all turn states is simple and should not be a bottleneck.

### Performance
In uncontended scenarios, locking overhead is negligible – a `Mutex`/`RwLock` is extremely fast (just a few atomic ops). Unlike channel messaging, locking does not require heap allocation for each operation. When there’s no contention, a mutex or `RwLock` gives lower latency direct access to the data. In fact, an informal benchmark found that directly sharing state with `Arc<Mutex>` was roughly 2–3× faster than an actor/message-passing approach under light load. The upshot is that for a Turn Manager with low concurrency, a locking approach will be very efficient.

### Correctness Considerations
Using locks means you must manually handle synchronization, but Rust’s type system prevents data races as long as you stick to thread-safe types (`Send + Sync`). Be cautious to avoid deadlocks if you ever have more than one lock. Just remember not to `.await` while holding a lock. If you follow these practices, the `Arc<RwLock>` pattern is straightforward and easy to reason about for in-memory state.

### Persistence with Locks
Since state changes must persist to SQLite, you’ll need to write to the database whenever you mutate the state. One strategy is to perform the DB update outside the lock to keep lock spans short: acquire a write lock, apply the state transition in memory, clone the necessary data for persistence, then release the lock and commit the change. This introduces a small window where in-memory and DB states differ, but it’s usually acceptable if you immediately write and handle errors. If that inconsistency window is a concern, you could reverse the order or use an atomic transaction. With infrequent writes, either approach works.

---

## Channel-Based Actor – Single-Threaded Manager Task

An alternative is to implement the Turn Manager as an actor: spawn a dedicated Tokio task that owns all turn states and have other parts of the app communicate with it via message passing (e.g. an async mpsc channel). In this model, the actor task’s event loop is the only place that mutates the state. Tauri commands or WebSocket handlers send commands to the actor, and the actor serially processes these messages. This avoids explicit locks entirely.

### Design
You can structure this with an enum of message types and a handle object to send messages. For example:
```rust
enum TurnManagerMessage {
    GetState { turn_id: u64, respond_to: oneshot::Sender<State> },
    UpdateState { turn_id: u64, new_state: State },
}
```
The actor task awaits messages and updates or queries state. If a response is needed, the message includes a one-shot channel.

### Advantages
- Serialized, atomic updates, no locks or deadlocks.
- Easier to incorporate timers or background triggers.
- Naturally enforces backpressure with bounded channels.

### Drawbacks
- All operations serialize through one queue, so no parallel reads.
- Added latency and heap allocations per message.
- More code complexity compared to direct locks.

### Persistence with Actor
The actor can perform DB writes as part of message handling. For example, upon receiving `UpdateState`, write to DB then update memory. This ensures sync between DB and in-memory state, though DB latency blocks the actor.

---

## Comparison and Recommendation

Given your project characteristics – few concurrent handlers, frequent reads, infrequent writes – an `Arc<RwLock>`-based solution is simplest and most efficient. It allows concurrent reads with minimal overhead. The actor approach is useful if you later need complex async workflows, timers, or backpressure, but it introduces serialization and overhead.

### Summary of Trade-offs
- **Arc Approach**: Simple, concurrent reads, minimal latency, straightforward code. Requires discipline with locks but suits high-read/low-write.
- **Actor Approach**: Serialized, structured, eliminates locks. Good for async orchestration but worse throughput for reads and more boilerplate.

---

## Conclusion

For a high-read/low-write, low-concurrency Turn Manager, an `Arc<RwLock>` is the best practice. Keep lock spans short, persist to SQLite during writes, and only refactor to an actor if state logic grows more complex.
