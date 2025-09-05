"""
Basic functionality tests for LIMINAL system.
These tests verify that the basic structure is in place and working.
"""

import pytest
import sys
import os
import numpy as np
import torch

# Add parent directory to path for imports
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from liminal import LiminalSystem, __version__


class TestBasicFunctionality:
    """Basic system functionality tests."""
    
    def test_version_exists(self):
        """Test that version is defined."""
        assert __version__ is not None
        assert isinstance(__version__, str)
        assert "0.1.0" in __version__
    
    def test_liminal_system_creation(self):
        """Test that LiminalSystem can be instantiated."""
        system = LiminalSystem()
        assert system is not None
        assert hasattr(system, 'config_path')
        assert not system._initialized
    
    def test_liminal_system_initialization(self):
        """Test that LiminalSystem can be initialized."""
        system = LiminalSystem()
        system.initialize()
        assert system._initialized
        
        # Should not re-initialize
        system.initialize()
        assert system._initialized
    
    def test_process_text_basic(self):
        """Test basic text processing (placeholder functionality)."""
        system = LiminalSystem()
        result = system.process_text("Test input")
        assert result is not None
        assert isinstance(result, str)
        assert "LIMINAL system processing" in result
    
    @pytest.mark.critical
    def test_numpy_availability(self):
        """Test that NumPy is available and working."""
        arr = np.array([1, 2, 3])
        assert arr.shape == (3,)
        assert np.sum(arr) == 6
    
    @pytest.mark.critical
    def test_torch_availability(self):
        """Test that PyTorch is available and working."""
        tensor = torch.tensor([1, 2, 3])
        assert tensor.shape == torch.Size([3])
        assert torch.sum(tensor).item() == 6
    
    @pytest.mark.gpu
    def test_cuda_availability(self):
        """Test CUDA availability (GPU test)."""
        if torch.cuda.is_available():
            device = torch.device("cuda")
            tensor = torch.tensor([1, 2, 3]).to(device)
            assert tensor.device.type == "cuda"
            assert torch.sum(tensor).item() == 6
        else:
            pytest.skip("CUDA not available")
    
    def test_project_structure(self):
        """Test that project structure is correct."""
        import liminal
        
        # Check that package can be imported
        assert liminal.__name__ == 'liminal'
        
        # Check for expected attributes
        assert hasattr(liminal, '__version__')
        assert hasattr(liminal, 'LiminalSystem')