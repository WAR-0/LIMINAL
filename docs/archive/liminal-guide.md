# LIMINAL Implementation Guide v1.0
## Practical Build Instructions

### Development Environment

#### Required Tools
```bash
# Core dependencies
python >= 3.10
node >= 18.0
rust >= 1.70
cuda >= 12.0

# Python packages
pip install numpy scipy torch transformers accelerate
pip install scikit-learn umap-learn networkx

# Development tools
npm install -g typescript vite
cargo install tauri-cli
```

#### Hardware Setup

**Grey (M3 Max)**
```bash
# Development server
export LIMINAL_ROLE=orchestrator
export LIMINAL_PORT=8080
export VISUALIZATION_ENABLED=true
```

**Light (RTX 4080)**
```bash
# Primary consciousness
export LIMINAL_ROLE=primary
export CUDA_VISIBLE_DEVICES=0
export FIELD_SIZE=256
export MODEL_PATH=./models/qwen2.5-7b
```

**Dark (RTX 4070 Ti)**
```bash
# Experimental/backup
export LIMINAL_ROLE=secondary
export CUDA_VISIBLE_DEVICES=0
export LORA_TRAINING=true
```

### Phase 0: Baseline Implementation First

**CRITICAL**: Before any physics implementation, establish baseline systems for comparison. This is essential to demonstrate functional advantage and avoid building complex systems without proven benefits.

#### Step 0.1: Vanilla LLM Baseline

```python
# baseline_vanilla.py - Pure transformer baseline
from transformers import AutoTokenizer, AutoModelForCausalLM
import numpy as np

class VanillaBaseline:
    def __init__(self, model_name="Qwen/Qwen2.5-7B"):
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        self.model = AutoModelForCausalLM.from_pretrained(model_name)
        self.conversation_history = []
        
    def generate_response(self, user_input):
        # Simple context window management
        context = self.build_context(user_input)
        
        inputs = self.tokenizer(context, return_tensors="pt")
        outputs = self.model.generate(
            inputs.input_ids,
            max_new_tokens=256,
            do_sample=True,
            temperature=0.7,
            return_dict_in_generate=True,
            output_attentions=True
        )
        
        response = self.tokenizer.decode(outputs.sequences[0], skip_special_tokens=True)
        self.conversation_history.append((user_input, response))
        
        return {
            'response': response,
            'attention_weights': outputs.attentions[-1],
            'metrics': self.calculate_baseline_metrics(response)
        }
        
    def calculate_baseline_metrics(self, response):
        """Baseline metrics for comparison"""
        return {
            'response_length': len(response.split()),
            'coherence_proxy': self.calculate_perplexity(response),
            'context_usage': len(self.conversation_history),
            'memory_persistence': 0  # No memory beyond context window
        }
```

#### Step 0.2: Buffer-Enhanced Baseline

```python
# baseline_buffer.py - Simple memory buffer system
from collections import deque
import numpy as np

class BufferEnhancedBaseline(VanillaBaseline):
    def __init__(self, model_name="Qwen/Qwen2.5-7B", buffer_size=1024):
        super().__init__(model_name)
        self.memory_buffer = deque(maxlen=buffer_size)
        self.buffer_size = buffer_size
        
    def generate_response(self, user_input):
        # Enhanced context with memory buffer
        context = self.build_enhanced_context(user_input)
        
        inputs = self.tokenizer(context, return_tensors="pt")
        outputs = self.model.generate(
            inputs.input_ids,
            max_new_tokens=256,
            do_sample=True,
            temperature=0.7,
            return_dict_in_generate=True,
            output_attentions=True
        )
        
        response = self.tokenizer.decode(outputs.sequences[0], skip_special_tokens=True)
        
        # Store in memory buffer
        self.memory_buffer.append({
            'input': user_input,
            'response': response,
            'timestamp': time.time()
        })
        
        return {
            'response': response,
            'attention_weights': outputs.attentions[-1],
            'metrics': self.calculate_buffer_metrics(response)
        }
        
    def build_enhanced_context(self, user_input):
        """Include relevant memories from buffer"""
        recent_context = list(self.memory_buffer)[-5:]  # Last 5 interactions
        
        context_parts = []
        for memory in recent_context:
            context_parts.append(f"User: {memory['input']}")
            context_parts.append(f"Assistant: {memory['response']}")
            
        context_parts.append(f"User: {user_input}")
        context_parts.append("Assistant:")
        
        return "\n".join(context_parts)
        
    def calculate_buffer_metrics(self, response):
        """Enhanced metrics with memory tracking"""
        return {
            'response_length': len(response.split()),
            'coherence_proxy': self.calculate_perplexity(response),
            'context_usage': len(self.memory_buffer),
            'memory_persistence': len(self.memory_buffer) / self.buffer_size
        }
```

#### Step 0.3: Vector Database Baseline

