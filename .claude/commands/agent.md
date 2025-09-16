---
name: agent
description: Implement new agent type
---

Implement new agent: $ARGUMENTS

Steps:
1. Create PTY configuration in src-tauri/src/agents/
2. Define message types in src-tauri/src/router/messages.rs
3. Add territory requirements in src-tauri/src/territory/
4. Generate TypeScript types with ts-rs
5. Create React component in src/components/agents/
6. Add integration tests
7. Test resource limits
8. Verify PTY isolation
9. Check message routing
10. Update documentation

Ensure:
- Resource limits enforced
- Lease system integration
- Message validation
- Error handling
- Performance targets met