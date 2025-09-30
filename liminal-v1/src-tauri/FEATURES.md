# LIMINAL V1 Feature Flags

## Overview
The LIMINAL codebase contains several experimental modules that are not yet integrated into the main application flow. These modules are maintained but marked with `#[allow(dead_code)]` to reduce compilation noise while development continues on the core MVP features.

## Current Status

### Active Modules (Always Enabled)
- **agent**: PTY subprocess management for agent isolation
- **router**: Priority-based message routing with 4-level queues
- **territory**: Lease-based resource management
- **metrics**: Performance monitoring and collection
- **executor**: Work-stealing task executor with runtime management
- **config**: Application configuration parsing

### Experimental Modules (Feature-Gated)
These modules are compiled but not actively used in the current MVP:

#### `consensus` Feature
- **Purpose**: Distributed consensus for multi-node agent coordination
- **Status**: Scaffolded, not integrated
- **Components**: ConsensusBroker, quorum voting
- **Enable**: `cargo build --features consensus`

#### `ledger` Feature
- **Purpose**: Event sourcing and replay capabilities
- **Status**: Partially integrated, writes events but replay not active
- **Components**: LedgerWriter, LedgerReader, ReplayCoordinator
- **Enable**: `cargo build --features ledger`

#### `health` Feature
- **Purpose**: System health monitoring and alerting
- **Status**: Scaffolded, basic threshold checks implemented
- **Components**: HealthMonitor, alert generation
- **Enable**: `cargo build --features health`

#### `experimental` Feature
- **Purpose**: Enable all experimental features at once
- **Enable**: `cargo build --features experimental`

## Usage

### Default Build (MVP Only)
```bash
cargo build
```

### With Specific Features
```bash
cargo build --features ledger
cargo build --features "ledger,health"
```

### With All Experimental Features
```bash
cargo build --features experimental
```

## Migration Plan

1. **Current (MVP)**: Core modules active, experimental modules present but dormant
2. **Phase 2**: Activate ledger for event replay demonstrations
3. **Phase 3**: Enable health monitoring in production builds
4. **Phase 4**: Implement consensus when multi-node support is added

## Dead Code Warnings

The following warnings are expected and can be ignored:
- Fields in config structs (reserved for future use)
- Unused imports from experimental modules
- Methods in experimental modules not yet called

These are intentionally retained to preserve the architecture design while focusing on MVP delivery.