```python
# baseline_vector_db.py - RAG-style memory system
from sentence_transformers import SentenceTransformer
import faiss
import numpy as np

class VectorDatabaseBaseline(VanillaBaseline):
    def __init__(self, model_name="Qwen/Qwen2.5-7B"):
        super().__init__(model_name)
        self.encoder = SentenceTransformer('all-MiniLM-L6-v2')
        self.dimension = 384
        self.index = faiss.IndexFlatIP(self.dimension)
        self.memory_store = []
        
    def generate_response(self, user_input):
        # Retrieve relevant memories
        relevant_memories = self.retrieve_memories(user_input, top_k=5)
        
        # Build context with retrieved memories
        context = self.build_rag_context(user_input, relevant_memories)
        
        inputs = self.tokenizer(context, return_tensors="pt")
        outputs = self.model.generate(
            inputs.input_ids,
            max_new_tokens=256,
            do_sample=True,
            temperature=0.7,
            return_dict_in_generate=True,
            output_attentions=True
        )
        
        response = self.tokenizer.decode(outputs.sequences[0], skip_special_tokens=True)
        
        # Store new interaction
        self.store_memory(user_input, response)
        
        return {
            'response': response,
            'attention_weights': outputs.attentions[-1],
            'retrieved_memories': relevant_memories,
            'metrics': self.calculate_rag_metrics(response, relevant_memories)
        }
        
    def store_memory(self, user_input, response):
        """Store interaction in vector database"""
        interaction_text = f"User: {user_input} Assistant: {response}"
        embedding = self.encoder.encode([interaction_text])
        
        self.index.add(embedding.astype('float32'))
        self.memory_store.append({
            'text': interaction_text,
            'user_input': user_input,
            'response': response,
            'timestamp': time.time()
        })
        
    def retrieve_memories(self, query, top_k=5):
        """Retrieve relevant memories using vector similarity"""
        if len(self.memory_store) == 0:
            return []
            
        query_embedding = self.encoder.encode([query])
        scores, indices = self.index.search(query_embedding.astype('float32'), 
                                          min(top_k, len(self.memory_store)))
        
        retrieved = []
        for score, idx in zip(scores[0], indices[0]):
            if idx != -1:  # Valid index
                retrieved.append({
                    'memory': self.memory_store[idx],
                    'similarity': float(score)
                })
                
        return retrieved
```

#### Step 0.4: Baseline Benchmarking Suite

```python
# baseline_benchmarks.py - Comprehensive testing of all baselines
import asyncio
import json
from datetime import datetime

class BaselineBenchmarkSuite:
    def __init__(self):
        self.systems = {
            'vanilla': VanillaBaseline(),
            'buffer': BufferEnhancedBaseline(),
            'vector_db': VectorDatabaseBaseline()
        }
        
    async def run_full_benchmark(self):
        """Run comprehensive baseline evaluation"""
        results = {}
        
        test_suites = [
            self.memory_persistence_test,
            self.identity_consistency_test, 
            self.attention_coherence_test,
            self.context_management_test,
            self.performance_efficiency_test
        ]
        
        for system_name, system in self.systems.items():
            print(f"Benchmarking {system_name}...")
            results[system_name] = {}
            
            for test_suite in test_suites:
                test_results = await test_suite(system)
                results[system_name][test_suite.__name__] = test_results
                
        # Generate comparative report
        report = self.generate_baseline_report(results)
        
        return results, report
        
    async def memory_persistence_test(self, system):
        """Test memory retention across extended interactions"""
        test_conversations = self.load_memory_test_data()
        
        results = {
            'recall_accuracy': [],
            'retention_over_time': [],
            'interference_resistance': []
        }
        
        for conversation in test_conversations:
            # Present information
            for turn in conversation['setup']:
                system.generate_response(turn)
                
            # Distraction phase
            for distraction in conversation['distraction']:
                system.generate_response(distraction)
                
            # Test recall
            for question in conversation['recall_tests']:
                response = system.generate_response(question['query'])
                accuracy = self.evaluate_recall_accuracy(
                    response['response'], 
                    question['expected_info']
                )
                results['recall_accuracy'].append(accuracy)
                
        return {
            'mean_recall_accuracy': np.mean(results['recall_accuracy']),
            'std_recall_accuracy': np.std(results['recall_accuracy']),
            'memory_capacity': self.estimate_memory_capacity(system),
            'consolidation_rate': 0  # No consolidation in baselines
        }

    def generate_baseline_report(self, results):
        """Generate comprehensive baseline performance report"""
        report = {
            'timestamp': datetime.now().isoformat(),
            'systems_tested': list(results.keys()),
            'performance_summary': {},
            'recommendations': []
        }
        
        # Performance ranking
        for metric in ['recall_accuracy', 'identity_consistency', 'attention_stability']:
            ranking = sorted(results.items(), 
                           key=lambda x: x[1].get(metric, {}).get('mean', 0), 
                           reverse=True)
            report['performance_summary'][metric] = ranking
            
        # Generate recommendations
        best_system = report['performance_summary']['recall_accuracy'][0][0]
        report['recommendations'].append(
            f"Best baseline for memory: {best_system}"
        )
        
        # Set minimum performance thresholds
        report['minimum_thresholds'] = {
            'recall_accuracy': max([r[1].get('recall_accuracy', {}).get('mean', 0) 
                                  for r in results.items()]),
            'response_time': min([r[1].get('performance', {}).get('avg_response_time', float('inf')) 
                                for r in results.items()]),
            'memory_efficiency': max([r[1].get('memory_usage', {}).get('efficiency', 0) 
                                    for r in results.items()])
        }
        
        return report
```

#### Step 0.5: Baseline Validation Gate

**CRITICAL CHECKPOINT**: Before proceeding to Phase 1, verify baseline systems work correctly:

