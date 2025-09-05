# Contributing to LIMINAL
## Physics-Based Memory Architecture Development Guidelines

**Document Version**: 1.0  
**Primary Developer**: Single developer with AI assistance  
**Community Role**: Advisory and specialized contributions

---

## Project Overview

LIMINAL is a research prototype implementing physics-based memory architecture for large language models. The project prioritizes functional memory improvements with consciousness research as secondary exploration. Contributions should align with this "memory architecture first" principle.

### Current Development Status
- **Phase**: Pre-development (documentation complete, ready for implementation)
- **Timeline**: 18-week MVP development cycle
- **Focus**: Single developer with community input and specialized contributions
- **Success Criteria**: Functional improvements over baseline systems with statistical validation

---

## Contribution Philosophy

### Primary Development Model
LIMINAL uses a **single developer + AI assistance** model with selective community contributions. This ensures focused development while leveraging community expertise for specific challenges.

### Community Contribution Areas
1. **Technical Review**: Expert review of physics, AI, and consciousness research approaches
2. **Specialized Implementation**: Contributions to specific technical components
3. **Testing and Validation**: Independent validation of claims and benchmarks
4. **Documentation**: Improvements to technical documentation and guides
5. **Research Integration**: Connections to relevant academic research and findings

### What We're NOT Seeking
- **Core Architecture Changes**: Fundamental approach is established through Multi-AI Assessment
- **Scope Expansion**: Project maintains strict scope discipline
- **Consciousness Claims**: No additional consciousness implementation attempts
- **Alternative Approaches**: Focus is on validating current approach, not exploring alternatives

---

## Contribution Types

### 1. Code Contributions

#### Accepted Code Contributions
- **Bug Fixes**: Clear bugs in physics engine, interface, or memory systems
- **Performance Optimizations**: GPU optimization, memory management, computational efficiency
- **Hardware Support**: Additional GPU profiles, CPU fallback improvements
- **Testing Infrastructure**: Unit tests, integration tests, benchmark improvements
- **Monitoring Enhancements**: Visualization improvements, metrics collection

#### Code Contribution Process
1. **Issue Discussion**: Discuss proposed changes in GitHub Issues before implementation
2. **Scope Alignment**: Ensure contribution aligns with MVP goals and resource allocation (70/20/10)
3. **Technical Review**: Core developer reviews technical approach and integration impact
4. **Implementation**: Follow coding standards and testing requirements
5. **Integration**: Core developer handles final integration and testing

### 2. Research Contributions

#### Valued Research Contributions
- **Literature Review**: Relevant papers and research supporting or challenging LIMINAL approach
- **Theoretical Analysis**: Mathematical or theoretical insights into physics-consciousness connections
- **Experimental Design**: Improvements to benchmarking and validation methodologies
- **Risk Assessment**: Additional risk identification and mitigation strategies

#### Research Contribution Process
1. **Research Proposal**: Submit research questions or literature relevant to LIMINAL
2. **Relevance Assessment**: Core developer evaluates relevance to current development phase
3. **Integration Planning**: Determine how research insights should influence implementation
4. **Documentation**: Research findings documented in appropriate project sections

### 3. Review and Validation

#### Expert Review Areas
- **Physics Implementation**: Numerical methods, field dynamics, stability analysis
- **AI Architecture**: LLM integration, attention mechanisms, memory systems  
- **Consciousness Theory**: Theoretical foundations, claims validation, scientific rigor
- **Statistical Methods**: Benchmarking approaches, significance testing, effect size analysis

#### Review Process
1. **Review Request**: Core developer requests review of specific components or documents
2. **Expert Assessment**: Independent review with written feedback and recommendations
3. **Discussion**: Open discussion of findings and implications
4. **Integration**: Core developer integrates feedback into development or documentation

---

## Development Standards

### Code Quality Standards

**Style Guidelines**:
- **Python Style**: PEP 8 compliance with Black formatting
- **Documentation**: Google-style docstrings for all public functions
- **Type Hints**: Full type annotation using mypy
- **Testing**: >90% code coverage for core components

**Example Code Style**:
```python
def update_field_state(
    field_state: FieldState,
    mass_distribution: np.ndarray,
    timestep: float = 0.1
) -> FieldState:
    """Update physics field state with new mass distribution.
    
    Args:
        field_state: Current field state to update
        mass_distribution: New mass distribution [256, 256] 
        timestep: Integration timestep in seconds
        
    Returns:
        Updated field state with new dynamics
        
    Raises:
        ValueError: If mass_distribution has incorrect shape
        PhysicsError: If field update creates instability
    """
    if mass_distribution.shape != (256, 256):
        raise ValueError(f"Expected shape (256, 256), got {mass_distribution.shape}")
    
    # Implementation with proper error handling
    try:
        updated_field = self._solve_field_equation(field_state, mass_distribution, timestep)
        return self._validate_field_stability(updated_field)
    except Exception as e:
        raise PhysicsError(f"Field update failed: {e}")
```

### Testing Requirements

**Unit Tests**:
- All physics functions must have unit tests with edge cases
- Interface functions tested with multiple projection methods
- Memory system tested with various consolidation scenarios
- Performance tests for all critical path operations

**Integration Tests**:
- End-to-end system functionality
- Cross-component interaction validation
- Hardware compatibility testing
- Long-term stability testing

