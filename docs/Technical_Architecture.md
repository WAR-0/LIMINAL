# LIMINAL Technical Architecture
## Physics-Based Memory Architecture Implementation Specification

**Document Version**: 1.0  
**Target Implementation**: Single Developer with AI Assistance  
**Hardware Target**: RTX 4080/4070 Ti (minimum RTX 3080)  
**Development Timeline**: 18-28 weeks phased implementation

---

## System Overview

LIMINAL implements a physics-based memory architecture for LLMs using continuous field dynamics to maintain persistent cognitive states. The system provides enhanced memory consolidation, attention coherence, and identity persistence through real-time physics simulation coupled with attention mechanisms.

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    LIMINAL System                           │
├─────────────────────────────────────────────────────────────┤
│  User Interface & API Layer                                 │
├─────────────────────────────────────────────────────────────┤
│  LLM Integration Layer (Qwen2.5-7B)                        │
├─────────────────────────────────────────────────────────────┤
│  Attention-Physics Interface (CRITICAL COMPONENT)           │
├─────────────────────────────────────────────────────────────┤
│  Physics Engine (256×256 Field @ 10Hz)                     │
├─────────────────────────────────────────────────────────────┤
│  Memory Management (LoRA + Topology)                       │
├─────────────────────────────────────────────────────────────┤
│  Monitoring & Visualization                                │
├─────────────────────────────────────────────────────────────┤
│  Storage Layer (Field State + Memory)                      │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Physics Engine Component

**Primary Function**: Real-time 2D field dynamics simulation
**Technology**: FFT-based screened Poisson solver
**Update Frequency**: 10Hz for stable real-time operation
**Field Resolution**: 256×256 (scalable to 512×512)

**Subcomponents**:
- **Field Solver**: Implements screened Poisson equation ∇²φ - κ²φ = αρ
- **Mass Distribution**: CIC (Cloud-in-Cell) deposition from attention weights
- **Wave Propagation**: Field dynamics with configurable wave speed
- **Boundary Conditions**: Periodic boundaries with damping zones
- **Conservation Monitoring**: Real-time mass/energy conservation tracking

**Implementation Stack**:
- **Core Solver**: NumPy + SciPy (FFT operations)
- **GPU Acceleration**: CuPy for CUDA-enabled operations
- **Performance**: Vectorized operations, minimal Python loops
- **Memory Management**: Efficient field state caching and history

### 2. Attention-Physics Interface (CRITICAL)

**Primary Function**: Bidirectional conversion between LLM attention and physics field
**Risk Level**: Highest (60% probability of project failure if this fails)
**Success Criteria**: Semantic-spatial correlation >0.4 across domains

**Interface Architecture**:
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   LLM Attention │───▶│  Projection     │───▶│  2D Mass Field  │
│   [N×D tensor]  │    │  (UMAP/t-SNE)   │    │  [256×256]      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
        ▲                        │                        │
        │                        ▼                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Attention Bias │◀───│  Field Query    │◀───│  Field Gradients│
│  [N×D tensor]   │    │  System         │    │  & Topology     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

**Subcomponents**:
- **Semantic Projection**: UMAP-based dimensionality reduction (3072D → 2D)
- **Mass Deposition**: Cloud-in-Cell mapping from 2D coordinates to grid
- **Field Sampling**: Gradient-based sampling from field to attention space
- **Coherence Monitoring**: Real-time semantic correlation tracking
- **Fallback System**: Automatic conventional mode if coherence fails

**Critical Implementation Details**:
- **UMAP Parameters**: n_neighbors=15, min_dist=0.1, metric='cosine'
- **Alternative Projections**: t-SNE backup, custom learned projection option
- **Real-time Updates**: Incremental UMAP updates for new tokens
- **Quality Monitoring**: Semantic clustering metrics, correlation tracking

### 3. LLM Integration Layer

**Primary Function**: Integration with Qwen2.5-7B language model
**Architecture**: Attention mechanism modification with field feedback

**Integration Points**:
- **Attention Hook**: Intercept attention weights during forward pass
- **Context Modification**: Inject field-derived context bias
- **Generation Loop**: Integration with streaming text generation
- **Memory Context**: Persistent context across conversation sessions