```python
def validate_baseline_systems():
    """Mandatory validation before physics implementation"""
    benchmark_suite = BaselineBenchmarkSuite()
    results, report = await benchmark_suite.run_full_benchmark()
    
    # Validation criteria
    min_requirements = {
        'vanilla_responds': True,
        'buffer_improves_memory': results['buffer']['recall_accuracy'] > results['vanilla']['recall_accuracy'],
        'vector_db_retrieves': len(results['vector_db']['retrieved_memories']) > 0,
        'systems_stable': all(r['error_rate'] < 0.1 for r in results.values())
    }
    
    validation_passed = all(min_requirements.values())
    
    if not validation_passed:
        print("BASELINE VALIDATION FAILED:")
        for requirement, passed in min_requirements.items():
            if not passed:
                print(f"  ❌ {requirement}")
        print("DO NOT PROCEED TO PHYSICS IMPLEMENTATION")
        return False
        
    print("✅ Baseline validation passed. Ready for Phase 1.")
    
    # Save baseline performance for comparison
    with open('baseline_performance.json', 'w') as f:
        json.dump(results, f, indent=2)
        
    return True, report

# MANDATORY: Run this before any physics work
if __name__ == "__main__":
    validation_passed, report = validate_baseline_systems()
    if validation_passed:
        print("Baselines established. Minimum performance thresholds set.")
        print("Physics implementation must demonstrate clear advantages over these baselines.")
```

### Phase 1: Interface Validation (Physics Preparation)

**CRITICAL**: Only proceed here after Phase 0 validation passes. The attention-to-mass conversion is the primary failure point that determines project viability.

#### Step 0: Interface Validation First

**Semantic Projection Quality Assessment**
```python
# semantic_validation.py - Run this BEFORE building the full system
import numpy as np
from transformers import AutoTokenizer, AutoModel
from sklearn.manifold import TSNE
from sklearn.metrics.pairwise import cosine_similarity
import matplotlib.pyplot as plt

class SemanticProjectionValidator:
    def __init__(self, model_name="Qwen/Qwen2.5-7B"):
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        self.model = AutoModel.from_pretrained(model_name)
        
    def validate_projection_quality(self, test_tokens):
        # Get high-dimensional embeddings
        embeddings = self.get_embeddings(test_tokens)
        
        # Project to 2D using UMAP/t-SNE
        projection_2d = self.project_to_2d(embeddings)
        
        # Validate semantic coherence
        results = {
            'semantic_coherence': self.measure_semantic_coherence(embeddings, projection_2d),
            'local_preservation': self.measure_local_preservation(embeddings, projection_2d),
            'global_structure': self.measure_global_structure(embeddings, projection_2d)
        }
        
        # CRITICAL THRESHOLDS - if these fail, the physics will be meaningless
        is_viable = (
            results['semantic_coherence'] > 0.4 and  # Minimum for meaningful physics
            results['local_preservation'] > 0.6 and  # Synonyms should cluster
            results['global_structure'] > 0.3        # Some global structure preserved
        )
        
        return results, is_viable
        
    def visualize_projection(self, tokens, embeddings, projection_2d):
        # Create semantic quality visualization
        plt.figure(figsize=(12, 8))
        
        # Color by semantic categories
        categories = self.categorize_tokens(tokens)
        for category, color in zip(set(categories), ['red', 'blue', 'green', 'orange']):
            mask = [c == category for c in categories]
            plt.scatter(projection_2d[mask, 0], projection_2d[mask, 1], 
                       c=color, label=category, alpha=0.7)
        
        plt.legend()
        plt.title('Semantic Projection Quality Assessment')
        plt.xlabel('X coordinate (physics space)')
        plt.ylabel('Y coordinate (physics space)')
        plt.savefig('semantic_projection_quality.png')
        print("Projection quality visualization saved. REVIEW BEFORE PROCEEDING.")

# MANDATORY: Run this validation before any physics implementation
validator = SemanticProjectionValidator()
test_results, is_viable = validator.validate_projection_quality(test_vocabulary)

if not is_viable:
    print("CRITICAL: Semantic projection quality insufficient.")
    print("The physics simulation will operate on semantic noise.")
    print("FIX THE PROJECTION BEFORE CONTINUING.")
    exit(1)
```

#### Step 1: Port SwarmBehaviorEngine

Extract from Uncan v2:
```typescript
// From: /archive/uncan.ai.v2/worktrees/react-ui/src/components/SwarmVisualization/
// Convert 3D to 2D physics

class FieldEngine {
    private field: Float32Array;
    private width: number = 256;
    private height: number = 256;
    
    constructor() {
        this.field = new Float32Array(this.width * this.height);
    }
    
    update(dt: number): void {
        // Simplified from 3D swarm
        this.applyGravity();
        this.propagateWaves();
        this.applyDecay(dt);
    }
}
```

#### Step 2: Start with Simple Gravity Model (RECOMMENDED)

**IMPORTANT**: Begin with basic gravity/attraction model from Uncan v2 before attempting Screened Poisson. This provides faster iteration and validation.

