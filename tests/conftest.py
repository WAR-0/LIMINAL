"""
Pytest fixtures and configuration for LIMINAL testing.
"""

import pytest
import numpy as np
import torch
from typing import Tuple
import tempfile
import os

# Set random seeds for reproducibility
np.random.seed(42)
torch.manual_seed(42)
if torch.cuda.is_available():
    torch.cuda.manual_seed(42)


@pytest.fixture
def field_resolution() -> Tuple[int, int]:
    """Standard field resolution for testing."""
    return (256, 256)


@pytest.fixture
def small_field_resolution() -> Tuple[int, int]:
    """Small field resolution for quick tests."""
    return (64, 64)


@pytest.fixture
def sample_attention_weights() -> np.ndarray:
    """Generate sample attention weights for testing."""
    size = 100  # 100 tokens
    weights = np.random.rand(size, size)
    # Make it somewhat realistic - stronger diagonal
    np.fill_diagonal(weights, weights.diagonal() * 2)
    # Normalize
    weights = weights / weights.sum(axis=1, keepdims=True)
    return weights.astype(np.float32)


@pytest.fixture
def sample_embeddings() -> np.ndarray:
    """Generate sample token embeddings for testing."""
    num_tokens = 100
    embedding_dim = 3072  # Qwen2.5-7B embedding dimension
    embeddings = np.random.randn(num_tokens, embedding_dim)
    # Normalize
    embeddings = embeddings / np.linalg.norm(embeddings, axis=1, keepdims=True)
    return embeddings.astype(np.float32)


@pytest.fixture
def sample_field_state(field_resolution) -> np.ndarray:
    """Generate sample field state for testing."""
    field = np.random.randn(*field_resolution) * 0.1
    # Add some structure - gaussian peaks
    x, y = np.meshgrid(
        np.linspace(-1, 1, field_resolution[0]),
        np.linspace(-1, 1, field_resolution[1])
    )
    for _ in range(3):  # Add 3 random peaks
        cx, cy = np.random.uniform(-0.5, 0.5, 2)
        sigma = 0.2
        field += np.exp(-((x - cx)**2 + (y - cy)**2) / (2 * sigma**2))
    return field.astype(np.float32)


@pytest.fixture
def temp_config_file():
    """Create a temporary configuration file for testing."""
    config_content = """
system:
  debug: true
  log_level: DEBUG
  
physics:
  field_resolution: [256, 256]
  update_frequency: 10
  timestep: 0.1
  damping: 0.05
  
interface:
  projection_method: umap
  n_neighbors: 15
  min_dist: 0.1
  semantic_threshold: 0.4
  
memory:
  consolidation_threshold: 0.8
  max_snapshots: 10
  compression_target: 15
"""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.yaml', delete=False) as f:
        f.write(config_content)
        temp_path = f.name
    
    yield temp_path
    
    # Cleanup
    if os.path.exists(temp_path):
        os.unlink(temp_path)


@pytest.fixture
def gpu_available():
    """Check if GPU is available for testing."""
    return torch.cuda.is_available()


@pytest.fixture
def mock_llm_response():
    """Mock LLM response for testing."""
    def _mock_response(prompt: str) -> str:
        return f"Mock response to: {prompt[:50]}..."
    return _mock_response


# Markers for conditional test execution
def pytest_configure(config):
    """Configure custom pytest markers."""
    config.addinivalue_line(
        "markers", "gpu: Tests requiring GPU hardware"
    )
    config.addinivalue_line(
        "markers", "slow: Tests that take >10 seconds"
    )
    config.addinivalue_line(
        "markers", "critical: Critical path tests that must pass"
    )


# Skip GPU tests if no GPU available
def pytest_collection_modifyitems(config, items):
    """Automatically skip GPU tests if no GPU is available."""
    if not torch.cuda.is_available():
        skip_gpu = pytest.mark.skip(reason="GPU not available")
        for item in items:
            if "gpu" in item.keywords:
                item.add_marker(skip_gpu)