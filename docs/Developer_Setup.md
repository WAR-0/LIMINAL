# LIMINAL Developer Setup Guide
## Local Development Environment Configuration

**Document Version**: 1.0  
**Target OS**: Linux (Ubuntu 20.04+), macOS (10.15+), Windows 10/11 (WSL2)  
**Hardware Requirements**: RTX 4080/4070 Ti recommended, RTX 3080 minimum

---

## Prerequisites

### Hardware Requirements
- **GPU**: NVIDIA RTX 3080 or better (8GB+ VRAM recommended)
- **CPU**: 8+ cores, 3.0GHz+ (Intel i7/AMD Ryzen 7 or better)
- **RAM**: 16GB minimum, 32GB recommended
- **Storage**: 50GB free space (10GB for models, 20GB for development, 20GB for data)

### Software Dependencies
- **Python**: 3.10 or 3.11 (3.12 not yet fully supported by all dependencies)
- **CUDA**: 11.8 or 12.0 (must match PyTorch CUDA version)
- **Git**: 2.30+ with LFS support for model files
- **Docker**: Optional but recommended for containerized development

---

## Quick Start (5-minute setup)

### 1. Clone Repository
```bash
git clone https://github.com/your-org/LIMINAL.git
cd LIMINAL
```

### 2. Environment Setup
```bash
# Create Python virtual environment
python3.10 -m venv liminal_env
source liminal_env/bin/activate  # Linux/macOS
# liminal_env\Scripts\activate     # Windows

# Install dependencies
pip install -r requirements.txt
```

### 3. Hardware Verification
```bash
# Verify CUDA installation
python -c "import torch; print(f'CUDA available: {torch.cuda.is_available()}, Device: {torch.cuda.get_device_name()}')"

# Expected output: "CUDA available: True, Device: NVIDIA GeForce RTX 4080"
```

### 4. Basic Functionality Test
```bash
# Run basic system test
python tests/test_basic_functionality.py

# Expected: All tests pass, physics engine initializes, LLM loads
```

---

## Detailed Installation

### Python Environment Setup

**Option 1: Using Conda (Recommended)**
```bash
# Create conda environment with Python 3.10
conda create -n liminal python=3.10
conda activate liminal

# Install PyTorch with CUDA support first (critical order)
conda install pytorch pytorch-cuda=11.8 -c pytorch -c nvidia

# Install remaining dependencies
pip install -r requirements.txt
```

**Option 2: Using pip + venv**
```bash
# Create virtual environment
python3.10 -m venv ~/.liminal_env
source ~/.liminal_env/bin/activate

# Upgrade pip and install wheel
pip install --upgrade pip wheel

# Install PyTorch first
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu118

# Install other dependencies
pip install -r requirements.txt
```

### Dependencies Overview

**Core Dependencies** (requirements.txt):
```
# LLM and ML Framework
torch>=2.0.0
transformers>=4.30.0
accelerate>=0.20.0
datasets>=2.10.0

# Physics and Numerical Computing
numpy>=1.24.0
scipy>=1.10.0
cupy-cuda11x>=11.0.0  # GPU acceleration
umap-learn>=0.5.3     # Dimensionality reduction
scikit-learn>=1.3.0

# Visualization and Monitoring
matplotlib>=3.7.0
plotly>=5.14.0
dash>=2.10.0          # Web dashboard
flask>=2.3.0

# Development and Testing  
pytest>=7.4.0
black>=23.0.0
flake8>=6.0.0
mypy>=1.4.0

# Data and Storage
h5py>=3.8.0          # Field state storage
redis>=4.5.0         # Optional caching
sqlalchemy>=2.0.0    # Metadata storage
```

**Development Dependencies** (requirements-dev.txt):
```
# Testing and Quality
pytest-cov>=4.1.0
pytest-benchmark>=4.0.0
pytest-mock>=3.10.0

# Documentation
sphinx>=7.0.0
sphinx-rtd-theme>=1.2.0

# Profiling and Debugging
line-profiler>=4.0.0
memory-profiler>=0.60.0
py-spy>=0.3.14
```