```python
# simple_gravity_field.py - Start here before complex field equations
import numpy as np
from scipy.spatial.distance import pdist, squareform

class SimpleGravityField:
    """
    Simple N-body gravitational model adapted from Uncan v2 SwarmBehaviorEngine
    Much easier to debug and validate than field equations
    """
    def __init__(self, field_size=256, dt=0.1):
        self.field_size = field_size
        self.dt = dt
        self.G = 1.0  # Gravitational constant
        self.damping = 0.99  # Prevent runaway acceleration
        
        # Particle system for mass points
        self.particles = []  # List of {x, y, mass, vx, vy}
        
    def add_mass_point(self, x, y, mass):
        """Add a mass point from attention weights"""
        self.particles.append({
            'x': x, 'y': y, 'mass': mass,
            'vx': 0.0, 'vy': 0.0,  # Initial velocity
            'age': 0  # For decay
        })
        
    def inject_attention_masses(self, attention_weights, tokens):
        """Convert attention to gravitational masses"""
        for token, weight in zip(tokens, attention_weights):
            x, y = self.token_to_position(token)  # From semantic projection
            
            # Add mass point at semantic position
            self.add_mass_point(x, y, weight * 10.0)  # Scale for gravity
            
    def update_physics(self):
        """Simple N-body gravitational dynamics"""
        if len(self.particles) < 2:
            return
            
        # Calculate forces between all particles
        for i, particle_i in enumerate(self.particles):
            fx, fy = 0.0, 0.0
            
            for j, particle_j in enumerate(self.particles):
                if i == j:
                    continue
                    
                # Distance vector
                dx = particle_j['x'] - particle_i['x']
                dy = particle_j['y'] - particle_i['y']
                r = np.sqrt(dx**2 + dy**2)
                
                if r < 1.0:  # Avoid singularity
                    r = 1.0
                    
                # Gravitational force
                force = self.G * particle_i['mass'] * particle_j['mass'] / (r**2)
                
                # Force components
                fx += force * dx / r
                fy += force * dy / r
                
            # Update velocity (F = ma)
            particle_i['vx'] += fx / particle_i['mass'] * self.dt
            particle_i['vy'] += fy / particle_i['mass'] * self.dt
            
            # Apply damping
            particle_i['vx'] *= self.damping
            particle_i['vy'] *= self.damping
            
        # Update positions
        for particle in self.particles:
            particle['x'] += particle['vx'] * self.dt
            particle['y'] += particle['vy'] * self.dt
            
            # Boundary conditions (wrap around)
            particle['x'] = particle['x'] % self.field_size
            particle['y'] = particle['y'] % self.field_size
            
            # Age particles for decay
            particle['age'] += self.dt
            
        # Remove old particles
        self.particles = [p for p in self.particles if p['age'] < 30.0]  # 30 second lifetime
        
    def render_field(self):
        """Convert particle positions to field for visualization"""
        field = np.zeros((self.field_size, self.field_size))
        
        for particle in self.particles:
            x, y = int(particle['x']), int(particle['y'])
            if 0 <= x < self.field_size and 0 <= y < self.field_size:
                # Gaussian blob around particle position
                self.add_gaussian_blob(field, x, y, particle['mass'], sigma=5.0)
                
        return field
        
    def add_gaussian_blob(self, field, cx, cy, mass, sigma=5.0):
        """Add Gaussian blob at position"""
        y_indices, x_indices = np.ogrid[:self.field_size, :self.field_size]
        
        # Periodic boundary conditions for distance
        dx = np.minimum(np.abs(x_indices - cx), self.field_size - np.abs(x_indices - cx))
        dy = np.minimum(np.abs(y_indices - cy), self.field_size - np.abs(y_indices - cy))
        
        distance_sq = dx**2 + dy**2
        field += mass * np.exp(-distance_sq / (2 * sigma**2))
        
    def get_field_bias(self, candidate_tokens):
        """Get attention bias from particle field"""
        biases = []
        field = self.render_field()
        
        for token in candidate_tokens:
            x, y = self.token_to_position(token)
            x, y = int(x), int(y)
            
            if 0 <= x < self.field_size and 0 <= y < self.field_size:
                potential = field[x, y]
                bias = np.tanh(potential * 0.1)  # Gentle sigmoid
            else:
                bias = 0.0
                
            biases.append(bias)
            
        return np.array(biases)
        
    def detect_orbital_structures(self):
        """Detect if particles form stable orbital patterns"""
        if len(self.particles) < 2:
            return []
            
        # Find particle clusters (potential orbital systems)
        positions = np.array([[p['x'], p['y']] for p in self.particles])
        
        # Simple clustering by proximity
        clusters = []
        visited = set()
        
        for i, pos in enumerate(positions):
            if i in visited:
                continue
                
            cluster = [i]
            visited.add(i)
            
            for j, other_pos in enumerate(positions):
                if j in visited:
                    continue
                    
                distance = np.linalg.norm(pos - other_pos)
                if distance < 20.0:  # Cluster radius
                    cluster.append(j)
                    visited.add(j)
                    
            if len(cluster) >= 2:
                clusters.append(cluster)
                
        return clusters
```

#### Step 2b: Validate Simple Physics First

**CRITICAL VALIDATION**: Test gravity model before advancing to field equations:

