# Implementation: Runbook Parser

## Overview

Implemented the foundational data model and Markdown parser for the director orchestration layer. This enables automated runbook execution by converting Markdown runbooks into executable task graphs.

## Components

### Data Models (`director/runbook.rs`)

**AgentRole Enum**
- Represents specialist roles: Systems, Interface, Router, Testing, Research, Director
- Includes `from_str()` for flexible parsing

**TurnStatus Enum**
- Tracks execution state: Pending, InProgress, Completed, Failed, Blocked
- Used by orchestration layer to manage task lifecycle

**Turn Struct**
- Core execution unit with id, specialist, prompt, acceptance criteria
- Supports parallel groups for concurrent execution
- Maintains dependency graph for sequential ordering
- Extensible metadata for custom fields

**Runbook Struct**
- Top-level container with epoch_id, goal, turns
- `build_dependency_graph()` constructs execution order from parallel groups
- `get_executable_turns()` returns turns ready for execution based on completed dependencies

### Parser (`director/parser.rs`)

**RunbookParser**
- Uses `pulldown-cmark` for Markdown processing
- Extracts structured data from formatted Markdown sections
- Validates required fields and returns helpful errors

**Parsing Logic**
- Runbook title: `# Runbook: [Name]` → epoch_id
- Epoch goal: `**Epoch Goal:** [Description]`
- Turn headers: `## Turn N — [Role]` → turn id and specialist
- Metadata fields: `**Specialist:**`, `**Parallel Group:**`, `**Dependencies:**`
- Prompt blocks: Multi-line `> ` quoted sections
- Acceptance criteria: Bulleted lists under `**Acceptance:**`

**Dependency Graph**
- Turns in same parallel group can execute concurrently
- Turns in different parallel groups have implicit dependencies on all prior groups
- Sequential turns (no parallel group) depend on all previous turns

## Implementation Details

### Parallel Groups
- `parallel_group: None` → sequential execution
- `parallel_group: Some(N)` → part of group N
- Lower group numbers execute before higher ones
- All turns in a group can run simultaneously

### Error Handling
- `ParseError::MissingField` for required data
- `ParseError::InvalidFormat` for malformed structure
- `ParseError::UnknownRole` for unrecognized specialists
- `ParseError::InvalidTurnNumber` for turn header parsing

### Testing
- Unit tests cover basic parsing, parallel groups, error cases
- Tests validate dependency graph construction
- Tests verify executable turn selection logic

## Usage Example

```rust
use liminal_v1::director::{RunbookParser, TurnStatus};

let content = std::fs::read_to_string("runbook.md")?;
let parser = RunbookParser::new(content);
let mut runbook = parser.parse()?;

// Get turns ready to execute
let executable = runbook.get_executable_turns();
for turn in executable {
    println!("Ready: Turn {} - {:?}", turn.id, turn.specialist);
}

// Mark turn completed
runbook.turns[0].status = TurnStatus::Completed;
```

## Performance Characteristics

- Parser: O(n) where n = content length
- Dependency graph: O(t²) where t = number of turns
- Executable turns: O(t) with completed set lookup

## Future Extensions

- Add duration tracking for turn execution
- Support conditional dependencies (not just implicit)
- Add validation hooks for custom constraints
- Implement runbook templates and macros
- Support nested parallel groups or DAG dependencies