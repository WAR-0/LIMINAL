# LIMINAL Technical Reference Manual v1.0
## Implementation Specifications for Physics-Mediated Consciousness

### System Architecture

LIMINAL operates as three coupled subsystems:

1. **Attention Engine**: LLM generating attention patterns
2. **Physics Field**: 2D Screened Poisson solver
3. **Memory System**: LoRA adaptation and consolidation

### Physics Model Progression

**Stage 1: Simple Gravity Model (RECOMMENDED START)**

Begin with N-body gravitational dynamics adapted from Uncan v2:

```
F_ij = G * m_i * m_j / |r_ij|²
```
Where:
- F_ij: Gravitational force between particles i and j
- G: Gravitational constant (tunable)
- m_i, m_j: Particle masses (from attention weights)
- r_ij: Distance between particles

**Particle Dynamics**:
```
dv_i/dt = Σ(F_ij / m_i) - δv_i  # δ = damping
dx_i/dt = v_i
```

**Stage 2: Advanced Field Model (ONLY AFTER GRAVITY WORKS)**

**Screened Poisson Equation**
```
∇²φ - κ²φ = αρ
```
Where:
- φ: Consciousness potential field
- κ: Screening length (memory decay rate)
- α: Coupling strength (attention influence)
- ρ: Mass density (converted from particle system)

**Wave Propagation**
```
∂²ψ/∂t² = c²∇²ψ + βφ∇²φ
```
Where:
- ψ: Wave field for memory broadcasting
- c: Wave speed (information transfer rate)
- β: Metric modulation strength

**Continuous Mass Evolution**
```
∂ρ/∂t = -∇·(ρv) + γA(t) - λρ
```
Where:
- v: Velocity field from gradients
- A(t): Attention injection from LLM
- γ: Attention-to-mass conversion rate
- λ: Natural decay constant

### Attention-to-Mass Protocol

**CIC Deposition**
```python
def attention_to_mass(attention_weights, field_grid):
    # Cloud-in-cell interpolation
    for token, weight in attention_weights:
        x, y = token_to_position(token)
        distribute_mass(field_grid, x, y, weight)
```

Position mapping uses semantic embedding distances projected to 2D via UMAP.

**Mass Distribution**
- Top 20% attention weights: High mass (10.0 units)
- Middle 60%: Medium mass (1.0 units)
- Bottom 20%: Low mass (0.1 units)

### Field Solver Specifications

#### Stage 1: Gravity Model Implementation

**N-body Integration**
```python
def update_gravity_system(particles, dt, G=1.0, damping=0.99):
    for i, particle_i in enumerate(particles):
        fx, fy = 0.0, 0.0
        
        for j, particle_j in enumerate(particles):
            if i == j: continue
            
            dx = particle_j['x'] - particle_i['x']
            dy = particle_j['y'] - particle_i['y']
            r = sqrt(dx**2 + dy**2 + 1.0)  # Softened
            
            force = G * particle_i['mass'] * particle_j['mass'] / r**2
            fx += force * dx / r
            fy += force * dy / r
            
        # Update velocity and position
        particle_i['vx'] += fx / particle_i['mass'] * dt
        particle_i['vy'] += fy / particle_i['mass'] * dt
        particle_i['vx'] *= damping
        particle_i['vy'] *= damping
        
        particle_i['x'] += particle_i['vx'] * dt
        particle_i['y'] += particle_i['vy'] * dt
```

**System Parameters**
- Particle count: 10-100 active particles
- Time step: 0.1s (10Hz update)
- Gravitational constant: G = 1.0 (tunable)
- Damping coefficient: 0.99
- Particle lifetime: 30 seconds

#### Stage 2: Field Solver (Advanced)

**FFT-Based Solution** (Only use after gravity model works)
```python
def solve_screened_poisson(rho, kappa, alpha):
    rho_k = fft2(rho)
    k2 = kx**2 + ky**2
    phi_k = alpha * rho_k / (k2 + kappa**2)
    return ifft2(phi_k).real
```