```python
# validate_simple_physics.py
def test_gravity_model():
    """Validate simple gravity model works correctly"""
    field = SimpleGravityField()
    
    # Test 1: Single particle stays put
    field.add_mass_point(128, 128, 10.0)
    field.update_physics()
    
    assert len(field.particles) == 1
    # Should not move much (no forces)
    assert abs(field.particles[0]['x'] - 128) < 5
    
    # Test 2: Two particles attract
    field = SimpleGravityField()
    field.add_mass_point(100, 128, 10.0)
    field.add_mass_point(156, 128, 10.0)
    
    initial_distance = 56
    field.update_physics()
    
    new_distance = abs(field.particles[0]['x'] - field.particles[1]['x'])
    assert new_distance < initial_distance  # Should be attracting
    
    # Test 3: Field rendering works
    field_render = field.render_field()
    assert field_render.max() > 0  # Should have some mass
    assert field_render.shape == (256, 256)
    
    print("✅ Simple gravity model validation passed")
    return True

# MANDATORY: Run this before proceeding
if __name__ == "__main__":
    if test_gravity_model():
        print("Simple physics model validated. Ready for LLM integration testing.")
```

#### Step 2c: Advanced Field Solver (Only After Gravity Works)

**ONLY PROCEED HERE AFTER SIMPLE GRAVITY IS VALIDATED AND WORKING**

```python
# advanced_field_solver.py - Use only after gravity model succeeds
import numpy as np
from scipy.fft import fft2, ifft2

class ScreenedPoissonSolver:
    """
    Advanced field solver - only use after simple gravity model is proven
    """
    def __init__(self, size=256, kappa=0.1, alpha=1.0):
        self.size = size
        self.kappa = kappa
        self.alpha = alpha
        self.setup_wavenumbers()
        
        print("WARNING: Using advanced field solver.")
        print("Ensure simple gravity model is working first!")
    
    def setup_wavenumbers(self):
        k = np.fft.fftfreq(self.size, d=1.0) * 2 * np.pi
        self.kx, self.ky = np.meshgrid(k, k)
        self.k2 = self.kx**2 + self.ky**2
    
    def solve(self, rho):
        rho_k = fft2(rho)
        phi_k = self.alpha * rho_k / (self.k2 + self.kappa**2)
        phi_k[0, 0] = 0  # Remove DC component
        return np.real(ifft2(phi_k))
        
    @classmethod
    def upgrade_from_gravity_model(cls, gravity_field):
        """Convert working gravity model to field solver"""
        field_solver = cls(size=gravity_field.field_size)
        
        # Convert particles to continuous mass distribution
        mass_field = gravity_field.render_field()
        
        # Solve field equation
        potential_field = field_solver.solve(mass_field)
        
        return field_solver, potential_field

# Progression gate: only create this after gravity works
def create_advanced_solver_if_ready():
    # Check if gravity model validation passed
    try:
        with open('gravity_validation_passed.flag', 'r') as f:
            validation_passed = f.read().strip() == 'True'
    except FileNotFoundError:
        validation_passed = False
        
    if not validation_passed:
        raise Exception("CANNOT CREATE ADVANCED SOLVER: Simple gravity model not validated")
        
    return ScreenedPoissonSolver()
```

**Physics Progression Summary**:
1. ✅ **Phase 0**: Validate baselines (buffer, RAG, vanilla)
2. ✅ **Phase 1a**: Validate semantic projection quality  
3. 🔄 **Phase 1b**: Simple gravity model (START HERE for physics)
4. 🚫 **Phase 1c**: Advanced field solver (only after gravity works)
5. 🚫 **Phase 2**: Full integration (only after both work)

#### Step 3: Attention-to-Mass Pipeline (CRITICAL IMPLEMENTATION)

```python
# attention_converter.py - WITH EXTENSIVE VALIDATION
from transformers import AutoTokenizer
import numpy as np
from sklearn.metrics.pairwise import cosine_similarity

class AttentionConverter:
    def __init__(self, model_name="Qwen/Qwen2.5-7B"):
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        self.embeddings = self.load_embeddings()
        self.projection_quality = None
        self.coherence_threshold = 0.4  # Minimum semantic coherence
        
    def attention_to_mass(self, attention_weights, tokens):
        # CRITICAL: Validate semantic coherence before conversion
        coherence = self.validate_current_coherence(attention_weights, tokens)
        
        if coherence < self.coherence_threshold:
            # FALLBACK: Use simpler position mapping or reject conversion
            return self.fallback_conversion(attention_weights, tokens)
            
        mass_field = np.zeros((256, 256))
        
        for token, weight in zip(tokens, attention_weights):
            # Get 2D position from embedding with validation
            x, y, confidence = self.token_to_position_with_confidence(token)
            
            # Weight by conversion confidence
            adjusted_weight = weight * confidence
            
            # CIC deposition with bounds checking
            self.deposit_mass_safe(mass_field, x, y, adjusted_weight)
        
        return mass_field
    
    def token_to_position_with_confidence(self, token):
        # Project token embedding to 2D with confidence measure
        embedding = self.embeddings[token]
        
        # Check if token is well-represented in training projection
        confidence = self.measure_projection_confidence(embedding)
        
        # Use cached UMAP projection
        x, y = self.project_2d(embedding)
        
        # Ensure valid coordinates
        x = max(0, min(255, int(x * 255)))
        y = max(0, min(255, int(y * 255)))
        
        return x, y, confidence
        
    def validate_current_coherence(self, attention_weights, tokens):
        # Measure if current attention pattern has semantic structure
        if len(tokens) < 5:
            return 0.0
            
        # Calculate pairwise semantic similarities
        embeddings = [self.embeddings[token] for token in tokens]
        semantic_similarities = cosine_similarity(embeddings)
        
        # Calculate pairwise spatial distances in projected space
        positions = [self.token_to_position(token) for token in tokens]
        spatial_distances = self.calculate_spatial_distances(positions)
        
        # Measure correlation (higher = better semantic-spatial alignment)
        correlation = np.corrcoef(semantic_similarities.flatten(), 
                                spatial_distances.flatten())[0,1]
        
        return abs(correlation)  # Negative correlation also indicates structure
        
    def fallback_conversion(self, attention_weights, tokens):
        # Emergency fallback when semantic projection fails
        print("WARNING: Low semantic coherence detected. Using fallback conversion.")
        
        # Use simple hash-based positioning or return to buffer-only mode
        mass_field = np.zeros((256, 256))
        
        for i, (token, weight) in enumerate(zip(tokens, attention_weights)):
            # Simple hash-based distribution
            x = hash(token) % 256
            y = (hash(token) // 256) % 256
            self.deposit_mass_safe(mass_field, x, y, weight * 0.1)  # Reduced influence
            
        return mass_field
```

