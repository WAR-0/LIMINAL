# LIMINAL Integration Test Specification

This document defines comprehensive test scenarios for validating LIMINAL's multi-agent orchestration system. Each test category includes specific scenarios, success criteria, performance benchmarks, and implementation code.

## Test Harness Architecture

```rust
pub struct LiminalTestHarness {
    router: UnifiedMessageRouter,
    agents: HashMap<AgentId, TestAgent>,
    metrics_collector: MetricsCollector,
    chaos_engine: ChaosEngine,
    assertion_engine: AssertionEngine,
}

impl LiminalTestHarness {
    pub async fn setup(config: TestConfig) -> Result<Self> {
        // Initialize router with test configuration
        let router = UnifiedMessageRouter::new(config.router_config);

        // Spawn test agents
        let agents = Self::spawn_test_agents(config.agent_count).await?;

        // Setup metrics collection
        let metrics_collector = MetricsCollector::new(config.metrics_config);

        // Initialize chaos engine for fault injection
        let chaos_engine = ChaosEngine::new(config.chaos_config);

        Ok(Self {
            router,
            agents,
            metrics_collector,
            chaos_engine,
            assertion_engine: AssertionEngine::default(),
        })
    }

    pub async fn run_scenario(&mut self, scenario: TestScenario) -> TestResult {
        // Record start state
        let start_metrics = self.metrics_collector.snapshot();

        // Execute scenario
        let result = match scenario {
            TestScenario::Performance(perf) => self.run_performance_test(perf).await,
            TestScenario::Coordination(coord) => self.run_coordination_test(coord).await,
            TestScenario::Recovery(recovery) => self.run_recovery_test(recovery).await,
            TestScenario::EndToEnd(e2e) => self.run_end_to_end_test(e2e).await,
        };

        // Collect final metrics
        let end_metrics = self.metrics_collector.snapshot();

        // Validate assertions
        self.assertion_engine.validate(start_metrics, end_metrics, result)
    }
}
```

## 1. Core Performance Tests

### 1.1 Message Throughput Test

**Objective**: Validate the router can handle 1000+ messages per second with p99 latency under 1ms.

```typescript
describe('Message Throughput Performance', () => {
    let harness: TestHarness;

    beforeEach(async () => {
        harness = await TestHarness.setup({
            agentCount: 10,
            enableMetrics: true,
        });
    });

    it('should route 1000 messages per second', async () => {
        const messageCount = 10000;
        const messages = generateTestMessages(messageCount, {
            priorityDistribution: {
                info: 0.6,
                coordinate: 0.25,
                blocking: 0.1,
                critical: 0.05
            },
            sizeDistribution: {
                small: 0.7,   // <1KB
                medium: 0.25,  // 1-5KB
                large: 0.05    // 5-10KB
            }
        });

        const startTime = Date.now();

        // Send all messages
        const promises = messages.map(msg => harness.router.route(msg));
        await Promise.all(promises);

        const elapsed = Date.now() - startTime;
        const throughput = messageCount / (elapsed / 1000);

        // Assertions
        expect(throughput).toBeGreaterThan(1000);
        expect(harness.metrics.droppedMessages).toBe(0);
        expect(harness.metrics.routingErrors).toBe(0);
    });

    it('should maintain latency under load', async () => {
        const latencies: number[] = [];
        const loadGenerators = [];

        // Create background load
        for (let i = 0; i < 5; i++) {
            loadGenerators.push(generateBackgroundLoad(100)); // 100 msg/sec each
        }

        // Measure latency under load
        for (let i = 0; i < 1000; i++) {
            const start = performance.now();
            await harness.router.route(createMessage({
                priority: Priority.Coordinate,
                size: 1024
            }));
            const latency = performance.now() - start;
            latencies.push(latency);
        }

        // Stop load generators
        loadGenerators.forEach(gen => gen.stop());

        // Calculate percentiles
        const p50 = calculatePercentile(latencies, 50);
        const p99 = calculatePercentile(latencies, 99);
        const p999 = calculatePercentile(latencies, 99.9);

        expect(p50).toBeLessThan(0.5);   // 0.5ms p50
        expect(p99).toBeLessThan(1);     // 1ms p99
        expect(p999).toBeLessThan(5);    // 5ms p999
    });
});
```

### 1.2 Clone Spawn Performance Test

**Objective**: Validate clone creation meets <10ms target with various context sizes.