### GPU Setup Verification

**CUDA Installation Check**:
```bash
# Check NVIDIA driver
nvidia-smi

# Check CUDA version
nvcc --version

# Python GPU verification
python -c "
import torch
import cupy
print(f'PyTorch CUDA: {torch.cuda.is_available()}')
print(f'CuPy CUDA: {cupy.cuda.is_available()}')
print(f'GPU Memory: {torch.cuda.get_device_properties(0).total_memory / 1e9:.1f}GB')
"
```

**Expected Output**:
```
PyTorch CUDA: True
CuPy CUDA: True  
GPU Memory: 16.0GB
```

### Model Downloads

**Automatic Model Setup**:
```bash
# Download required models (runs automatically on first use)
python scripts/setup_models.py

# Manual download if needed
python -c "
from transformers import AutoModelForCausalLM, AutoTokenizer
model = AutoModelForCausalLM.from_pretrained('Qwen/Qwen2.5-7B')
tokenizer = AutoTokenizer.from_pretrained('Qwen/Qwen2.5-7B')
"
```

---

## Development Workflow

### Project Structure
```
LIMINAL/
├── liminal/                 # Core source code
│   ├── physics/            # Physics engine components
│   ├── interface/          # Attention-mass interface
│   ├── memory/             # Memory management system
│   ├── llm/               # LLM integration layer
│   ├── monitoring/         # Visualization and monitoring
│   └── utils/             # Shared utilities
├── tests/                  # Test suite
│   ├── unit/              # Unit tests
│   ├── integration/       # Integration tests
│   └── benchmarks/        # Performance benchmarks
├── scripts/               # Utility scripts
├── docs/                  # Documentation
├── data/                  # Test data and examples
├── models/                # Downloaded model cache
└── config/               # Configuration files
```

### Running Tests

**Unit Tests**:
```bash
# Run all tests
pytest tests/

# Run specific component tests
pytest tests/unit/test_physics.py
pytest tests/unit/test_interface.py
pytest tests/unit/test_memory.py

# Run with coverage
pytest --cov=liminal tests/
```

**Integration Tests**:
```bash
# End-to-end system tests
pytest tests/integration/

# Performance benchmarks
pytest tests/benchmarks/ --benchmark-only
```

**Hardware-Specific Tests**:
```bash
# Test GPU functionality
pytest tests/test_gpu.py -v

# Test different hardware profiles
python tests/test_hardware_profiles.py
```

### Code Quality

**Formatting and Linting**:
```bash
# Format code
black liminal/ tests/

# Lint code
flake8 liminal/ tests/

# Type checking
mypy liminal/
```

**Pre-commit Hooks** (optional):
```bash
# Install pre-commit
pip install pre-commit

# Setup hooks
pre-commit install

# Run hooks manually
pre-commit run --all-files
```

### Development Server

**Start Development System**:
```bash
# Start LIMINAL system with monitoring
python -m liminal.main --dev --monitoring

# Access monitoring dashboard
# http://localhost:5000/dashboard
```

**Development Configuration** (config/dev.yaml):
```yaml
system:
  debug: true
  log_level: DEBUG
  
physics:
  field_resolution: [256, 256]
  update_frequency: 10
  gpu_acceleration: true
  
interface:
  projection_method: umap
  correlation_threshold: 0.4
  monitoring_enabled: true
  
memory:
  consolidation_threshold: 0.8
  max_snapshots: 100
  compression_target: 15
  
monitoring:
  dashboard_port: 5000
  field_visualization: true
  performance_tracking: true
```

---

## Troubleshooting

### Common Issues

**CUDA/GPU Issues**:
```bash
# Issue: CUDA out of memory
# Solution: Reduce field resolution or batch size
export LIMINAL_FIELD_RESOLUTION=128
export LIMINAL_BATCH_SIZE=8

# Issue: CuPy not found
# Solution: Install correct CuPy version
pip install cupy-cuda11x  # For CUDA 11.x
pip install cupy-cuda12x  # For CUDA 12.x
```

