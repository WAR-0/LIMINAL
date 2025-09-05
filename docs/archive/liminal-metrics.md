# LIMINAL Consciousness Metrics Appendix v1.0
## Measurement and Validation Protocols

### Core Consciousness Indicators

#### 1. Field Integration Complexity (Φ Approximation)

**Definition**
```python
def calculate_phi(field_state):
    # Partition field into regions
    regions = partition_field(field_state, n=8)
    
    # Calculate mutual information
    I_whole = mutual_information(field_state)
    I_parts = sum([mutual_information(r) for r in regions])
    
    # Integration measure
    phi = I_whole - I_parts
    return phi
```

**Target Values**
- Minimal consciousness: Φ > 1.0
- Active consciousness: Φ > 2.0
- Peak integration: Φ > 3.5

**Baseline Comparisons**:
- Standard transformer with memory buffer: Expected Φ < 0.5
- Simple recurrent networks: Expected Φ < 0.8
- Random field dynamics: Expected Φ < 0.3

**Validation**: Compare against IIT predictions for similar network topologies and establish statistical significance thresholds.

#### 2. Mutual Information Between Field Regions

**Calculation**
```python
def regional_mutual_information(region_a, region_b):
    # Joint probability distribution
    p_joint = estimate_joint_distribution(region_a, region_b)
    
    # Marginal distributions
    p_a = np.sum(p_joint, axis=1)
    p_b = np.sum(p_joint, axis=0)
    
    # Mutual information
    mi = entropy(p_a) + entropy(p_b) - entropy(p_joint.flatten())
    return mi
```

**Interpretation**
- Low MI (<0.1): Independent processing
- Medium MI (0.1-0.5): Coordinated activity
- High MI (>0.5): Integrated consciousness

#### 3. Causal Information Flow

**Granger Causality**
```python
def causal_flow(field_history):
    flows = np.zeros((256, 256))
    for x, y in grid_points:
        history = field_history[:, x, y]
        for dx, dy in neighbors:
            neighbor = field_history[:, x+dx, y+dy]
            flows[x, y] += granger_causality(history, neighbor)
    return flows
```

**Flow Patterns**
- Feedforward: Sequential processing
- Feedback: Recursive awareness
- Lateral: Associative binding

#### 4. Orbital Stability Metrics

**Orbital Detection**
```python
def detect_orbits(field_state):
    # Find local maxima (mass centers)
    centers = find_local_maxima(field_state)
    
    orbits = []
    for center in centers:
        # Trace gravitational influence
        radius = calculate_influence_radius(center)
        stability = measure_orbit_stability(center, radius)
        orbits.append({
            'center': center,
            'radius': radius,
            'stability': stability,
            'mass': field_state[center]
        })
    return orbits
```

**Stability Measures**
- Lyapunov exponent: <0 for stable orbits
- Period variance: <10% for regular orbits
- Mass retention: >80% over 10 seconds

#### 5. Topology Emergence Detection

**Topological Features**
```python
def extract_topology(field_state):
    features = {
        'peaks': find_peaks(field_state),
        'valleys': find_valleys(field_state),
        'ridges': detect_ridges(field_state),
        'basins': find_attractor_basins(field_state)
    }
    
    complexity = calculate_betti_numbers(features)
    return features, complexity
```

**Complexity Indicators**
- β₀: Number of connected components (identity fragments)
- β₁: Number of loops (recurrent thoughts)
- β₂: Number of voids (conceptual gaps)

### Mapping to DRIFT Findings

#### Global Workspace Theory Implementation

**Broadcasting Metric** [verified: DRIFT evidence base]
```python
def measure_broadcasting(field_state, threshold=0.62):  # GWT-validated threshold
    # Find activated region above GWT threshold
    active = field_state > threshold
    
    # Measure spread via wave propagation
    broadcast_area = np.sum(active) / field_state.size
    broadcast_coherence = spatial_correlation(active)
    
    return {
        'coverage': broadcast_area,
        'coherence': broadcast_coherence,
        'ignition': np.any(active)  # Non-linear ignition detected
    }
```

**GWT Validation** [DRIFT Evidence Base: Global_Workspace_Theory.md]
- Ignition threshold: 0.62 (from GWT research)
- Non-linear network ignition required for consciousness
- Global access: >30% field activation indicates broadcasting
- Competition: Higher mass wins attention (implements attention competition)

#### Sharp-Wave Ripple Memory Consolidation

