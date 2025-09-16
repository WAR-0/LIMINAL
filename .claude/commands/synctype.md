---
name: synctype
description: Synchronize Rust and TypeScript types
---

Synchronize types for: $ARGUMENTS

Process:
1. Identify Tauri command structs
2. Add serde derives
3. Configure ts-rs
4. Generate TypeScript
5. Update service layer
6. Fix mismatches
7. Test serialization

Critical types:
- AgentMessage
- Priority
- Territory
- LeaseRequest
- RouterEvent

Verify:
- Field name matching
- Optional handling
- Enum representation
- Nested structures