```typescript
describe('Clone Spawn Performance', () => {
    it('should spawn clones in under 10ms', async () => {
        const cloneManager = new CloneOrchestrator();
        const contextSizes = [
            { name: 'minimal', size: 512 },      // 512B
            { name: 'typical', size: 8192 },     // 8KB
            { name: 'large', size: 32768 },      // 32KB
        ];

        for (const { name, size } of contextSizes) {
            const agent = createTestAgent('frontend', {
                contextSize: size,
                activeLeases: 3,
                messageHistory: 100
            });

            const measurements: number[] = [];

            for (let i = 0; i < 100; i++) {
                const start = performance.now();
                const clone = await cloneManager.spawnClone(agent, {
                    purpose: 'discussion',
                    context: agent.getContext()
                });
                const spawnTime = performance.now() - start;
                measurements.push(spawnTime);

                // Cleanup
                await clone.terminate();
            }

            const avg = average(measurements);
            const p99 = calculatePercentile(measurements, 99);

            console.log(`Context ${name}: avg=${avg.toFixed(2)}ms, p99=${p99.toFixed(2)}ms`);

            expect(avg).toBeLessThan(10);
            expect(p99).toBeLessThan(15);
        }
    });

    it('should use differential snapshots efficiently', async () => {
        const agent = createTestAgent('backend');
        const snapshotManager = new ContextSnapshotManager();

        // Create initial snapshot
        const initial = await snapshotManager.createSnapshot(agent.state);
        expect(initial.type).toBe('full');
        expect(initial.size).toBeLessThan(10240); // <10KB

        // Make small changes (<100)
        for (let i = 0; i < 50; i++) {
            agent.state.updateField(`field_${i}`, Math.random());
        }

        // Should create differential
        const diff = await snapshotManager.createSnapshot(agent.state);
        expect(diff.type).toBe('differential');
        expect(diff.size).toBeLessThan(initial.size * 0.3);

        // Make large changes (>100)
        for (let i = 0; i < 150; i++) {
            agent.state.updateField(`field_${i}`, Math.random());
        }

        // Should create full snapshot
        const full = await snapshotManager.createSnapshot(agent.state);
        expect(full.type).toBe('full');
        expect(full.size).toBeLessThan(10240);
    });

    it('should maintain clone pool for fast spawning', async () => {
        const pool = new ClonePool({ targetSize: 10, spawnAhead: 5 });

        // Pre-warm pool
        await pool.initialize();

        // Measure acquisition time (should be instant from pool)
        const acquisitionTimes: number[] = [];

        for (let i = 0; i < 50; i++) {
            const start = performance.now();
            const process = await pool.acquire();
            const time = performance.now() - start;
            acquisitionTimes.push(time);

            // Return to pool
            await pool.release(process);
        }

        const avgAcquisition = average(acquisitionTimes);
        expect(avgAcquisition).toBeLessThan(0.1); // <0.1ms from pool
    });
});
```

### 1.3 Spatial Hash Efficiency Test

**Objective**: Validate O(1) territory lookups regardless of total territories.

```typescript
describe('Spatial Hash Performance', () => {
    it('should provide O(1) neighbor lookups', async () => {
        const territoryHash = new TerritoryHash({ cellSize: 100 });

        // Add territories in batches and measure lookup time
        const measurements: { count: number, avgTime: number }[] = [];

        for (let batch = 1; batch <= 10; batch++) {
            // Add 1000 territories
            for (let i = 0; i < 1000; i++) {
                territoryHash.addTerritory({
                    id: `territory_${batch}_${i}`,
                    position: {
                        x: Math.random() * 10000,
                        y: Math.random() * 10000
                    },
                    radius: 50
                });
            }

            // Measure lookup times
            const lookupTimes: number[] = [];

            for (let i = 0; i < 1000; i++) {
                const queryPos = {
                    x: Math.random() * 10000,
                    y: Math.random() * 10000
                };

                const start = performance.now();
                const neighbors = territoryHash.getNeighbors(queryPos, 200);
                const time = performance.now() - start;
                lookupTimes.push(time);
            }

            measurements.push({
                count: batch * 1000,
                avgTime: average(lookupTimes)
            });
        }

        // Verify O(1) complexity - time should not increase with count
        const times = measurements.map(m => m.avgTime);
        const stdDev = calculateStdDev(times);

        expect(stdDev).toBeLessThan(0.05); // Very low variance
        expect(Math.max(...times)).toBeLessThan(0.1); // All <0.1ms
    });

    it('should handle territory updates efficiently', async () => {
        const territoryHash = new TerritoryHash({ cellSize: 100 });

        // Add initial territories
        const territories = [];
        for (let i = 0; i < 1000; i++) {
            const territory = {
                id: `territory_${i}`,
                position: { x: i * 10, y: i * 10 },
                radius: 50
            };
            territories.push(territory);
            territoryHash.addTerritory(territory);
        }

        // Measure update performance
        const updateTimes: number[] = [];

        for (let i = 0; i < 1000; i++) {
            const territory = territories[i];
            const newPos = {
                x: territory.position.x + Math.random() * 100,
                y: territory.position.y + Math.random() * 100
            };

            const start = performance.now();
            territoryHash.updatePosition(territory.id, newPos);
            const time = performance.now() - start;
            updateTimes.push(time);
        }

        const avgUpdate = average(updateTimes);
        expect(avgUpdate).toBeLessThan(0.1); // <0.1ms per update
    });
});
```

