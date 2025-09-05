#!/usr/bin/env python3
"""
Semantic Projection Validation Tool for LIMINAL.
Critical validation for attention-to-mass interface - the make-or-break component.

This tool validates that semantic relationships are preserved when projecting
from high-dimensional attention space to 2D physics field coordinates.

Success Criteria:
- Global semantic correlation >0.4
- Local semantic clustering >0.6
- Cross-domain validation across 3+ domains
"""

import numpy as np
import matplotlib.pyplot as plt
from sklearn.metrics.pairwise import cosine_similarity
from sklearn.manifold import TSNE
from umap import UMAP
import argparse
import json
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
import warnings
warnings.filterwarnings('ignore')


@dataclass
class ValidationResult:
    """Results from semantic projection validation."""
    method: str
    domain: str
    global_correlation: float
    local_clustering: float
    cross_domain_score: float
    passed: bool
    details: Dict


class SemanticProjectionValidator:
    """Validates semantic preservation in dimensionality reduction."""
    
    def __init__(self, 
                 global_threshold: float = 0.4,
                 local_threshold: float = 0.6,
                 n_samples: int = 1000):
        """Initialize validator with thresholds.
        
        Args:
            global_threshold: Minimum global semantic correlation required
            local_threshold: Minimum local clustering correlation required
            n_samples: Number of samples to use for validation
        """
        self.global_threshold = global_threshold
        self.local_threshold = local_threshold
        self.n_samples = n_samples
        
    def generate_test_embeddings(self, domain: str = "mixed") -> Tuple[np.ndarray, List[str]]:
        """Generate test embeddings for a specific domain.
        
        Args:
            domain: One of 'science', 'literature', 'history', 'mixed'
            
        Returns:
            Embeddings array and labels
        """
        np.random.seed(42)  # Reproducibility
        
        if domain == "science":
            categories = ['physics', 'chemistry', 'biology', 'mathematics', 'computer_science']
        elif domain == "literature":
            categories = ['poetry', 'fiction', 'drama', 'essay', 'biography']
        elif domain == "history":
            categories = ['ancient', 'medieval', 'renaissance', 'modern', 'contemporary']
        else:  # mixed
            categories = ['science', 'art', 'history', 'technology', 'philosophy']
        
        embeddings = []
        labels = []
        
        samples_per_category = self.n_samples // len(categories)
        embedding_dim = 3072  # Qwen2.5-7B dimension
        
        for i, category in enumerate(categories):
            # Create clustered embeddings for each category
            center = np.random.randn(embedding_dim)
            center = center / np.linalg.norm(center)
            
            for j in range(samples_per_category):
                # Add noise around category center
                noise_level = 0.3 if j < samples_per_category // 2 else 0.6
                embedding = center + np.random.randn(embedding_dim) * noise_level
                embedding = embedding / np.linalg.norm(embedding)
                embeddings.append(embedding)
                labels.append(f"{category}_{j}")
        
        return np.array(embeddings, dtype=np.float32), labels
    
    def compute_semantic_similarity(self, embeddings: np.ndarray) -> np.ndarray:
        """Compute pairwise semantic similarity matrix.
        
        Args:
            embeddings: High-dimensional embeddings
            
        Returns:
            Similarity matrix
        """
        return cosine_similarity(embeddings)
    
    def compute_spatial_similarity(self, coordinates: np.ndarray) -> np.ndarray:
        """Compute pairwise spatial similarity from 2D coordinates.
        
        Args:
            coordinates: 2D projected coordinates
            
        Returns:
            Similarity matrix based on distance
        """
        # Compute pairwise distances
        distances = np.sqrt(((coordinates[:, None] - coordinates[None, :]) ** 2).sum(axis=2))
        
        # Convert to similarity (inverse distance with scaling)
        max_dist = distances.max()
        similarities = 1 - (distances / max_dist)
        
        return similarities
    
    def calculate_correlations(self, 
                              semantic_sim: np.ndarray, 
                              spatial_sim: np.ndarray) -> Tuple[float, float]:
        """Calculate global and local correlations.
        
        Args:
            semantic_sim: Semantic similarity matrix
            spatial_sim: Spatial similarity matrix
            
        Returns:
            Global correlation and local clustering score
        """
        # Global correlation: overall correlation between similarity matrices
        global_corr = np.corrcoef(semantic_sim.flatten(), spatial_sim.flatten())[0, 1]
        
        # Local clustering: correlation for nearest neighbors only
        k = 10  # Number of nearest neighbors
        local_scores = []
        
        for i in range(len(semantic_sim)):
            # Get k nearest neighbors in semantic space
            semantic_neighbors = np.argsort(semantic_sim[i])[-k-1:-1]
            
            # Get k nearest neighbors in spatial space
            spatial_neighbors = np.argsort(spatial_sim[i])[-k-1:-1]
            
            # Calculate overlap
            overlap = len(set(semantic_neighbors) & set(spatial_neighbors)) / k
            local_scores.append(overlap)
        
        local_clustering = np.mean(local_scores)
        
        return global_corr, local_clustering
    
    def validate_projection(self, 
                          embeddings: np.ndarray,
                          method: str = 'umap',
                          **kwargs) -> Tuple[np.ndarray, ValidationResult]:
        """Validate a projection method.
        
        Args:
            embeddings: High-dimensional embeddings to project
            method: Projection method ('umap', 'tsne', or 'custom')
            **kwargs: Additional parameters for projection method
            
        Returns:
            2D coordinates and validation results
        """
        # Project to 2D
        if method == 'umap':
            projector = UMAP(
                n_components=2,
                n_neighbors=kwargs.get('n_neighbors', 15),
                min_dist=kwargs.get('min_dist', 0.1),
                metric='cosine',
                random_state=42
            )
            coordinates = projector.fit_transform(embeddings)
            
        elif method == 'tsne':
            projector = TSNE(
                n_components=2,
                perplexity=kwargs.get('perplexity', 30),
                metric='cosine',
                random_state=42
            )
            coordinates = projector.fit_transform(embeddings)
            
        else:  # custom
            # Placeholder for custom projection method
            # For now, use PCA as a simple baseline
            from sklearn.decomposition import PCA
            projector = PCA(n_components=2, random_state=42)
            coordinates = projector.fit_transform(embeddings)
        
        # Normalize coordinates to [0, 1] for field mapping
        coordinates = (coordinates - coordinates.min(axis=0)) / (coordinates.max(axis=0) - coordinates.min(axis=0))
        
        # Compute similarities
        semantic_sim = self.compute_semantic_similarity(embeddings)
        spatial_sim = self.compute_spatial_similarity(coordinates)
        
        # Calculate correlations
        global_corr, local_clustering = self.calculate_correlations(semantic_sim, spatial_sim)
        
        # Determine if validation passed
        passed = (global_corr >= self.global_threshold and 
                 local_clustering >= self.local_threshold)
        
        result = ValidationResult(
            method=method,
            domain="test",
            global_correlation=global_corr,
            local_clustering=local_clustering,
            cross_domain_score=0.0,  # Will be set by cross-domain validation
            passed=passed,
            details={
                'n_samples': len(embeddings),
                'projection_params': kwargs,
                'thresholds': {
                    'global': self.global_threshold,
                    'local': self.local_threshold
                }
            }
        )
        
        return coordinates, result
    
    def cross_domain_validation(self, 
                              method: str = 'umap',
                              domains: List[str] = ['science', 'literature', 'history'],
                              **kwargs) -> Dict[str, ValidationResult]:
        """Validate across multiple domains.
        
        Args:
            method: Projection method to validate
            domains: List of domains to test
            **kwargs: Parameters for projection method
            
        Returns:
            Dictionary of results by domain
        """
        results = {}
        all_scores = []
        
        for domain in domains:
            print(f"\nValidating {method} on {domain} domain...")
            embeddings, labels = self.generate_test_embeddings(domain)
            coordinates, result = self.validate_projection(embeddings, method, **kwargs)
            result.domain = domain
            results[domain] = result
            all_scores.append(result.global_correlation)
            
            print(f"  Global correlation: {result.global_correlation:.3f}")
            print(f"  Local clustering: {result.local_clustering:.3f}")
            print(f"  Passed: {result.passed}")
        
        # Calculate cross-domain average
        cross_domain_score = np.mean(all_scores)
        
        # Update cross-domain scores
        for domain in results:
            results[domain].cross_domain_score = cross_domain_score
        
        # Overall pass: all domains must pass
        overall_pass = all(r.passed for r in results.values())
        
        print(f"\nCross-domain average: {cross_domain_score:.3f}")
        print(f"Overall validation: {'PASSED' if overall_pass else 'FAILED'}")
        
        return results
    
    def visualize_projection(self, 
                           embeddings: np.ndarray,
                           coordinates: np.ndarray,
                           labels: Optional[List[str]] = None,
                           title: str = "Semantic Projection Validation"):
        """Visualize the projection results.
        
        Args:
            embeddings: Original high-dimensional embeddings
            coordinates: 2D projected coordinates  
            labels: Optional labels for points
            title: Plot title
        """
        fig, axes = plt.subplots(1, 2, figsize=(15, 6))
        
        # Plot 1: 2D projection colored by categories
        if labels:
            categories = [l.split('_')[0] for l in labels]
            unique_categories = list(set(categories))
            colors = plt.cm.tab10(np.linspace(0, 1, len(unique_categories)))
            
            for i, cat in enumerate(unique_categories):
                mask = [c == cat for c in categories]
                axes[0].scatter(coordinates[mask, 0], coordinates[mask, 1], 
                              c=[colors[i]], label=cat, alpha=0.6, s=10)
            
            axes[0].legend()
        else:
            axes[0].scatter(coordinates[:, 0], coordinates[:, 1], alpha=0.6, s=10)
        
        axes[0].set_title("2D Projection")
        axes[0].set_xlabel("Dimension 1")
        axes[0].set_ylabel("Dimension 2")
        
        # Plot 2: Correlation heatmap
        semantic_sim = self.compute_semantic_similarity(embeddings[:100])  # Subset for visibility
        spatial_sim = self.compute_spatial_similarity(coordinates[:100])
        
        correlation_diff = np.abs(semantic_sim - spatial_sim)
        im = axes[1].imshow(correlation_diff, cmap='RdYlGn_r', vmin=0, vmax=1)
        axes[1].set_title("Similarity Preservation\n(red = poor, green = good)")
        axes[1].set_xlabel("Sample index")
        axes[1].set_ylabel("Sample index")
        plt.colorbar(im, ax=axes[1])
        
        plt.suptitle(title)
        plt.tight_layout()
        plt.show()


