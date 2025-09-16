---
name: router
description: Work on message routing
---

Message routing task: $ARGUMENTS

Analysis:
1. Check queue performance
2. Verify priority ordering
3. Test acknowledgments
4. Profile latency
5. Check for drops/duplicates
6. Review dead letter queue

Implementation:
1. Use BTreeMap for priority
2. Implement backpressure
3. Add monitoring hooks
4. Test concurrent load
5. Verify <10ms latency

Testing:
- Unit test queue operations
- Integration test routing
- Load test with 10+ agents
- Chaos test with failures