**Example Test Structure**:
```python
class TestPhysicsEngine:
    @pytest.fixture
    def physics_engine(self):
        return PhysicsEngine(field_resolution=(256, 256))
    
    def test_field_update_conserves_mass(self, physics_engine):
        """Test that field updates conserve total mass."""
        initial_mass = np.random.random((256, 256))
        field_state = physics_engine.update_field(initial_mass)
        
        assert np.isclose(
            np.sum(initial_mass), 
            np.sum(field_state.mass),
            rtol=1e-6
        ), "Mass conservation violated"
    
    @pytest.mark.gpu
    def test_gpu_acceleration_matches_cpu(self, physics_engine):
        """Test GPU results match CPU within numerical precision."""
        mass_dist = np.random.random((256, 256))
        
        cpu_result = physics_engine.update_field(mass_dist, use_gpu=False)
        gpu_result = physics_engine.update_field(mass_dist, use_gpu=True)
        
        assert np.allclose(cpu_result.field, gpu_result.field, rtol=1e-5)
```

### Documentation Standards

**Code Documentation**:
- All public functions must have docstrings
- Complex algorithms require inline comments explaining approach
- Configuration options documented with examples
- API documentation generated automatically from docstrings

**Architecture Documentation**:
- Design decisions documented with rationale
- Component interfaces clearly specified
- Performance characteristics documented
- Failure modes and recovery procedures documented

---

## Branching Strategy

### Branch Types
- **main**: Stable development branch, always deployable
- **feature/**: Feature development branches
- **bugfix/**: Bug fix branches  
- **research/**: Research exploration branches (no merge to main)

### Development Workflow
1. **Issue Creation**: Create GitHub issue describing proposed change
2. **Branch Creation**: Create feature branch from main
3. **Development**: Implement changes following code standards
4. **Testing**: Ensure all tests pass, add new tests as needed
5. **Pull Request**: Submit PR with description linking to issue
6. **Review**: Code review and discussion
7. **Integration**: Core developer handles merge and integration testing

### Commit Message Standards
```
type(scope): brief description

Longer description if needed explaining the change and its rationale.

Fixes #issue_number
```

**Commit Types**:
- `feat`: New feature implementation
- `fix`: Bug fix
- `perf`: Performance improvement
- `test`: Test additions or improvements
- `docs`: Documentation changes
- `refactor`: Code refactoring without behavior change

---

## Issue Management

### Issue Types

**Bug Reports**:
```markdown
## Bug Description
Brief description of the issue

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen

## Actual Behavior  
What actually happens

## Environment
- OS: 
- Python Version:
- GPU:
- LIMINAL Version:

## Additional Context
Any other relevant information
```

**Feature Requests**:
```markdown
## Feature Description
Clear description of proposed feature

## Motivation
Why is this feature needed?

## Proposed Implementation
High-level implementation approach

## Acceptance Criteria
How will we know this feature is complete?

## Impact Assessment
How does this affect existing functionality?
```

**Research Discussions**:
```markdown
## Research Question
What research question or finding is being discussed?

## Relevant Literature
Links to relevant papers or research

## Implications for LIMINAL
How does this affect LIMINAL's approach?

## Proposed Actions
What changes or investigations are suggested?
```

### Issue Labels
- **Priority**: `critical`, `high`, `medium`, `low`
- **Type**: `bug`, `feature`, `research`, `documentation`
- **Component**: `physics`, `interface`, `memory`, `llm`, `monitoring`
- **Status**: `needs-review`, `in-progress`, `blocked`, `ready-to-merge`

---

## Communication Guidelines

### GitHub Discussions
- **Technical Discussions**: Architecture decisions, implementation approaches
- **Research Discussions**: Consciousness theory, physics approaches, literature review
- **Community Q&A**: General questions about the project and its goals

### Code Review Process
- **Constructive Feedback**: Focus on code quality, performance, and maintainability
- **Clear Communication**: Specific suggestions with rationale
- **Respectful Dialogue**: Professional communication acknowledging different perspectives
- **Learning Opportunity**: Treat reviews as mutual learning experiences

### Research Discussion Guidelines
- **Evidence-Based**: Cite relevant literature and research
- **Scope Awareness**: Consider impact on development timeline and resources
- **Constructive Criticism**: Focus on improving the project, not proving points
- **Scientific Rigor**: Maintain high standards for claims and evidence

---

## Recognition and Attribution

### Contributor Recognition
- **Code Contributors**: Listed in CONTRIBUTORS.md with specific contributions
- **Research Contributors**: Acknowledged in research documentation and potential publications
- **Review Contributors**: Acknowledged in relevant technical documentation
- **Community Contributors**: Recognized for ongoing support and discussion

### Publication and Citation
- **Research Publications**: Major contributors included as co-authors where appropriate
- **Technical Documentation**: Contributors credited for significant documentation improvements
- **Open Source Recognition**: Standard open source attribution practices followed

---

## Getting Started as a Contributor

### 1. Understand the Project
- Read the Multi-AI Assessment Synthesis documents thoroughly
- Understand the "memory architecture first" principle
- Review the technical architecture and MVP roadmap

### 2. Set Up Development Environment
- Follow the Developer Setup Guide completely
- Verify all tests pass in your environment
- Run basic functionality tests to ensure system works

### 3. Find Contribution Opportunities
- Review open issues marked `good-first-issue` or `help-wanted`
- Join discussions in GitHub Discussions
- Offer expertise in your areas of specialization

### 4. Start Small
- Begin with documentation improvements or small bug fixes
- Build familiarity with the codebase and development process
- Gradually take on larger contributions as you understand the project better

### 5. Engage with the Community
- Participate in technical discussions
- Offer constructive feedback on proposed changes
- Share relevant research or insights from your expertise

---

This contributing guide ensures productive community involvement while maintaining focus on LIMINAL's core development goals and timeline.