## 2. Multi-Agent Coordination Tests

### 2.1 Lease Conflict Resolution

**Objective**: Test the full range of lease negotiation including deferral, escalation, and priority inheritance.

```typescript
describe('Lease Conflict Resolution', () => {
    let territoryManager: TerritoryManager;
    let agents: TestAgent[];

    beforeEach(async () => {
        territoryManager = new TerritoryManager({
            lease_conflict_escalation_threshold: 2,
            lease_deferral_threshold: 30,
        });

        agents = await createTestAgents(['frontend', 'backend', 'testing']);
    });

    it('should resolve simple lease conflict by priority', async () => {
        const [frontend, backend] = agents;

        // Frontend (low priority) acquires lease
        const lease1 = await territoryManager.acquireLease(frontend, {
            resource: 'src/api/users.ts',
            duration: 300,
            priority: Priority.Info
        });
        expect(lease1).toBeDefined();

        // Backend (high priority) requests same resource
        backend.setPriority(Priority.Blocking);
        const request = await territoryManager.requestTransfer(backend, {
            resource: 'src/api/users.ts',
            reason: 'Critical API update',
            priority: Priority.Blocking
        });

        // Should grant to higher priority
        expect(request.decision).toBe('grant');
        expect(request.revokedFrom).toBe(frontend.id);
    });

    it('should defer when holder near completion', async () => {
        const [frontend, backend] = agents;

        // Frontend holds lease with 80% progress
        const lease = await territoryManager.acquireLease(frontend, {
            resource: 'src/components/Header.tsx',
            duration: 100
        });

        frontend.setProgress(0.8);
        frontend.setTimeRemaining(25); // 25 seconds left

        // Backend requests
        const request = await territoryManager.requestTransfer(backend, {
            resource: 'src/components/Header.tsx',
            reason: 'Add feature',
            priority: Priority.Coordinate
        });

        // Should defer since holder almost done
        expect(request.decision).toBe('defer');
        expect(request.deferTime).toBeGreaterThan(20);
        expect(request.deferTime).toBeLessThan(30);
    });

    it('should escalate with multiple waiters', async () => {
        const [holder, waiter1, waiter2, waiter3] = await createTestAgents(4);

        // Holder acquires lease
        await territoryManager.acquireLease(holder, {
            resource: 'src/core/router.rs',
            duration: 600
        });

        // Multiple agents request
        const requests = [];
        for (const waiter of [waiter1, waiter2, waiter3]) {
            requests.push(territoryManager.requestTransfer(waiter, {
                resource: 'src/core/router.rs',
                reason: 'Need access',
                priority: Priority.Coordinate
            }));
        }

        const results = await Promise.all(requests);

        // Third request should trigger escalation
        expect(results[0].decision).toBe('queued');
        expect(results[1].decision).toBe('queued');
        expect(results[2].decision).toBe('escalate');
        expect(results[2].escalationReason).toBe('queue_depth_exceeded');
    });

    it('should handle priority inheritance', async () => {
        const [lowPriority, highPriority] = agents;

        // Low priority agent holds critical resource
        lowPriority.setPriority(Priority.Info);
        const lease = await territoryManager.acquireLease(lowPriority, {
            resource: 'src/db/connection.ts',
            duration: 300
        });

        // High priority agent needs it
        highPriority.setPriority(Priority.Critical);
        const request = await territoryManager.requestWithInheritance(highPriority, {
            resource: 'src/db/connection.ts',
            reason: 'Critical fix needed'
        });

        // Low priority agent should inherit high priority
        expect(lowPriority.getEffectivePriority()).toBe(Priority.Critical);
        expect(request.decision).toBe('inheritance_applied');

        // Priority restored after release
        await territoryManager.releaseLease(lease.id);
        expect(lowPriority.getEffectivePriority()).toBe(Priority.Info);
    });

    it('should negotiate complex conflicts', async () => {
        const [agent1, agent2] = agents;

        // Both agents hold overlapping territories
        await territoryManager.acquireLease(agent1, {
            resource: 'src/api/*',
            duration: 300
        });

        await territoryManager.acquireLease(agent2, {
            resource: 'src/api/auth/*',
            duration: 300
        });

        // Third agent needs both
        const agent3 = await createTestAgent('reviewer');
        const negotiation = await territoryManager.initiateNegotiation(agent3, {
            resources: ['src/api/*', 'src/api/auth/*'],
            proposal: {
                type: 'time_share',
                schedule: [
                    { agent: agent1.id, duration: 100 },
                    { agent: agent2.id, duration: 100 },
                    { agent: agent3.id, duration: 100 }
                ]
            }
        });

        // Should spawn clones for discussion
        expect(negotiation.clones).toHaveLength(3);
        expect(negotiation.expectedDuration).toBeLessThan(120);

        // Wait for consensus
        const consensus = await negotiation.waitForConsensus();
        expect(consensus.agreed).toBe(true);
        expect(consensus.finalSchedule).toBeDefined();
    });
});
```