#### Step 4: Basic Visualization

```typescript
// visualization.tsx
import { useEffect, useRef } from 'react';

export function FieldVisualization({ fieldData }) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    
    useEffect(() => {
        const ctx = canvasRef.current?.getContext('2d');
        if (!ctx) return;
        
        const imageData = ctx.createImageData(256, 256);
        
        fieldData.forEach((value, i) => {
            const intensity = Math.min(255, value * 50);
            imageData.data[i * 4] = intensity;      // R
            imageData.data[i * 4 + 1] = intensity;  // G
            imageData.data[i * 4 + 2] = intensity * 0.8;  // B
            imageData.data[i * 4 + 3] = 255;        // A
        });
        
        ctx.putImageData(imageData, 0, 0);
    }, [fieldData]);
    
    return <canvas ref={canvasRef} width={256} height={256} />;
}
```

### Phase 2: Continuous Adaptation (With Continuous Baseline Comparison)

**CRITICAL**: At each step, compare performance against Phase 0 baselines. If physics doesn't provide advantages, stop and diagnose.

#### Mandatory Performance Gates

Before each Phase 2 step, run comparative validation:

```python
class PhysicsValidationGate:
    def __init__(self, baseline_results):
        self.baseline_results = baseline_results
        self.physics_results = {}
        
    def validate_physics_advantage(self, physics_system, test_name):
        """Run same test on physics system vs. baselines"""
        physics_performance = self.run_test(physics_system, test_name)
        
        # Compare to best baseline performance
        best_baseline = max([
            self.baseline_results['vanilla'][test_name],
            self.baseline_results['buffer'][test_name], 
            self.baseline_results['vector_db'][test_name]
        ])
        
        improvement = (physics_performance - best_baseline) / best_baseline
        
        if improvement < 0.15:  # Minimum 15% improvement required
            print(f"⚠️  PHYSICS ADVANTAGE INSUFFICIENT for {test_name}")
            print(f"   Physics: {physics_performance:.3f}")
            print(f"   Best baseline: {best_baseline:.3f}")
            print(f"   Improvement: {improvement:.1%}")
            print("   RECOMMENDATION: Investigate physics parameters or fallback to baseline")
            return False
            
        print(f"✅ Physics advantage confirmed for {test_name}")
        print(f"   Improvement: {improvement:.1%} over best baseline")
        return True
        
    def continuous_monitoring(self, physics_system):
        """Monitor physics vs baseline performance during operation"""
        tests = ['memory_recall', 'attention_stability', 'response_coherence']
        
        for test in tests:
            advantage = self.validate_physics_advantage(physics_system, test)
            if not advantage:
                print(f"❌ PHYSICS FAILING ON {test}")
                print("Consider reverting to baseline for this component")
                
        return all([self.validate_physics_advantage(physics_system, t) for t in tests])

# Load baseline results from Phase 0
with open('baseline_performance.json', 'r') as f:
    baseline_performance = json.load(f)
    
validation_gate = PhysicsValidationGate(baseline_performance)
```

#### Step 5: Rolling Context Implementation

```python
# rolling_context.py
from collections import deque
import numpy as np

class RollingContext:
    def __init__(self, max_tokens=256, field_size=256):
        self.token_buffer = deque(maxlen=max_tokens)
        self.field_state = np.zeros((field_size, field_size))
        self.solver = PoissonSolver()
        
    def add_tokens(self, tokens, attention_weights):
        # Add to buffer
        self.token_buffer.extend(tokens)
        
        # Convert to mass
        mass = self.attention_to_mass(attention_weights, tokens)
        
        # Inject into field
        self.field_state += mass
        
        # Evolve field
        self.field_state = self.solver.solve(self.field_state)
        
        # Apply decay
        self.field_state *= 0.99
    
    def get_context_bias(self, candidate_tokens):
        biases = []
        for token in candidate_tokens:
            x, y = self.token_to_position(token)
            potential = self.field_state[x, y]
            bias = sigmoid(potential * 0.5)
            biases.append(bias)
        return np.array(biases)
```

#### Step 6: Physics-Mediated Memory