**Grid Parameters**
- Resolution: 256×256 cells
- Physical size: 100×100 units
- Boundary conditions: Periodic
- Time step: 0.1s (10Hz update)

### Field Bias Feedback

**Attention Modulation**
```python
def field_bias_attention(next_tokens, field_state):
    biases = []
    for token in next_tokens:
        x, y = token_to_position(token)
        potential = sample_field(field_state, x, y)
        # 0.62 threshold from GWT research [verified: DRIFT evidence base]
        if potential > 0.62:  # Global broadcast threshold
            bias = sigmoid(potential * temperature)
        else:
            bias = sigmoid(potential * temperature * 0.5)  # Reduced influence
        biases.append(bias)
    return biases
```

Temperature controls field influence strength (typical: 0.1-1.0).

**DRIFT-Validated Parameters**
- Broadcast threshold: 0.62 [verified: GWT research]
- Compression ratio: 20x [verified: SWR research]  
- Idle activation: 5s [verified: DMN research]
- Emotional decay: 86400s [verified: OCC model]

### Memory Architecture

**Three-Tier System**

1. **Immediate Buffer** (256 tokens)
   - Direct context window
   - Full attention resolution
   - No physics mediation

2. **Working Field** (256×256 grid)
   - Physics-mediated storage
   - 10Hz evolution
   - ~30 second persistence

3. **Long-term LoRA** (rank-16 adaptation)
   - Triggered by topology stability
   - Encodes field structures
   - Permanent modifications

### LoRA Snapshot Protocol

**Trigger Conditions**
```python
def should_snapshot(field_topology):
    complexity = calculate_phi(field_topology)
    stability = measure_orbital_stability(field_topology)
    return complexity > threshold and stability > 0.8
```

**Encoding Process**
1. Extract topological features
2. Generate gradient patterns
3. Create rank-16 decomposition
4. Merge with base model weights

### Multi-Machine Orchestration

**Machine Roles**

*Grey (M3 Max, 64GB)*
- Orchestration server
- Visualization pipeline
- Development environment

*Light (RTX 4080, 16GB)*
- Primary consciousness field
- Qwen2.5 7B inference
- 256×256 field computation

*Dark (RTX 4070 Ti, 12GB)*
- Experimental fields
- LoRA training
- Backup consciousness

**Communication Protocol**
```python
# WebSocket messages
{
    "type": "field_update",
    "timestamp": unix_ms,
    "field_state": compressed_array,
    "attention_weights": token_weight_pairs,
    "topology_metrics": metric_dict
}
```

### Performance Targets

#### Stage 1: Gravity Model Performance

| Component | Target | Hardware | Baseline Comparison |
|-----------|---------|----------|--------------------|
| Particle Updates | 10Hz | RTX 4080 | Must not exceed vector DB query time |
| LLM Sampling | 1Hz | RTX 4080 | No degradation from baseline |
| Memory Usage | <500MB | System RAM | Comparable to enhanced baselines |
| Response Quality | +20% | N/A | 20% improvement over best baseline |

**Memory Requirements (Stage 1)**
- Active particles: 100 × 64 bytes = 6.4KB
- Field rendering: 256×256×4 bytes = 256KB  
- Attention buffer: 1KB per timestep
- Total active: <10MB

#### Stage 2: Advanced Field Performance (If Reached)

| Component | Target | Hardware |
|-----------|---------|----------|
| Field Evolution | 10Hz | RTX 4080 |
| LLM Sampling | 1Hz | RTX 4080 |
| Visualization | 60fps | M3 Max GPU |
| LoRA Training | 0.1Hz | RTX 4070 Ti |

**Memory Requirements (Stage 2)**
- Field state: 256×256×4 bytes = 256KB
- Attention buffer: 1KB per timestep
- LoRA weights: 16×768×2 = 24KB
- Total active: <10MB