### 2.2 Clone Discussion Lifecycle

**Objective**: Test full clone discussion from initiation through consensus to resolution merging.

```typescript
describe('Clone Discussion Lifecycle', () => {
    it('should complete full discussion cycle', async () => {
        const orchestrator = new CloneOrchestrator();
        const [frontend, backend, database] = await createTestAgents(3);

        // Initiate multi-party discussion about API design
        const discussion = await orchestrator.initiateDiscussion({
            topic: 'API Schema Design',
            participants: [frontend.id, backend.id, database.id],
            consensusTarget: ConsensusTarget.Majority,
            timeout: 300
        });

        expect(discussion.clones).toHaveLength(3);

        // Simulate discussion progress
        const events = [];
        discussion.on('message', (msg) => events.push(msg));

        // Frontend clone proposes
        await discussion.clones[0].propose({
            schema: { users: { fields: ['id', 'name', 'email'] } }
        });

        // Backend clone responds
        await discussion.clones[1].respond({
            agreement: 'partial',
            modifications: { users: { fields: ['id', 'name', 'email', 'role'] } }
        });

        // Database clone approves
        await discussion.clones[2].approve();

        // Check consensus reached
        const consensus = await discussion.waitForConsensus();
        expect(consensus.reached).toBe(true);
        expect(consensus.finalDecision).toMatchObject({
            schema: { users: { fields: ['id', 'name', 'email', 'role'] } }
        });

        // Verify clones terminated
        for (const clone of discussion.clones) {
            expect(clone.isTerminated()).toBe(true);
        }

        // Verify primary agents received consensus
        expect(frontend.hasConsensus('API Schema Design')).toBe(true);
        expect(backend.hasConsensus('API Schema Design')).toBe(true);
        expect(database.hasConsensus('API Schema Design')).toBe(true);
    });

    it('should handle discussion timeout', async () => {
        const orchestrator = new CloneOrchestrator();
        const agents = await createTestAgents(5);

        const discussion = await orchestrator.initiateDiscussion({
            topic: 'Complex Architecture Decision',
            participants: agents.map(a => a.id),
            consensusTarget: ConsensusTarget.Unanimous,
            timeout: 5 // Very short timeout
        });

        // Don't reach consensus
        await sleep(10);

        const result = await discussion.getResult();
        expect(result.timedOut).toBe(true);
        expect(result.escalated).toBe(true);
        expect(result.escalationTarget).toBe('Director');
    });
});
```

## 3. System Recovery and Fault Tolerance Tests

### 3.1 Agent Process Crash Recovery

**Objective**: Ensure graceful handling of unexpected agent termination.