**Technical Implementation**:
```python
# Simplified integration architecture
class LiminalLLM(Qwen2Model):
    def __init__(self, physics_engine, interface):
        super().__init__()
        self.physics_engine = physics_engine
        self.interface = interface
        self.attention_hooks = []
    
    def forward(self, input_ids, attention_mask=None):
        # Standard forward pass with attention interception
        outputs = super().forward(input_ids, attention_mask)
        
        # Extract and process attention weights
        attention_weights = self.extract_attention_weights(outputs)
        
        # Update physics field
        self.interface.update_field(attention_weights)
        
        # Apply field feedback (if enabled)
        if self.field_feedback_enabled:
            field_bias = self.interface.get_attention_bias()
            outputs = self.apply_field_bias(outputs, field_bias)
        
        return outputs
```

### 4. Memory Management System

**Primary Function**: Persistent memory through LoRA snapshots triggered by topology analysis
**Architecture**: Event-driven consolidation with topology-based triggers

**Subcomponents**:
- **Topology Analyzer**: Real-time analysis of field topology changes
- **Consolidation Triggers**: Stability-based triggers for memory formation
- **LoRA Manager**: Low-Rank Adaptation snapshot creation and storage
- **Memory Retrieval**: Topology-based memory recall and integration
- **Compression System**: 20x compression target through selective consolidation

**Memory Architecture**:
```
Field Topology → Stability Analysis → Consolidation Decision
     ↓               ↓                       ↓
Peak Detection → Threshold Check → LoRA Snapshot Creation
     ↓               ↓                       ↓
Semantic Tagging → Storage → Retrieval Index Update
```

### 5. Monitoring & Visualization System

**Primary Function**: Real-time system monitoring and field visualization
**Target Users**: Developer debugging, research analysis, user interface

**Monitoring Components**:
- **Performance Metrics**: Processing speed, memory usage, field update rates
- **Quality Metrics**: Semantic coherence, attention stability, generation quality
- **Field Visualization**: Real-time 2D field rendering with topology overlay
- **System Health**: Component status, error rates, recovery system status
- **Research Analytics**: Consciousness-like pattern detection and analysis

**Visualization Architecture**:
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Real-time      │───▶│  Field          │───▶│  Web Interface  │
│  Field Data     │    │  Renderer       │    │  (Flask/FastAPI)│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Topology       │    │  Metrics        │    │  Interactive    │
│  Analysis       │    │  Dashboard      │    │  Controls       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Technology Stack

### Core Implementation
- **Programming Language**: Python 3.10+
- **LLM Framework**: Transformers (HuggingFace) with custom attention hooks
- **Physics Computation**: NumPy, SciPy, CuPy (GPU acceleration)
- **Dimensionality Reduction**: UMAP-learn, scikit-learn (t-SNE backup)
- **Neural Network Integration**: PyTorch with custom model extensions

### Performance & Optimization
- **GPU Computing**: CUDA 11.8+ (CuPy), PyTorch CUDA
- **Memory Management**: Memory mapping for large field states
- **Parallel Processing**: ThreadPoolExecutor for I/O operations
- **Caching**: Redis for field state caching (optional)

### Monitoring & Interface
- **Visualization**: Matplotlib (development), Plotly (web interface)
- **Web Framework**: FastAPI for RESTful API and real-time updates
- **Real-time Communication**: WebSockets for live field updates
- **Data Storage**: HDF5 for field history, SQLite for metadata

### Development & Testing
- **Testing Framework**: pytest with fixtures for component testing
- **Code Quality**: black, flake8, mypy for code formatting and type checking
- **Documentation**: Sphinx for API documentation
- **Version Control**: Git with semantic versioning

## Data Models

### Core Data Structures

**Field State**:
```python
@dataclass
class FieldState:
    field: np.ndarray  # [256, 256] float32 - main field values
    mass: np.ndarray   # [256, 256] float32 - mass distribution
    velocity: np.ndarray  # [256, 256, 2] float32 - field velocity
    timestamp: float   # Unix timestamp
    sequence_id: int   # Sequential state identifier
    metadata: Dict[str, Any]  # Additional state information
```

**Attention Mapping**:
```python
@dataclass
class AttentionMapping:
    token_ids: List[int]           # Token identifiers
    attention_weights: np.ndarray  # [N, N] attention matrix
    projected_coords: np.ndarray   # [N, 2] 2D coordinates
    semantic_correlation: float    # Quality metric
    projection_method: str         # 'umap', 't-sne', 'custom'
    timestamp: float
```