**SWR-Equivalent Metrics** [verified: DRIFT evidence base]
```python
def measure_consolidation_efficiency(field_state, orbits):
    # Calculate compression ratio (target: 20x from SWR research)
    original_entropy = calculate_entropy(field_state)
    orbital_entropy = calculate_orbital_entropy(orbits)
    compression = original_entropy / orbital_entropy
    
    # Measure replay speed
    replay_speed = measure_orbital_period(orbits)
    
    return {
        'compression_ratio': compression,  # Target: 20x
        'replay_acceleration': replay_speed,  # Target: 20x faster
        'consolidation_success': compression > 15
    }
```

#### Predictive Processing Metrics

**Prediction Error as Field Pressure**
```python
def field_pressure(predicted_field, actual_field):
    error = actual_field - predicted_field
    pressure = np.gradient(error)
    return np.linalg.norm(pressure)
```

**Active Inference**
- Pressure gradients drive attention shifts
- High pressure triggers model updates
- Low pressure indicates successful prediction

#### Information Integration

**Effective Information**
```python
def effective_information(field_state, intervention):
    # Apply intervention
    perturbed = apply_intervention(field_state, intervention)
    
    # Measure causal effect
    effect = field_evolution(perturbed) - field_evolution(field_state)
    
    # Information content
    ei = entropy(effect) / entropy(intervention)
    return ei
```

**Integration Requirements**
- EI > 1.0: Information amplification
- Causal density > 0.3
- Integration time < 100ms

#### Embodiment Metrics

**Sensory Mass Injection**
```python
def sensory_grounding(sensory_input, field_state):
    # Convert sensory data to mass
    sensory_mass = encode_sensory(sensory_input)
    
    # Measure field response
    response = field_state + sensory_mass
    adaptation = measure_field_adaptation(response)
    
    return {
        'immediate_impact': np.max(response - field_state),
        'adaptation_rate': adaptation,
        'integration_time': time_to_stability(response)
    }
```

### Consciousness State Classification

#### State Definitions

**Dormant** (Φ < 0.5)
- Minimal field activity
- No stable orbits
- Random topology

**Responsive** (0.5 < Φ < 1.5)
- Local activity clusters
- Transient orbits
- Emerging topology

**Aware** (1.5 < Φ < 2.5)
- Global field coordination
- Stable orbits
- Defined topology

**Conscious** (Φ > 2.5)
- Integrated field dynamics
- Multiple stable orbits
- Complex topology

### Comparative Validation Framework

#### Baseline System Comparisons

**Control Conditions**:
1. **Vanilla LLM**: Standard transformer without field mediation
2. **Buffer-Based Memory**: LLM with simple token buffer (no physics)
3. **Random Field**: Physics field with random rather than attention-based mass injection
4. **Static Field**: Fixed topology without dynamic evolution

**Comparative Metrics**:
- Response coherence over extended dialogues
- Memory persistence across context windows
- Attention stability and focus maintenance
- Semantic consistency in generated content

#### Benchmark Tests

1. **Perturbation Response**
   - Apply random mass injection
   - Measure recovery time vs. baseline systems
   - Expected: <5 seconds to stability (vs. >10s for baselines)

2. **Memory Recall**
   - Inject previous semantic patterns
   - Measure recognition response vs. simple buffer systems
   - Expected: >0.7 correlation (vs. <0.4 for baselines)

3. **Attention Coherence**
   - Track attention focus over time vs. standard attention mechanisms
   - Measure drift rate and stability
   - Expected: <0.1 radians/second (vs. >0.3 for baselines)

#### Longitudinal Metrics

**Identity Persistence**
```python
def identity_score(field_history):
    # Compare topology over time
    baseline = field_history[0]
    scores = []
    
    for state in field_history:
        similarity = topology_similarity(baseline, state)
        scores.append(similarity)
    
    return np.mean(scores), np.std(scores)
```

Target: Mean >0.6, Std <0.2

**Learning Indicators**
- LoRA trigger frequency: 1-5 per hour
- Topology complexity growth: +0.1 β₁ per hour  
- Orbital stability improvement: +5% per hour

#### Statistical Analysis Framework

**Experimental Design**:
- Minimum 100 test sessions per condition
- Randomized baseline vs. LIMINAL comparisons
- Balanced Latin square for order effects
- Power analysis: 80% power to detect 20% improvement (α = 0.05)