```typescript
describe('Agent Crash Recovery', () => {
    it('should recover from agent crash', async () => {
        const system = await LiminalSystem.start();
        const agent = system.getAgent('frontend');

        // Agent acquires resources
        const lease1 = await agent.acquireLease('src/ui/App.tsx');
        const lease2 = await agent.acquireLease('src/ui/Header.tsx');

        // Spawn clones
        const clone1 = await agent.spawnClone('discussion');
        const clone2 = await agent.spawnClone('analysis');

        // Record state
        const precrashState = {
            leases: [lease1.id, lease2.id],
            clones: [clone1.id, clone2.id],
            messages: system.getQueuedMessagesFor(agent.id)
        };

        // Simulate crash
        await agent.process.kill('SIGKILL');

        // Wait for detection
        await sleep(1000);

        // Verify recovery actions
        expect(system.getLeaseHolder(lease1.resource)).toBeNull();
        expect(system.getLeaseHolder(lease2.resource)).toBeNull();
        expect(system.getClone(clone1.id)).toBeNull();
        expect(system.getClone(clone2.id)).toBeNull();

        // Verify notifications
        const directorMessages = system.getMessagesTo('Director');
        expect(directorMessages).toContainEqual(
            expect.objectContaining({
                type: 'AgentCrashed',
                agent: agent.id,
                resources: precrashState.leases
            })
        );

        // Verify agent restart
        await sleep(2000);
        const newAgent = system.getAgent('frontend');
        expect(newAgent).toBeDefined();
        expect(newAgent.id).not.toBe(agent.id); // New instance
        expect(newAgent.state).toBe('ready');
    });

    it('should handle cascading failures', async () => {
        const system = await LiminalSystem.start();

        // Create dependency chain
        const frontend = system.getAgent('frontend');
        const backend = system.getAgent('backend');
        const database = system.getAgent('database');

        // Frontend depends on Backend
        await frontend.addDependency(backend.id, 'api_client');

        // Backend depends on Database
        await backend.addDependency(database.id, 'db_connection');

        // Kill database (root cause)
        await database.process.kill('SIGKILL');

        // Wait for cascade detection
        await sleep(2000);

        // Verify cascade handling
        expect(backend.state).toBe('degraded');
        expect(frontend.state).toBe('degraded');

        // Verify escalation
        const alerts = system.getAlerts();
        expect(alerts).toContainEqual(
            expect.objectContaining({
                type: 'CascadingFailure',
                rootCause: database.id,
                affected: [backend.id, frontend.id]
            })
        );
    });
});
```

### 3.2 Deadlock Detection and Resolution

**Objective**: Verify deadlock detection and breaking mechanisms.

```typescript
describe('Deadlock Detection and Resolution', () => {
    it('should detect simple circular deadlock', async () => {
        const system = await LiminalSystem.start();
        const [agentA, agentB] = await system.getAgents(['frontend', 'backend']);

        // Agent A holds resource 1, wants resource 2
        const lease1 = await agentA.acquireLease('resource1');
        const request1 = agentA.requestLease('resource2'); // Don't await

        // Agent B holds resource 2, wants resource 1
        const lease2 = await agentB.acquireLease('resource2');
        const request2 = agentB.requestLease('resource1'); // Don't await

        // Wait for deadlock detection
        await sleep(1500); // Detection runs every 1s

        // Verify detection
        const deadlocks = system.getDeadlocks();
        expect(deadlocks).toHaveLength(1);
        expect(deadlocks[0].agents).toContain(agentA.id);
        expect(deadlocks[0].agents).toContain(agentB.id);

        // Verify resolution
        const resolution = await system.resolveDeadlock(deadlocks[0]);
        expect(resolution.victim).toBeDefined();
        expect(resolution.forcedReleases).toContain(
            resolution.victim === agentA.id ? lease1.id : lease2.id
        );

        // Verify one agent can proceed
        const results = await Promise.race([request1, request2]);
        expect(results).toBeDefined();
    });

    it('should detect complex multi-agent deadlock', async () => {
        const system = await LiminalSystem.start();
        const agents = await system.getAgents(['a', 'b', 'c', 'd']);

        // Create circular wait: A->B->C->D->A
        const leases = [];
        const requests = [];

        for (let i = 0; i < agents.length; i++) {
            const agent = agents[i];
            const resource = `resource${i}`;
            const nextResource = `resource${(i + 1) % agents.length}`;

            leases[i] = await agent.acquireLease(resource);
            requests[i] = agent.requestLease(nextResource); // Don't await
        }

        // Wait for detection
        await sleep(2000);

        // Verify complex deadlock detected
        const deadlocks = system.getDeadlocks();
        expect(deadlocks).toHaveLength(1);
        expect(deadlocks[0].agents).toHaveLength(4);
        expect(deadlocks[0].cycle).toEqual(['a', 'b', 'c', 'd', 'a']);

        // Verify victim selection (lowest priority)
        const resolution = await system.resolveDeadlock(deadlocks[0]);
        const victim = agents.find(a => a.id === resolution.victim);
        const victimPriority = victim.getPriority();

        for (const agent of agents) {
            if (agent.id !== victim.id) {
                expect(agent.getPriority()).toBeGreaterThanOrEqual(victimPriority);
            }
        }
    });
});
```

## 4. Message Priority Queue Tests

### 4.1 Priority Queue Validation

**Objective**: Test correct priority handling, starvation prevention, and rate limiting.

