# Designing a Message Priority and Routing System for Multi-Agent Communication

## Introduction
In a multi-agent coding system like FORGE (hosting several AI coding agents on a local desktop), an efficient message priority and routing mechanism is crucial. Agents such as Director, Frontend, Backend, Testing, and Reviewer (with clones for sub-tasks) constantly exchange messages for command-control (task assignments), coordination (e.g. resource leases), discussions (knowledge sharing), and telemetry (status updates). Although the system is not hard real-time, it targets soft real-time responsiveness (ideally under 100 ms message routing latency). This calls for a robust priority scheme to ensure urgent communications get prompt attention without starving routine messages.

To design an optimal solution, we draw inspiration from multiple domains:

- Modern Message Queue Systems (RabbitMQ, Kafka, Redis, ZeroMQ, NATS) for how they handle priority messaging and routing.
- Operating System Schedulers (Linux kernel scheduling, real-time algorithms) for managing task priorities, preemption, and avoiding priority inversion.
- Network QoS Mechanisms (DiffServ, traffic shaping, weighted fair queuing, token buckets) for ensuring high-priority traffic while maintaining fairness.
- Human Priority Protocols (emergency communications, hospital triage, air traffic control, military command precedence, incident response) for principles of escalation and priority levels.

Using these insights, we propose a Message Priority Specification with Routing Rules and Escalation Paths tailored to FORGE’s architecture and needs. The goal is to determine optimal priority levels (FORGE currently uses 4 levels: info, coordinate, blocking, critical), decide on static vs dynamic prioritization, prevent “priority inflation”, establish when high-priority messages should interrupt ongoing processing versus queueing, handle priority inversion between agents, and possibly grant certain agents (like the Director) special priority privileges.

## Lessons from Message Queue Systems

### RabbitMQ Priority Queues
RabbitMQ supports message priority by internally maintaining separate sub-queues for each priority level. It allows priority values 1–255, but recommends using only a small number of levels (1–5) to avoid overhead. Each additional priority level incurs memory/CPU overhead due to sub-queues. Use a handful of tiers (e.g., low, normal, high, critical). Delivery ordering interacts with consumer prefetch; small prefetch helps reordering. FORGE should mirror this: keep 4–5 levels max and implement a dispatcher that always pulls from the highest non-empty queue.

### Kafka Partitioning
Kafka does not provide broker-level priority reordering. Achieve effective priority by segregation and parallelism: separate topics/partitions per priority and allocate more consumers to high-priority buckets. Takeaway for FORGE: we are not constrained by log ordering, but the idea of scaling resources with priority applies.

### Redis
Redis Pub/Sub has no inherent priority. Implement priority queues using multiple lists or a sorted set. A common pattern: multiple lists per priority and BLPOP from high to low. FORGE router should maintain separate queues and always service the highest first.

### ZeroMQ
No built-in priority. Use multiple sockets/channels and poll high-priority first. This reinforces the separate-channel design and the need to prevent starvation with occasional servicing of lower levels.

### NATS
Subjects and queue groups, no native message priority. Use distinct subjects per priority and subscriber policy. FORGE will implement priority in-process; the lesson is simplicity and channel separation.

## Insights from Operating System Scheduling

- Few classes work better than many fine-grained levels. Reserve absolute preemption for truly critical tasks.
- Preemption: high-priority work interrupts lower-priority processing. Use cooperative preemption points in agents to remain responsive.
- Priority inversion: apply priority inheritance and ceilings. If a high-priority workflow depends on a lower-priority agent, elevate the handling of the dependent work.
- Fairness: adopt weighted servicing or aging to prevent starvation, similar to CFS ideas.

## Network QoS Analogies

- DiffServ: small number of traffic classes; rate-limit EF-like “critical” to avoid abuse. Map to FORGE: prevent priority inflation with quotas and admission control.
- WFQ: strict priority risks starvation. Use weighted service or aging so lower tiers see progress.
- Token bucket: allow bursts of critical, enforce average rate. Apply per-agent quotas for critical traffic.