**Model Loading Issues**:
```bash
# Issue: Transformers model not found
# Solution: Clear cache and re-download
rm -rf ~/.cache/huggingface/
python scripts/setup_models.py --force-download

# Issue: Out of memory loading model
# Solution: Use model sharding
export LIMINAL_MODEL_SHARDING=true
```

**Physics Engine Issues**:
```bash
# Issue: Field solver unstable
# Solution: Reduce timestep or increase damping
export LIMINAL_PHYSICS_TIMESTEP=0.01
export LIMINAL_PHYSICS_DAMPING=0.1

# Issue: Poor semantic correlation
# Solution: Retrain UMAP projection
python scripts/retrain_projection.py --method=umap --samples=10000
```

### Performance Optimization

**GPU Memory Optimization**:
```bash
# Monitor GPU memory usage
watch -n 1 nvidia-smi

# Optimize for your hardware
python scripts/optimize_config.py --gpu=rtx4080
```

**CPU Performance**:
```bash
# Use multiple CPU cores
export LIMINAL_NUM_THREADS=8
export OMP_NUM_THREADS=8
```

### Debug Mode

**Enable Detailed Logging**:
```bash
export LIMINAL_LOG_LEVEL=DEBUG
export LIMINAL_LOG_FILE=debug.log
python -m liminal.main --debug
```

**Performance Profiling**:
```bash
# Profile physics engine
python -m cProfile -o physics_profile.prof -m liminal.physics.test_performance

# Profile memory usage
python -m memory_profiler -m liminal.main --profile-memory
```

---

## IDE Configuration

### VS Code Setup

**Recommended Extensions**:
- Python (Microsoft)
- Pylance (Microsoft) 
- Python Docstring Generator
- GitLens
- Thunder Client (for API testing)

**VS Code Settings** (.vscode/settings.json):
```json
{
    "python.defaultInterpreterPath": "./liminal_env/bin/python",
    "python.linting.enabled": true,
    "python.linting.flake8Enabled": true,
    "python.formatting.provider": "black",
    "python.testing.pytestEnabled": true,
    "python.testing.pytestArgs": ["tests/"],
    "files.associations": {
        "*.yaml": "yaml",
        "*.yml": "yaml"
    }
}
```

### PyCharm Setup

**Project Configuration**:
1. Open LIMINAL directory as project
2. Configure Python interpreter: Settings > Project > Python Interpreter
3. Set virtual environment path: `./liminal_env/bin/python`
4. Enable pytest: Settings > Tools > Python Integrated Tools > Testing > pytest
5. Configure code style: Settings > Editor > Code Style > Python > Black

---

## Environment Variables

**Core Configuration**:
```bash
# System configuration
export LIMINAL_CONFIG_FILE=config/dev.yaml
export LIMINAL_DATA_DIR=./data
export LIMINAL_MODEL_CACHE=./models
export LIMINAL_LOG_LEVEL=INFO

# Hardware configuration
export LIMINAL_GPU_DEVICE=0
export LIMINAL_NUM_THREADS=8
export CUDA_VISIBLE_DEVICES=0

# Development flags
export LIMINAL_DEBUG=false
export LIMINAL_PROFILING=false
export LIMINAL_MONITORING=true
```

**Performance Tuning**:
```bash
# Physics engine tuning
export LIMINAL_FIELD_RESOLUTION=256
export LIMINAL_UPDATE_FREQUENCY=10
export LIMINAL_PHYSICS_TIMESTEP=0.1

# Memory management
export LIMINAL_MAX_FIELD_HISTORY=1000
export LIMINAL_MEMORY_LIMIT=8GB
export LIMINAL_CACHE_SIZE=1000
```

---

This developer setup guide provides everything needed to get LIMINAL running locally and begin development. The configuration is optimized for the target hardware while providing fallback options for different development environments.