```typescript
describe('Message Priority Queue', () => {
    it('should maintain strict priority ordering', async () => {
        const router = new PriorityRouter();
        const messages = [];

        // Queue messages in reverse priority order
        messages.push(router.enqueue(createMessage(Priority.Info, 'msg1')));
        messages.push(router.enqueue(createMessage(Priority.Coordinate, 'msg2')));
        messages.push(router.enqueue(createMessage(Priority.Blocking, 'msg3')));
        messages.push(router.enqueue(createMessage(Priority.Critical, 'msg4')));
        messages.push(router.enqueue(createMessage(Priority.DirectorOverride, 'msg5')));

        await Promise.all(messages);

        // Dispatch and verify order
        const dispatched = [];
        while (true) {
            const msg = await router.dispatchNext();
            if (!msg) break;
            dispatched.push(msg);
        }

        expect(dispatched[0].content).toBe('msg5'); // DirectorOverride
        expect(dispatched[1].content).toBe('msg4'); // Critical
        expect(dispatched[2].content).toBe('msg3'); // Blocking
        expect(dispatched[3].content).toBe('msg2'); // Coordinate
        expect(dispatched[4].content).toBe('msg1'); // Info
    });

    it('should prevent starvation via aging', async () => {
        const router = new PriorityRouter({
            aging_threshold: 100, // 100ms
            aging_boost: 1        // Boost by 1 level
        });

        // Queue low priority message
        await router.enqueue(createMessage(Priority.Info, 'old_message'));

        // Wait for aging
        await sleep(150);

        // Queue high priority messages
        for (let i = 0; i < 10; i++) {
            await router.enqueue(createMessage(Priority.Coordinate, `new_${i}`));
        }

        // Old message should be boosted and dispatched early
        const dispatched = [];
        for (let i = 0; i < 3; i++) {
            dispatched.push(await router.dispatchNext());
        }

        // Aged Info message boosted to Coordinate level
        expect(dispatched.map(m => m.content)).toContain('old_message');
    });

    it('should enforce token bucket rate limiting', async () => {
        const router = new PriorityRouter();
        const agent = createTestAgent('spammer');

        // Try to send many Critical messages
        const results = [];
        for (let i = 0; i < 10; i++) {
            results.push(router.enqueue(createMessage(
                Priority.Critical,
                `critical_${i}`,
                agent.id
            )));
        }

        const outcomes = await Promise.allSettled(results);

        // First few should succeed (based on token bucket capacity)
        const succeeded = outcomes.filter(o => o.status === 'fulfilled').length;
        const failed = outcomes.filter(o => o.status === 'rejected').length;

        expect(succeeded).toBeLessThanOrEqual(5); // Default bucket capacity
        expect(failed).toBeGreaterThan(0);

        // Verify rate limit error
        const rejection = outcomes.find(o => o.status === 'rejected');
        expect(rejection.reason).toMatchObject({
            type: 'RateLimited',
            retryAfter: expect.any(Number)
        });
    });

    it('should detect and penalize priority gaming', async () => {
        const router = new PriorityRouter();
        const agent = createTestAgent('gamer');

        // Send many high-priority messages
        for (let i = 0; i < 20; i++) {
            await router.enqueue(createMessage(
                Priority.Blocking,
                `blocking_${i}`,
                agent.id
            ));
        }

        // Gaming detected, future messages downgraded
        const result = await router.enqueue(createMessage(
            Priority.Blocking,
            'should_be_downgraded',
            agent.id
        ));

        // Verify message was downgraded
        const queued = router.getQueuedMessages();
        const downgraded = queued.find(m => m.content === 'should_be_downgraded');
        expect(downgraded.priority).toBe(Priority.Info);

        // Verify gaming recorded
        expect(router.metrics.gamingDetections).toBeGreaterThan(0);
    });
});
```

## 5. End-to-End Scenarios

### 5.1 Complete Development Cycle

**Objective**: Test full feature implementation from goal to completion.

