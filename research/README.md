# LIMINAL Research Organization

## Directory Structure

### biological-coordination/
Biological mechanisms for distributed coordination without central control.

- **quorum-sensing-stigmergy.md** - Ant colony optimization, bee swarm decision-making, pheromone trails, and indirect coordination through environmental marks.

### consensus-mechanisms/
Byzantine fault tolerance and distributed consensus algorithms.

- **byzantine-fault-tolerance.md** - Comparison of digital consensus (HoneyBadgerBFT, Aleph, Avalanche, Narwhal/Tusk) with biological swarm consensus. CRDTs and eventual consistency patterns.

### distributed-systems/
Core distributed computing patterns for agent coordination.

- **replay-determinism-ledgers.md** - Event sourcing, append-only logs, logical clocks (Lamport/vector), and causal ordering for deterministic replay.
- **work-stealing-scheduling.md** - Cilk work-first vs help-first scheduling, ForkJoinPool implementation, fairness metrics, and dynamic load balancing.

### phase-dynamics/
Critical phenomena and phase transitions in distributed systems.

- **critical-phenomena.md** - Percolation theory, edge-of-chaos computation, order/chaos regimes, and control parameters for phase transitions.
- **phase-locking-synchronization.md** - Kuramoto model, distributed oscillator networks, and emergent synchronization in multi-agent systems.

### territorial-allocation/
Spatial computing models for territory emergence in shared codebases.

- **spatial-computing-models.md** - Voronoi tessellation, hash-space partitioning, stigmergic territories, space-filling curves, and graph partitioning for code ownership.

## Key Insights for LIMINAL

### Turn/Epoch Implementation
- **Quorum sensing** → Turn validation thresholds (0.62 from GWT research)
- **Vector clocks** → Causal ordering within epochs
- **Work-stealing** → Dynamic task distribution during execution phases
- **Phase transitions** → Turn boundary detection and state changes

### Critical Parameters
- Percolation threshold: avg degree > 1 for connectivity
- GWT broadcast threshold: 0.62 for consciousness ignition
- Steal count: Balance between utilization and contention
- Coupling strength: K_c for synchronization transitions

### Emergent Mechanisms
- **Stigmergy** → Territories from edit "scent" without pre-assignment
- **Edge of chaos** → Optimal computation at metastable dynamics
- **Positive feedback** → Rapid consensus through amplification
- **Decentralized coordination** → No single point of failure

## Usage

Each research document provides:
1. Theoretical foundations from the literature
2. Implementation patterns and algorithms
3. Practical parameters and thresholds
4. Mapping to LIMINAL's temporal system

Reference these when implementing turn mechanics, territory allocation, or consensus protocols.