## Human Systems

- Emergency comms: explicit override to seize channel for urgent traffic. Director Override mirrors this.
- Hospital triage: 3–5 tiers, dynamic reassessment, do the greatest good. Apply dynamic escalation and clear definitions.
- ATC: emergencies preempt, separate channels, holding patterns for less urgent traffic.
- Military precedence: Flash, Immediate, Priority, Routine, plus Flash Override. Strict preemption with disciplined use.

## FORGE Message Priority Design

### Priority Levels

1. Info (low): telemetry, logs, non-urgent discussion. Never interrupts.
2. Coordinate (normal): tasking, leases, routine ops. Prompt but yields to higher tiers.
3. Blocking (high): sender cannot proceed until addressed. Jumps ahead of Coordinate/Info.
4. Critical (highest): urgent alerts or commands. Preempt delivery and handling. Reserved for true emergencies.
5. Director Override (capability): rare top-level interruption for system control; only Director issues.

Rationale: four tiers balance clarity and control; aligns with messaging, QoS, and human systems.

### Router Mechanics

- Separate in-memory queues per priority: critical, blocking, coordinate, info.
- Dispatcher services highest non-empty queue; deliver immediately.
- Prefetch/flow control: limit in-flight per agent to keep reordering effective.
- Starvation mitigation:
  - Weighted round-robin slice for lower tiers, or
  - Aging-based promotion after wait thresholds.
- Broadcast: enqueue per recipient at corresponding priority.

### Static vs Dynamic Priority

- Static tagging by type/context at send time.
- Dynamic escalation:
  - Blocking escalates to Critical on timeout.
  - Coordinate escalates to Blocking if it starts blocking progress.
- Aging: router promotes long-waiting lower-tier messages.
- Inherited priority: propagate initiator priority along dependency chains.
- De-escalation by explicit resolution messages; no silent downgrades.

### Preventing Priority Inflation

- Clear policy and examples per tier.
- Restrict Critical to Director and vetted conditions; clones limited to Info/Coordinate.
- Monitoring dashboards and logs for high-tier usage.
- Token-bucket quotas for Critical per agent; excess demoted or delayed.
- Post-incident review to correct misuse.

### Interrupt vs Queue

- Agents process from inbox; implement cooperative preemption points.
- Critical interrupts current flow at safe boundaries; handle ASAP.
- Blocking handled next, ahead of any Coordinate/Info.
- Coordinate/Info strictly queue.
- Long tasks must yield periodically or run in sub-threads to remain responsive.
- Keep atomic sections brief to avoid masking interrupts.

### Priority Inversion Policy

- Propagate requester priority in dependent requests.
- Elevate handling of lock holders when high-priority dependents wait (inheritance).
- Director monitors for prolonged waits/deadlocks and intervenes.
- Avoid cyclic waits; Director override breaks deadlocks.

### Priority Privileges

- Director:
  - Full use of Blocking/Critical.
  - May issue Override.
  - Optional implicit elevation of Director’s Blocking above peers’.
- Workers (Frontend/Backend/Testing/Reviewer):
  - Use Coordinate/Blocking; Critical only for true failure signals, often to Director.
- Clones:
  - Info/Coordinate only; escalate via parent.

### Escalation Paths

- Time-based:
  - Blocking -> Critical on threshold breach; alert Director.
  - Coordinate -> Blocking if it halts progress beyond tolerance.
- State-based:
  - Multiple related issues escalate system-wide via Director.
  - Health anomalies generate Critical alerts to Director.
- Resolution:
  - “All clear” messages broadcast after fix; agents resume normal ops.

## Summary Rules

- Four levels plus Director Override.
- Strict priority dispatch with starvation safeguards.
- Cooperative preemption at agent level for Blocking/Critical.
- Dynamic escalation, priority inheritance, Director-led resolution.
- Controls and quotas to prevent inflation.
- Meet soft real-time target with in-process Rust router and per-agent flow control.