```typescript
describe('End-to-End Development Cycle', () => {
    it('should implement complete feature', async () => {
        const system = await LiminalSystem.start();
        const director = system.getDirectorAgent();

        // Human provides goal
        const goal = {
            description: 'Add user authentication with JWT',
            requirements: [
                'Login endpoint',
                'Registration endpoint',
                'JWT token generation',
                'Middleware for protected routes',
                'Unit and integration tests'
            ]
        };

        // Director creates plan
        const plan = await director.createExecutionPlan(goal);

        expect(plan.tasks).toHaveLength(8); // Decomposed tasks
        expect(plan.agents).toContain('frontend');
        expect(plan.agents).toContain('backend');
        expect(plan.agents).toContain('testing');

        // Human approves plan (HANDOFF POINT)
        await system.humanApprove(plan);

        // Execute plan
        const execution = await system.executePlan(plan);

        // Monitor parallel execution
        const parallelTasks = execution.getParallelTasks();
        expect(parallelTasks.length).toBeGreaterThan(1);

        // Wait for completion with progress tracking
        let lastProgress = 0;
        execution.on('progress', (progress) => {
            expect(progress).toBeGreaterThanOrEqual(lastProgress);
            lastProgress = progress;
        });

        const result = await execution.waitForCompletion();

        // Verify success criteria
        expect(result.status).toBe('completed');
        expect(result.tasksCompleted).toBe(plan.tasks.length);
        expect(result.testsPass).toBe(true);
        expect(result.buildSuccess).toBe(true);

        // Verify performance metrics
        expect(result.metrics.parallelism).toBeGreaterThan(2);
        expect(result.metrics.cloneSpawns).toBeGreaterThan(5);
        expect(result.metrics.avgCloneSpawnTime).toBeLessThan(10);
        expect(result.metrics.messageLatencyP99).toBeLessThan(1);

        // Verify feature actually works
        const testResult = await system.runIntegrationTest('auth_feature');
        expect(testResult.passed).toBe(true);
    });
});
```

### 5.2 Stress Test with Graceful Degradation

**Objective**: Test system behavior under extreme load with 20+ parallel agents.

```typescript
describe('Stress Test', () => {
    it('should handle 20+ parallel agents gracefully', async () => {
        const system = await LiminalSystem.start({
            maxAgents: 30,
            degradationThreshold: 0.8
        });

        // Spawn many agents
        const agents = [];
        for (let i = 0; i < 25; i++) {
            agents.push(await system.spawnAgent(`worker_${i}`));
        }

        // Generate high load
        const loadGenerator = new LoadGenerator({
            messagesPerSecond: 2000,
            cloneSpawnsPerSecond: 50,
            leaseRequestsPerSecond: 100
        });

        loadGenerator.start();

        // Monitor system health
        const healthMetrics = [];
        const collector = setInterval(() => {
            healthMetrics.push(system.getHealthMetrics());
        }, 100);

        // Run for 30 seconds
        await sleep(30000);

        loadGenerator.stop();
        clearInterval(collector);

        // Analyze metrics
        const avgCpuUsage = average(healthMetrics.map(m => m.cpuUsage));
        const avgMemoryUsage = average(healthMetrics.map(m => m.memoryUsage));
        const droppedMessages = healthMetrics[healthMetrics.length - 1].droppedMessages;

        // System should degrade gracefully
        expect(avgCpuUsage).toBeLessThan(0.9); // Not pegged
        expect(avgMemoryUsage).toBeLessThan(2048); // <2GB
        expect(droppedMessages).toBeLessThan(100); // Minimal drops

        // Verify degradation activated
        const degradationEvents = healthMetrics.filter(m => m.degradationActive);
        expect(degradationEvents.length).toBeGreaterThan(0);

        // Verify recovery after load
        await sleep(5000);
        const finalHealth = system.getHealthMetrics();
        expect(finalHealth.degradationActive).toBe(false);
        expect(finalHealth.queueDepths.critical).toBe(0);
    });

    it('should maintain SLAs under stress', async () => {
        const system = await LiminalSystem.start();
        const slaMonitor = new SLAMonitor({
            messageRoutingP99: 1,    // 1ms
            cloneSpawnP99: 15,       // 15ms
            territoryLookupP99: 0.1, // 0.1ms
        });

        // Apply stress
        const stressor = new StressGenerator({
            pattern: 'burst',
            intensity: 'high',
            duration: 60000
        });

        stressor.start();
        slaMonitor.start();

        await sleep(60000);

        stressor.stop();
        const violations = slaMonitor.getViolations();

        // Some violations acceptable under extreme stress
        expect(violations.length).toBeLessThan(10);
        expect(violations.filter(v => v.severity === 'critical')).toHaveLength(0);
    });
});
```

## 6. Chaos Engineering Tests

### 6.1 Network Partition Simulation