**Memory Snapshot**:
```python
@dataclass
class MemorySnapshot:
    snapshot_id: str              # Unique identifier
    field_topology: np.ndarray    # Compressed topology representation
    lora_weights: Dict[str, torch.Tensor]  # LoRA adaptation weights
    trigger_event: str            # What caused consolidation
    semantic_summary: str         # AI-generated summary
    creation_time: float
    access_count: int
    relevance_score: float
```

### API Specifications

**Core System API**:
```python
class LiminalSystem:
    def __init__(self, config: LiminalConfig):
        """Initialize LIMINAL system with configuration"""
    
    def process_text(self, text: str) -> LiminalResponse:
        """Process text through physics-enhanced LLM"""
    
    def get_field_state(self) -> FieldState:
        """Get current physics field state"""
    
    def get_system_metrics(self) -> SystemMetrics:
        """Get real-time system performance metrics"""
    
    def enable_field_feedback(self, enabled: bool):
        """Enable/disable physics-to-attention feedback"""
    
    def consolidate_memory(self, force: bool = False) -> MemorySnapshot:
        """Trigger memory consolidation"""
```

**Monitoring API**:
```python
class LiminalMonitor:
    def get_semantic_correlation(self) -> float:
        """Get current semantic-spatial correlation"""
    
    def get_attention_stability(self) -> AttentionMetrics:
        """Get attention coherence metrics"""
    
    def get_field_visualization(self) -> FieldVisualization:
        """Get field state for visualization"""
    
    def get_topology_analysis(self) -> TopologyMetrics:
        """Get field topology analysis"""
```

## Performance Requirements

### Real-Time Constraints
- **Field Updates**: <100ms per update cycle (10Hz target)
- **LLM Integration**: <50ms additional latency per token
- **Semantic Projection**: <200ms for incremental updates
- **Memory Access**: <10ms for consolidated memory retrieval

### Resource Requirements
- **GPU Memory**: 8GB minimum (RTX 4080), 6GB viable (RTX 3080)
- **System RAM**: 16GB minimum, 32GB recommended
- **Storage**: 10GB for field history, 5GB for memory snapshots
- **Network**: Not applicable (local execution)

### Scalability Targets
- **Field Resolution**: 256×256 baseline, scalable to 512×512
- **Context Length**: 8K tokens baseline, extensible to 32K
- **Memory Snapshots**: 1000+ snapshots with efficient retrieval
- **Session Duration**: 24+ hours continuous operation

## Security & Safety Considerations

### Data Security
- **No Personal Data Storage**: System operates on conversation content only
- **Local Processing**: All computation performed locally, no cloud dependencies
- **Memory Encryption**: Optional encryption for memory snapshots
- **Access Controls**: API authentication for monitoring interfaces

### System Safety
- **Graceful Degradation**: Automatic fallback to conventional mode
- **Error Recovery**: Comprehensive error handling and system recovery
- **Resource Monitoring**: Automatic resource usage monitoring and limits
- **Emergency Shutoff**: Manual override for all physics coupling

### Research Ethics
- **Consciousness Claims**: Clear labeling of speculative consciousness research
- **Performance Claims**: Rigorous validation of all functional claims
- **Data Handling**: Ethical handling of conversation data and system logs
- **Community Standards**: Adherence to AI research community standards

## Deployment Architecture

### Development Environment
```
Developer Workstation
├── Python 3.10+ Environment
├── CUDA 11.8+ Toolkit
├── Git Repository (local)
├── Testing Framework
└── Monitoring Interface (localhost)
```

### Production Deployment (Future)
```
Production Server
├── Containerized Application (Docker)
├── GPU Resource Management
├── API Gateway (FastAPI)
├── Monitoring Dashboard
├── Backup & Recovery System
└── Log Aggregation
```

### Hardware Configuration Profiles
```python
HARDWARE_PROFILES = {
    'rtx_4080': {
        'field_resolution': (256, 256),
        'update_frequency': 10,  # Hz
        'batch_size': 32,
        'memory_limit': '7GB'
    },
    'rtx_3080': {
        'field_resolution': (256, 256),
        'update_frequency': 8,   # Hz
        'batch_size': 16,
        'memory_limit': '5GB'
    },
    'cpu_fallback': {
        'field_resolution': (128, 128),
        'update_frequency': 2,   # Hz
        'batch_size': 4,
        'memory_limit': '2GB'
    }
}
```

---

This technical architecture provides the engineering foundation necessary to begin LIMINAL development. It translates the strategic validation into actionable technical specifications while maintaining focus on the critical success factors identified in the Multi-AI Assessment Synthesis.