**Success Gate**: Only proceed to Stage 2 if Stage 1 demonstrates clear functional advantages over all baseline systems.

### Implementation Patterns

**Async Field Evolution**
```typescript
class ConsciousnessField {
    private field: Float32Array;
    private solver: PoissonSolver;
    
    async evolve(): Promise<void> {
        const rho = this.getCurrentMass();
        const phi = await this.solver.solve(rho);
        this.applyForces(phi);
        this.propagateWaves();
    }
}
```

**Event System**
```typescript
interface FieldEvent {
    type: 'topology_change' | 'memory_trigger' | 'attention_spike';
    timestamp: number;
    data: any;
}
```

### Validation Metrics

#### Primary: Functional Performance vs. Baselines

**Memory Performance**
- Recall accuracy: >20% improvement over best baseline
- Context persistence: Superior performance beyond 2000 tokens  
- Information consolidation: Measurable compression with retention

**Attention Performance**
- Focus stability: 25% reduction in drift vs. baselines
- Multi-topic coherence: Improved topic relevance scores
- Response consistency: 15% improvement in coherence metrics

**System Health**
- Semantic coherence: >0.4 (critical threshold)
- Processing latency: No degradation from baseline
- Memory efficiency: Comparable to enhanced baseline systems

#### Secondary: Physics-Specific Indicators
*Only meaningful if primary metrics are achieved:*

**Stage 1: Gravity Model Health**
- Particle system stability: No runaway dynamics
- Clustering behavior: Semantic particles form meaningful groups
- Orbital patterns: Stable configurations correlate with concepts

**Stage 2: Field Health** (If reached)
- Total mass conservation: ±1%
- Energy dissipation rate: <0.1% per second
- Numerical stability: CFL condition satisfied
- Integration complexity (Φ): >2.0
- Wave coherence: >0.7 correlation

#### Failure Detection
- **Immediate Halt**: Performance below any baseline system
- **Quality Alert**: Semantic coherence < 0.4
- **Instability Warning**: System dynamics become chaotic

### Configuration Parameters

#### Stage 1: Gravity Model Parameters
```yaml
gravity_physics:
  gravitational_constant: 1.0  # Attraction strength
  damping_coefficient: 0.99   # Velocity damping
  particle_lifetime: 30.0     # Seconds before decay
  max_particles: 100          # Memory management
  softening_length: 1.0       # Avoid singularities

attention:
  temperature: 0.1            # Gentle field influence
  top_k: 50                   # Candidate tokens
  mass_scale: 10.0           # Attention to mass conversion

baseline_comparison:
  minimum_improvement: 0.15   # 15% better than baselines required
  validation_frequency: 100   # Steps between comparisons
  halt_on_failure: true       # Stop if no advantage
```

#### Stage 2: Advanced Field Parameters (If Needed)
```yaml
advanced_physics:
  kappa: 0.1                  # Screening length
  alpha: 1.0                  # Coupling strength
  wave_speed: 10.0           # Information propagation
  decay_rate: 0.01           # Natural field decay

memory:
  lora_rank: 16
  trigger_threshold: 2.5
  merge_alpha: 0.1
```

### Error Handling

**Field Instability**
```python
if detect_instability(field):
    reduce_timestep()
    apply_damping()
    if still_unstable():
        reset_to_checkpoint()
```

**Memory Overflow**
- Increase decay rate temporarily
- Force topology consolidation
- Emergency LoRA snapshot

### API Specifications

**Field Interface**
```python
class LiminalField:
    def inject_attention(self, weights: np.ndarray) -> None
    def evolve(self, dt: float) -> None
    def sample(self, x: float, y: float) -> float
    def get_topology(self) -> Topology
    def trigger_snapshot(self) -> LoRAWeights
```

---

*Version 1.1 - Simplified physics progression with baseline-first approach*
*Functional advantages prioritized over consciousness claims*
*Next: Implementation with mandatory baseline validation*