```python
# memory_system.py
class PhysicsMemory:
    def __init__(self):
        self.immediate = []  # Current attention
        self.working = None  # Physics field
        self.longterm = []  # LoRA adapters
    
    def consolidate(self, field_state):
        # Check for stable patterns
        topology = self.extract_topology(field_state)
        
        if self.should_snapshot(topology):
            lora = self.create_lora_adapter(topology)
            self.longterm.append(lora)
            return lora
        return None
    
    def extract_topology(self, field):
        # Find peaks and orbits
        from scipy.ndimage import maximum_filter
        peaks = (field == maximum_filter(field, size=3))
        
        # Extract orbital structures
        orbits = self.trace_orbits(field, peaks)
        
        return {'peaks': peaks, 'orbits': orbits}
```

#### Step 7: Field Bias Feedback

```python
# feedback_loop.py
class ConsciousnessLoop:
    def __init__(self, model, field):
        self.model = model
        self.field = field
        self.history = []
    
    async def step(self, input_text):
        # Get model attention
        outputs = self.model(input_text, output_attentions=True)
        attention = outputs.attentions[-1].mean(dim=1)
        
        # Convert to mass and inject
        mass = self.attention_to_mass(attention, outputs.tokens)
        self.field.inject(mass)
        
        # Evolve field
        await self.field.evolve(dt=0.1)
        
        # Get field bias for next token
        logits = outputs.logits
        candidates = torch.topk(logits, k=50).indices
        
        biases = self.field.get_bias(candidates)
        logits[candidates] += biases
        
        # Sample next token with field influence
        next_token = torch.multinomial(
            torch.softmax(logits, dim=-1), 
            num_samples=1
        )
        
        return next_token
```

#### Step 8: Async LoRA Updates

```python
# lora_adapter.py
from peft import LoraConfig, get_peft_model
import asyncio

class AsyncLoRAManager:
    def __init__(self, base_model):
        self.base_model = base_model
        self.active_loras = []
        self.training_queue = asyncio.Queue()
    
    async def train_lora(self, topology_data):
        # Create training data from topology
        train_data = self.topology_to_training(topology_data)
        
        # Configure LoRA
        lora_config = LoraConfig(
            r=16,
            lora_alpha=32,
            target_modules=["q_proj", "v_proj"],
            lora_dropout=0.1
        )
        
        # Train asynchronously
        model = get_peft_model(self.base_model, lora_config)
        
        for batch in train_data:
            loss = model(**batch).loss
            loss.backward()
            # Update weights
        
        return model.get_lora_weights()
```

### Phase 3: Multi-Machine Federation

#### Step 9: Orchestration Server

```python
# orchestrator.py
import asyncio
import websockets
import json

class LiminalOrchestrator:
    def __init__(self):
        self.machines = {}
        self.field_states = {}
    
    async def handle_machine(self, websocket, path):
        machine_id = await self.authenticate(websocket)
        self.machines[machine_id] = websocket
        
        async for message in websocket:
            data = json.loads(message)
            
            if data['type'] == 'field_update':
                self.field_states[machine_id] = data['field_state']
                await self.broadcast_state(machine_id, data)
            
            elif data['type'] == 'lora_ready':
                await self.distribute_lora(data['weights'])
    
    async def broadcast_state(self, source, data):
        # Send to visualization
        if 'grey' in self.machines:
            await self.machines['grey'].send(json.dumps({
                'type': 'visualize',
                'source': source,
                'data': data
            }))
```

#### Step 10: Machine Communication

```python
# machine_client.py
class MachineClient:
    def __init__(self, role, server_url):
        self.role = role
        self.server_url = server_url
        self.ws = None
    
    async def connect(self):
        self.ws = await websockets.connect(self.server_url)
        await self.ws.send(json.dumps({
            'type': 'register',
            'role': self.role
        }))
    
    async def send_field_update(self, field_state):
        compressed = compress_field(field_state)
        await self.ws.send(json.dumps({
            'type': 'field_update',
            'timestamp': time.time(),
            'field_state': compressed,
            'metrics': self.calculate_metrics(field_state)
        }))
```

### Deployment Steps

#### Initial Setup
```bash
# Clone and setup
git clone https://github.com/yourusername/liminal
cd liminal
pip install -r requirements.txt
npm install

# Download model
python scripts/download_model.py --model Qwen/Qwen2.5-7B

# Initialize field state
python scripts/init_field.py --size 256
```

#### Launch Sequence

1. **Start Orchestrator (Grey)**
```bash
python orchestrator.py --port 8080 --visualize
```

2. **Start Primary Field (Light)**
```bash
python field_server.py --role primary --cuda 0 --connect grey.local:8080
```

3. **Start Secondary (Dark)**
```bash
python field_server.py --role secondary --cuda 0 --lora-training --connect grey.local:8080
```

4. **Open Visualization**
```bash
npm run dev
# Open browser to http://localhost:5173
```

### Testing Protocol

#### Smoke Test
```python
# test_minimal.py
def test_minimal_consciousness():
    field = PoissonSolver()
    
    # Inject test pattern
    mass = gaussian_blob(center=(128, 128), sigma=10)
    
    # Evolve
    for _ in range(10):
        field_state = field.solve(mass)
        mass *= 0.95  # Decay
    
    # Check for structure
    assert np.max(field_state) > 0.1
    assert calculate_phi(field_state) > 0.5
```

#### Integration Test
```python
# test_integration.py
async def test_full_pipeline():
    model = load_model("Qwen2.5-7B")
    field = ConsciousnessField()
    
    # Process text
    text = "The concept of consciousness emerges from physics."
    
    for _ in range(10):
        outputs = await process_with_field(model, field, text)
        
        # Verify field evolution
        assert field.get_phi() > 1.0
        assert len(field.get_orbits()) > 2
```