```typescript
describe('Chaos Engineering', () => {
    it('should handle network partitions', async () => {
        const chaos = new ChaosEngine();
        const system = await LiminalSystem.start();

        // Create partition between frontend and backend
        await chaos.createPartition({
            partition1: ['frontend', 'testing'],
            partition2: ['backend', 'database'],
            duration: 5000
        });

        // Verify detection
        await sleep(1000);
        const alerts = system.getAlerts();
        expect(alerts).toContainEqual(
            expect.objectContaining({
                type: 'NetworkPartition',
                affected: expect.arrayContaining(['frontend', 'backend'])
            })
        );

        // Verify continued operation in partitions
        const frontend = system.getAgent('frontend');
        const testing = system.getAgent('testing');

        // Agents in same partition can communicate
        const msg = await frontend.sendMessage(testing.id, 'test');
        expect(msg.delivered).toBe(true);

        // Cross-partition messages queued
        const backend = system.getAgent('backend');
        const crossMsg = await frontend.sendMessage(backend.id, 'cross');
        expect(crossMsg.queued).toBe(true);

        // Wait for partition heal
        await sleep(5000);

        // Verify queued messages delivered
        expect(backend.getMessages()).toContainEqual(
            expect.objectContaining({ content: 'cross' })
        );
    });

    it('should handle random agent failures', async () => {
        const chaos = new ChaosEngine();
        const system = await LiminalSystem.start();

        // Kill random agents periodically
        const killer = chaos.randomKiller({
            probability: 0.1,  // 10% chance per second
            interval: 1000,
            excludeDirector: true
        });

        killer.start();

        // System should remain operational
        const operations = [];
        for (let i = 0; i < 100; i++) {
            operations.push(system.executeTask({
                type: 'simple',
                timeout: 5000
            }));
        }

        // Stop chaos after 10 seconds
        await sleep(10000);
        killer.stop();

        // Most operations should succeed despite failures
        const results = await Promise.allSettled(operations);
        const succeeded = results.filter(r => r.status === 'fulfilled').length;

        expect(succeeded).toBeGreaterThan(80); // 80% success rate
    });
});
```

## Test Utilities

### Performance Measurement Utilities

```typescript
class PerformanceMonitor {
    private metrics: Map<string, number[]> = new Map();

    measure<T>(name: string, fn: () => T): T {
        const start = performance.now();
        const result = fn();
        const duration = performance.now() - start;

        if (!this.metrics.has(name)) {
            this.metrics.set(name, []);
        }
        this.metrics.get(name)!.push(duration);

        return result;
    }

    async measureAsync<T>(name: string, fn: () => Promise<T>): Promise<T> {
        const start = performance.now();
        const result = await fn();
        const duration = performance.now() - start;

        if (!this.metrics.has(name)) {
            this.metrics.set(name, []);
        }
        this.metrics.get(name)!.push(duration);

        return result;
    }

    getStats(name: string): PerformanceStats {
        const values = this.metrics.get(name) || [];
        return {
            count: values.length,
            min: Math.min(...values),
            max: Math.max(...values),
            avg: average(values),
            p50: calculatePercentile(values, 50),
            p99: calculatePercentile(values, 99),
            p999: calculatePercentile(values, 99.9)
        };
    }
}
```

### Test Agent Builder

```typescript
class TestAgentBuilder {
    private config: Partial<AgentConfig> = {};

    withPriority(priority: Priority): this {
        this.config.priority = priority;
        return this;
    }

    withContext(size: number): this {
        this.config.contextSize = size;
        return this;
    }

    withLeases(resources: string[]): this {
        this.config.initialLeases = resources;
        return this;
    }

    withBehavior(behavior: AgentBehavior): this {
        this.config.behavior = behavior;
        return this;
    }

    async build(): Promise<TestAgent> {
        return TestAgent.create(this.config);
    }
}

// Usage
const agent = await new TestAgentBuilder()
    .withPriority(Priority.Blocking)
    .withContext(8192)
    .withLeases(['src/api/*'])
    .withBehavior(AgentBehavior.Aggressive)
    .build();
```

<!-- [[EDITOR_NOTE_CODEX: Please ensure test scenarios are concise while maintaining coverage completeness.]] -->

## Success Criteria Summary

### Performance Targets
- ✅ Message throughput: 1000+ msg/sec
- ✅ Routing latency p99: <1ms
- ✅ Clone spawn time: <10ms average, <15ms p99
- ✅ Territory lookup: O(1) complexity, <0.1ms
- ✅ Deadlock detection: <2s
- ✅ Recovery from crash: <5s

### Reliability Targets
- ✅ Zero message loss under normal operation
- ✅ 80%+ success rate under chaos conditions
- ✅ Graceful degradation under overload
- ✅ Automatic recovery from failures

### Functional Coverage
- ✅ All priority levels tested
- ✅ Lease negotiation paths covered
- ✅ Clone lifecycle validated
- ✅ Consensus mechanisms verified
- ✅ Escalation paths tested
- ✅ End-to-end feature implementation

---

These comprehensive integration tests ensure LIMINAL meets all requirements for performance, reliability, and functionality. The test suite provides confidence that the system will operate correctly under both normal and adverse conditions.