def main():
    """Main validation script."""
    parser = argparse.ArgumentParser(description="Validate semantic projection for LIMINAL")
    parser.add_argument('--method', type=str, default='all', 
                       choices=['umap', 'tsne', 'custom', 'all'],
                       help='Projection method to validate')
    parser.add_argument('--samples', type=int, default=1000,
                       help='Number of samples to use for validation')
    parser.add_argument('--visualize', action='store_true',
                       help='Show visualization plots')
    parser.add_argument('--save-results', type=str,
                       help='Save results to JSON file')
    
    args = parser.parse_args()
    
    # Initialize validator
    validator = SemanticProjectionValidator(n_samples=args.samples)
    
    # Determine methods to test
    if args.method == 'all':
        methods = ['umap', 'tsne', 'custom']
    else:
        methods = [args.method]
    
    # Run validation
    all_results = {}
    
    for method in methods:
        print(f"\n{'='*60}")
        print(f"Validating {method.upper()} projection method")
        print(f"{'='*60}")
        
        # Method-specific parameters
        if method == 'umap':
            params = {'n_neighbors': 15, 'min_dist': 0.1}
        elif method == 'tsne':
            params = {'perplexity': 30}
        else:
            params = {}
        
        # Run cross-domain validation
        results = validator.cross_domain_validation(method, **params)
        all_results[method] = results
        
        # Visualize if requested
        if args.visualize:
            embeddings, labels = validator.generate_test_embeddings('mixed')
            coordinates, _ = validator.validate_projection(embeddings, method, **params)
            validator.visualize_projection(embeddings, coordinates, labels, 
                                         f"{method.upper()} Projection Validation")
    
    # Summary
    print(f"\n{'='*60}")
    print("VALIDATION SUMMARY")
    print(f"{'='*60}")
    
    for method in all_results:
        avg_global = np.mean([r.global_correlation for r in all_results[method].values()])
        avg_local = np.mean([r.local_clustering for r in all_results[method].values()])
        all_pass = all(r.passed for r in all_results[method].values())
        
        print(f"\n{method.upper()}:")
        print(f"  Average global correlation: {avg_global:.3f}")
        print(f"  Average local clustering: {avg_local:.3f}")
        print(f"  Overall: {'PASSED' if all_pass else 'FAILED'}")
    
    # Save results if requested
    if args.save_results:
        results_dict = {}
        for method in all_results:
            results_dict[method] = {}
            for domain, result in all_results[method].items():
                results_dict[method][domain] = {
                    'global_correlation': result.global_correlation,
                    'local_clustering': result.local_clustering,
                    'passed': result.passed
                }
        
        with open(args.save_results, 'w') as f:
            json.dump(results_dict, f, indent=2)
        print(f"\nResults saved to {args.save_results}")


if __name__ == "__main__":
    main()