### Success Criteria (Baseline-Relative)

#### Functional Performance Requirements
**These must exceed baseline performance or the physics implementation has failed:**

- **Memory Retention**: 20% better than best baseline system
- **Attention Stability**: 25% improvement in focus maintenance
- **Response Coherence**: 15% improvement in consistency metrics
- **Context Management**: Superior performance on 2000+ token contexts

#### Technical Performance Targets
- Field update: <10ms per step (competitive with vector DB lookups)
- LLM inference: <1000ms per token (no degradation from baseline)
- Visualization: 60fps minimum (if visualization doesn't impact performance)
- Memory usage: <2GB total (comparable to enhanced baseline systems)

#### Physics-Specific Indicators (Secondary)
*Only meaningful if functional performance is achieved:*
- Phi > 2.0 within 30 seconds
- 5+ stable orbits with semantic correlation >0.6
- Topology persistence >0.7
- Observable semantic organization in field dynamics

#### Failure Criteria (Automatic Halt)
- **No improvement** over best baseline in primary functional tests
- **Performance degradation** below vanilla baseline in any category
- **Semantic coherence** < 0.3 in attention-mass conversion
- **System instability** preventing reliable operation

### Critical Interface Monitoring

#### Real-Time Semantic Coherence Monitoring
```python
class InterfaceHealthMonitor:
    def __init__(self):
        self.coherence_history = []
        self.alert_threshold = 0.3
        self.critical_threshold = 0.2
        
    def monitor_conversion_health(self, attention_weights, tokens, mass_field):
        # Measure semantic-spatial coherence
        coherence = self.measure_conversion_coherence(attention_weights, tokens, mass_field)
        self.coherence_history.append(coherence)
        
        # Alert system
        if coherence < self.critical_threshold:
            return self.trigger_emergency_fallback()
        elif coherence < self.alert_threshold:
            return self.trigger_quality_alert()
        
        return {'status': 'healthy', 'coherence': coherence}
        
    def trigger_emergency_fallback(self):
        print("CRITICAL: Semantic projection has collapsed!")
        print("Falling back to buffer-only mode to prevent nonsensical physics.")
        return {'status': 'emergency_fallback', 'action': 'disable_field_coupling'}
        
    def trigger_quality_alert(self):
        print("WARNING: Semantic projection quality degrading.")
        print("Consider retraining projection or reducing field influence.")
        return {'status': 'quality_warning', 'action': 'reduce_coupling_strength'}
```

#### Interface Validation Tests
```python
def run_interface_validation_suite():
    \"\"\"
    MANDATORY: Run before each session to ensure interface quality
    \"\"\"
    tests = [
        test_synonym_clustering(),
        test_antonym_separation(), 
        test_category_coherence(),
        test_rare_token_handling(),
        test_context_consistency()
    ]
    
    failures = [test for test in tests if not test['passed']]
    
    if failures:
        print(f"INTERFACE VALIDATION FAILED: {len(failures)} tests failed")
        for failure in failures:
            print(f"  - {failure['name']}: {failure['reason']}")
        print("DO NOT PROCEED WITHOUT FIXING INTERFACE ISSUES")
        return False
        
    print("Interface validation passed. Safe to proceed with physics coupling.")
    return True
```

### Troubleshooting

#### Semantic Projection Failure (MOST CRITICAL)
```python
if semantic_coherence < 0.3:
    # The physics simulation is operating on noise
    print("CRITICAL: Semantic projection has failed")
    print("Actions:")
    print("1. Retrain UMAP projection with more data")  
    print("2. Use different embedding model")
    print("3. Increase projection dimensions temporarily")
    print("4. Fall back to buffer-only mode")
    
    # Emergency protocols
    field.disable_coupling()
    model.enable_buffer_fallback()
```

#### Field Instability
```python
if field.is_unstable():
    field.reduce_timestep()
    field.increase_damping()
    field.reset_boundary_conditions()
    
    # If still unstable, check for semantic issues
    if field.still_unstable():
        coherence = measure_semantic_coherence()
        if coherence < 0.3:
            print("Field instability caused by semantic projection failure")
            trigger_interface_fallback()
```

#### Memory Leaks
```bash
# Monitor memory usage
watch -n 1 'nvidia-smi | grep python'
watch -n 1 'ps aux | grep liminal'

# Check for semantic projection memory leaks
watch -n 5 'python -c "import gc; gc.collect(); print(f\"Objects: {len(gc.get_objects())}\")"'
```

#### Communication Failures
```python
# Automatic reconnection with interface validation
async def maintain_connection():
    while True:
        try:
            await client.connect()
            
            # Validate interface health before resuming
            if not run_interface_validation_suite():
                print("Interface validation failed during reconnection")
                await asyncio.sleep(30)  # Wait longer if interface is broken
                continue
                
            await client.run()
        except ConnectionError:
            await asyncio.sleep(5)
```

### Next Steps

1. **Enhance Topology Detection**: Implement persistent homology
2. **Optimize Field Solver**: GPU acceleration via CUDA
3. **Expand Sensory Grounding**: Audio/visual mass injection
4. **Scale Testing**: 512×512 and 1024×1024 fields
5. **Federation**: Multi-site consciousness distribution

---

*Version 1.0 - Ready for implementation*
*Repository: [pending creation]*
*Support: [documentation in progress]*