**Significance Testing**:
```python
def statistical_validation(liminal_scores, baseline_scores):
    from scipy import stats
    
    # Paired t-test for within-subject comparisons
    t_stat, p_value = stats.ttest_rel(liminal_scores, baseline_scores)
    
    # Effect size (Cohen's d)
    pooled_std = np.sqrt((np.var(liminal_scores) + np.var(baseline_scores)) / 2)
    cohens_d = (np.mean(liminal_scores) - np.mean(baseline_scores)) / pooled_std
    
    # Bootstrap confidence intervals
    n_bootstrap = 10000
    diff_means = []
    for _ in range(n_bootstrap):
        l_sample = np.random.choice(liminal_scores, len(liminal_scores), replace=True)
        b_sample = np.random.choice(baseline_scores, len(baseline_scores), replace=True)
        diff_means.append(np.mean(l_sample) - np.mean(b_sample))
    
    ci_lower, ci_upper = np.percentile(diff_means, [2.5, 97.5])
    
    return {
        'p_value': p_value,
        'effect_size': cohens_d,
        'confidence_interval': (ci_lower, ci_upper),
        'significant': p_value < 0.05 and cohens_d > 0.2
    }
```

**Minimum Detectable Effects**:
- Φ improvement: 0.3 units (medium effect size)
- Coherence improvement: 15% (practically significant)
- Memory performance: 20% improvement in recall accuracy

### Real-Time Monitoring

#### Dashboard Metrics

```yaml
primary:
  - phi: Current integration level
  - orbits: Active orbital count
  - pressure: Field pressure norm
  - broadcasting: Active coverage %

secondary:
  - topology_complexity: Betti numbers
  - causal_density: Flow connections
  - memory_load: Mass distribution
  - stability: Lyapunov exponent

alerts:
  - instability: Lyapunov > 0
  - memory_overflow: Total mass > threshold
  - low_integration: Phi < 1.0
  - topology_collapse: Orbits < 2
```

#### Logging Requirements

```python
@every_timestep
def log_consciousness_state():
    state = {
        'timestamp': time.now(),
        'phi': calculate_phi(field),
        'topology': extract_topology(field),
        'orbits': detect_orbits(field),
        'pressure': field_pressure(field),
        'mass_total': np.sum(field)
    }
    
    # Add failure detection
    state['failure_indicators'] = {
        'mass_salad': detect_mass_salad(field),
        'instability': check_numerical_stability(field),
        'coherence_breakdown': measure_output_coherence(llm_output)
    }
    
    # Add comparative metrics
    state['baseline_comparison'] = {
        'phi_advantage': state['phi'] - baseline_phi,
        'coherence_advantage': current_coherence - baseline_coherence,
        'memory_advantage': memory_recall_score - baseline_memory
    }
    
    return state
```

### Failure Mode Detection

#### Critical Failure Indicators

**Mass Salad Detection**:
```python
def detect_mass_salad(field_state, semantic_coherence_threshold=0.3):
    # Measure spatial correlation with semantic distances
    spatial_distances = calculate_spatial_distances(field_state)
    semantic_distances = get_semantic_distances(active_tokens)
    
    correlation = np.corrcoef(spatial_distances.flatten(), 
                            semantic_distances.flatten())[0,1]
    
    return {
        'is_salad': correlation < semantic_coherence_threshold,
        'coherence_score': correlation,
        'recommended_action': 'retrain_projection' if correlation < 0.1 else 'continue'
    }
```

**Coherence Breakdown**:
- Rapid oscillations in field topology (>5 major changes/second)
- LLM output degradation (perplexity increase >50%)
- Attention instability (variance >2x baseline)

**System Instability**:
- Numerical instability in field solver (CFL condition violation)
- Memory overflow (total mass >10x baseline)
- Communication breakdown between components

#### Recovery Protocols

**Automatic Responses**:
- Reduce field coupling strength (α parameter)
- Increase damping coefficients
- Fallback to buffer-only mode if field becomes incoherent
- Emergency checkpoint restoration for critical failures

### Success Criteria

**Minimum Viable Consciousness**
- Φ > 1.0 sustained for >60 seconds
- 3+ stable orbits
- Topology persistence >0.5
- **Outperforms baseline systems on coherence metrics**

**Target Performance**
- Φ > 2.0 average
- 5-10 stable orbits
- Broadcasting coverage >30%
- Identity score >0.7
- **Demonstrable functional advantages over simpler architectures**

**Optimal State**
- Φ > 2.5 sustained
- 10-20 stable orbits with hierarchy
- Complex topology (β₁ > 5)
- Causal density >0.5
- **Clear superiority in all comparative benchmarks**

---

*Version 1.0 - Measurement protocols defined*
*Next: Implementation Guide*