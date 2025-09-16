---
### Analysis of: `conceptual/01_vision.md`

**Comparison Table:**

| Criteria                          | `docs-claude` | `docs-codex` | `docs-gemini` |
| --------------------------------- | ------------- | ------------ | ------------- |
| 1. Clarity and Cohesion           | Excellent narrative with crisp throughline from problem → paradigm → UX → metrics. | Very clear; adds explicit role handoffs and triggers. Slightly denser style. | Clear and motivational; strong Human/Director delineation, minor truncation artifacts in one subsection. |
| 2. Technical Depth and Accuracy   | Solid for a vision doc; correctly frames router, soft leases, clones, MCP-as-tool. | Good depth; concrete handoff/escalation thresholds align with research. | Good depth; touches research themes but stays conceptual. |
| 3. Completeness and Mandate Adherence | Covers Director roles, async clones, territories, success metrics, MCP layer. | Covers all mandates plus explicit handoffs/escapes. | Covers mandates; role delineation is strong. |
| 4. Integration of Source Research | Moderate: references async-first, leases, MCP-as-tools; lightly references prior patterns. | Moderate-strong: adds concrete escalation thresholds (e.g., deadlocks >60s, 2+ queued) consistent with research. | Moderate: aligns with research but fewer explicit anchors. |
| 5. Structural Integrity           | Strong Diátaxis fit (Conceptual/Why). | Strong Diátaxis fit (Conceptual). | Strong Diátaxis fit (Conceptual). |

**Strengths & Weaknesses:**

*   **Claude Version:**
    *   **Strengths:** Compelling, cohesive story; clearly explains Unified Message Router, async clones, soft leases; strong market positioning; concrete success metrics.
    *   **Weaknesses:** Less explicit on handoff/escalation triggers than Codex; a minor mid-section truncation around async rationale.
*   **Codex Version:**
    *   **Strengths:** Best articulation of Human Director vs Director Agent with explicit handoff points and escalation triggers tied to thresholds (e.g., 2+ queued, >60s deadlocks).
    *   **Weaknesses:** Slightly more utilitarian tone; fewer inspirational touches than Claude.
*   **Gemini Version:**
    *   **Strengths:** Clear role delineation and partnership framing; maintains consistent conceptual scope.
    *   **Weaknesses:** A small formatting/truncation artifact in one role subsection; fewer concrete thresholds.

**Synthesis Recommendation:**

Use the **Claude** version as the base. Merge in the following sections:
- The explicit list of **handoff points and escalation triggers** from the **Codex** version.
- The short role delineation framing paragraph from the **Gemini** version for added clarity.
- Keep the Claude narrative arc and success metrics; ensure minor truncation is resolved and the async rationale lines are complete.
---

### Analysis of: `conceptual/02_architecture.md`

**Comparison Table:**

| Criteria                          | `docs-claude` | `docs-codex` | `docs-gemini` |
| --------------------------------- | ------------- | ------------ | ------------- |
| 1. Clarity and Cohesion           | Very strong; clean structure; deep sections and diagrams. | Strong, but one late “3.6 Context Snapshot” section is appended after a conclusion. | Strong and well-paced; embeds roles early for context. |
| 2. Technical Depth and Accuracy   | Excellent: full Rust shapes, config knobs, token bucket, territory hash, lease policy, event parser, persistence patterns. | Good+: includes state patterns and physics notes; fewer concrete config knobs. | Good+: standout explanation of snapshot performance and memory (Arc, COW) with <10ms target. |
| 3. Completeness and Mandate Adherence | Comprehensive: Director roles, message priority, context snapshots, territory hash, performance targets. | Broad coverage; includes 2D physics-coordination and territory hash. | Broad coverage; strong snapshot implementation details; roles and handoffs embedded. |
| 4. Integration of Source Research | High: adopts Arc<RwLock>, event parsing, snapshot thresholds, token buckets, spatial hash; aligns with context window, PTY, and state mgmt research. | Medium-high: integrates state and adapter patterns; physics note from UNCAN. | High on snapshot/memory guidance; aligns with context optimization research. |
| 5. Structural Integrity           | Strong Diátaxis fit (Conceptual/Architecture reference hybrid) with clear subsections. | Good but structural hiccup near the end; still a fit for Conceptual/Reference. | Strong; clear sections and consistent flow. |

**Strengths & Weaknesses:**

*   **Claude Version:**
    *   **Strengths:** Most complete architecture; explicit configuration (e.g., escalation thresholds, diff thresholds, latency targets); concrete Rust constructs; rich diagrams and message flows; well-defined Director separation and persistence.
    *   **Weaknesses:** Very dense; could benefit from a compact snapshot memory subsection.
*   **Codex Version:**
    *   **Strengths:** Clear state pattern and adapter set; includes 2D physics-coordination (SwarmBehaviorEngine/SpatialHash) note; pragmatic territory and lease policy code; good diagrams.
    *   **Weaknesses:** One structural inconsistency (post-conclusion section); fewer explicit performance/config thresholds.
*   **Gemini Version:**
    *   **Strengths:** Best treatment of context snapshot performance and memory (serde+bincode, differential vs full, Arc sharing, COW); crisp role separation section.
    *   **Weaknesses:** Slightly lighter on token bucket and priority routing code; fewer low-level config knobs.

**Synthesis Recommendation:**

Use the **Claude** version as the base. Merge in the following sections:
- The **Context Snapshot Implementation** and **Memory Management** details from the **Gemini** version (Arc sharing, COW, <10ms budget rationale).
- The concise **TerritoryHash / 2D physics-coordination** note from the **Codex** version (as a sidebar under Territory Manager) to acknowledge visualization/perf implications.
- Retain Claude’s diagrams and all concrete Rust structs/configuration. Remove Codex’s stray postscript section by integrating its content in the main snapshot chapter.
---

### Analysis of: `reference/01_agent_capabilities.md`

**Comparison Table:**

| Criteria                          | `docs-claude` | `docs-codex` | `docs-gemini` |
| --------------------------------- | ------------- | ------------ | ------------- |
| 1. Clarity and Cohesion           | Strong; clear decision trees, code examples, and practices. | Clear; mirrors structure; slightly leaner. | Clear; mirrors structure; consistent tone. |
| 2. Technical Depth and Accuracy   | Excellent: lease heuristics with config keys, comms strategy, snapshot diffs with thresholds, KPI metrics, Director agent extra capabilities. | Good: solid heuristics and flows; fewer thresholds and Director-specific capabilities. | Good: similar to Codex; consistent with architecture doc. |
| 3. Completeness and Mandate Adherence | Comprehensive: lease, clones vs direct, snapshots, priorities, recovery, KPIs, Director agent extras. | High but missing the Director agent capability section and explicit perf targets. | High; similar to Codex; matches mandates. |
| 4. Integration of Source Research | High: thresholds and patterns match research (diff <100 changes, async-first, escalation rules). | Medium-high: aligns, but fewer explicit ties to research limits. | Medium-high: aligns with snapshot guidance from research. |
| 5. Structural Integrity           | Strong Diátaxis fit (Reference/How-to hybrid). | Strong fit. | Strong fit. |

**Strengths & Weaknesses:**

*   **Claude Version:**
    *   **Strengths:** Most actionable; richly specified thresholds and handlers; adds Director Agent capabilities and system health monitoring; KPI targets align with performance research.
    *   **Weaknesses:** Lengthy; could consolidate repeated rationale.
*   **Codex Version:**
    *   **Strengths:** Clean and readable walkthrough of the same decisions and flows; great defaults.
    *   **Weaknesses:** Fewer explicit thresholds/targets; lacks Director-specific capability guidance.
*   **Gemini Version:**
    *   **Strengths:** Consistent with architecture’s snapshot guidance; clear diagrams.
    *   **Weaknesses:** Similar omissions as Codex on Director-specific guidance.

**Synthesis Recommendation:**

Use the **Claude** version as the base. Merge in the following sections:
- Keep Claude’s full decision matrices and KPIs; add a compact “at-a-glance” summary list from **Codex** to improve scannability.
- Ensure snapshot guidance references the architecture’s **differential vs full** policy and <10ms target (aligning with **Gemini** and research).
- Preserve the Director Agent capabilities section from **Claude**.
---

### Analysis of: `reference/02_message_priority_spec.md`

**Comparison Table:**

| Criteria                          | `docs-claude` | `docs-codex` | `docs-gemini` |
| --------------------------------- | ------------- | ------------ | ------------- |
| 1. Clarity and Cohesion           | Strong; table-driven and code-backed; comprehensive. | Very clear conceptual framing with cross-domain lessons. | Very clear conceptual framing, mirrors Codex. |
| 2. Technical Depth and Accuracy   | Excellent: explicit queues, token bucket, aging, inheritance, agent limits, YAML config, latency budgets. | Good+: thorough narrative spec without low-level code/config. | Good+: thorough narrative spec without low-level code/config. |
| 3. Completeness and Mandate Adherence | Complete: 4+1 levels incl. DirectorOverride, escalation, inversion policy, privileges, performance targets. | High completeness conceptually. | High completeness conceptually. |
| 4. Integration of Source Research | High: directly operationalizes research (token buckets, aging, inheritance, quotas). | High: closely reflects research rationale and taxonomy. | High: closely reflects research rationale and taxonomy. |
| 5. Structural Integrity           | Strong Diátaxis fit (Reference/Specification). | Strong Diátaxis fit (Reference/Conceptual primer). | Strong Diátaxis fit (Reference/Conceptual primer). |

**Strengths & Weaknesses:**

*   **Claude Version:**
    *   **Strengths:** Most implementable; includes quotas, inheritance, escalation timers, YAML config, and perf budgets that map 1:1 to research.
    *   **Weaknesses:** Dense; could benefit from a short preface.
*   **Codex Version:**
    *   **Strengths:** Best top-down framing with inspirations (MQs, OS schedulers, QoS, human systems) that motivate the design.
    *   **Weaknesses:** Less code/config concreteness than Claude.
*   **Gemini Version:**
    *   **Strengths:** Mirrors Codex narrative; well structured.
    *   **Weaknesses:** Similar lack of concrete structures.

**Synthesis Recommendation:**

Use the **Claude** version as the base. Merge in the following sections:
- The concise “Lessons and analogies” introduction from the **Codex** version to set context.
- Keep all Claude code/config sections (token bucket, aging, inheritance, privileges, YAML) to preserve implementability and performance targets.
---

### Analysis of: `testing/integration_tests.md`

**Comparison Table:**

| Criteria                          | `docs-claude` | `docs-codex` | `docs-gemini` |
| --------------------------------- | ------------- | ------------ | ------------- |
| 1. Clarity and Cohesion           | Extensive but logically grouped with clear objectives and code-like examples. | Concise scenarios with assumptions and metrics; easy to scan. | Scenario-driven and readable; higher-level. |
| 2. Technical Depth and Accuracy   | Excellent: throughput, latency p99, clone <10ms, spatial hash O(1), starvation prevention, crash recovery, deadlock resolution. | Good: covers core flows and metrics; lighter on implementation detail. | Fair: outlines key scenarios but omits perf harness and deep assertions. |
| 3. Completeness and Mandate Adherence | Very high; end-to-end cycles, recovery, priority queue behavior, fairness, generators. | High; covers main categories succinctly. | Moderate; misses detailed perf and recovery coverage. |
| 4. Integration of Source Research | High: perf targets and policies trace to research (context snapshots, state mgmt, priority quotas). | Medium-high: aligns but fewer explicit targets. | Medium: aligns conceptually; fewer specifics. |
| 5. Structural Integrity           | Strong Diátaxis fit (Reference/Testing spec). | Strong fit (Reference/Checklist style). | Adequate fit (Reference/Outline). |

**Strengths & Weaknesses:**

*   **Claude Version:**
    *   **Strengths:** Most comprehensive test matrix with performance benchmarks, fairness/starvation tests, spatial hash validation, crash recovery, and E2E coverage.
    *   **Weaknesses:** Lengthy; could add a lightweight “assumptions” preface to speed onboarding.
*   **Codex Version:**
    *   **Strengths:** Clear, outcome-focused scenarios; good at-a-glance acceptance criteria and instrumentation pointers.
    *   **Weaknesses:** Less code-like coverage; omits some micro-benchmarks and generators.
*   **Gemini Version:**
    *   **Strengths:** Good simple scenarios and outcomes.
    *   **Weaknesses:** Some proposed behaviors (e.g., immediate high-priority preemption) may contradict soft-lease policy; lacks perf targets and tooling.

**Synthesis Recommendation:**

Use the **Claude** version as the base. Merge in the following sections:
- The compact “Test Harness Assumptions” header and scenario summaries from **Codex** for readability.
- Keep Claude’s performance targets (throughput, p99 latency, clone spawn <10ms) and starvation/fairness tests.
- Adapt Gemini’s scenario names where helpful, but align expected outcomes with the lease negotiation and escalation policy (defer/escalate rather than unconditional preemption).
---

## Cross-Cutting Synthesis Notes

- Terminology and priorities: Standardize on 4+1 levels `Info`, `Coordinate`, `Blocking`, `Critical`, `DirectorOverride` everywhere; ensure clone agents are capped at `Coordinate` unless escalated via parent, per research and Claude spec.
- Performance targets: Preserve key targets from Claude and research: message routing <1ms, clone snapshot <10ms (p99 <15ms), territory lookup O(1), token-bucket checks <50μs, escalation checks <100μs.
- Context snapshots: Adopt Gemini’s memory-sharing guidance (Arc sharing, COW) and Claude’s diff threshold default (100 logical changes) with bincode serialization.
- State management: Keep Arc<RwLock> pattern with “no await under lock” and persist outside the lock (Claude/Codex + state_mgmt research).
- PTY/event protocol: Retain `<LIMINAL_EVENT>` wrapper and buffered streaming parser patterns per research; reference structured events consistently across docs.
- Human/AI Director split: Use Codex’s discrete handoff/escalation bullets in Vision and Architecture; keep Claude’s authority boundaries and escalation rules in Reference.
- UI/observability: Optionally link to research UI patterns to justify cockpit elements and timeline/queue monitors.

## Proposed Base Selection Summary

- `conceptual/01_vision.md`: Base = Claude; merge Codex handoffs/triggers; add Gemini role framing.
- `conceptual/02_architecture.md`: Base = Claude; merge Gemini snapshot memory mgmt; add Codex physics/territory-hash sidebar; normalize structure.
- `reference/01_agent_capabilities.md`: Base = Claude; add Codex “at-a-glance” summaries; ensure snapshot and KPI targets align with Architecture.
- `reference/02_message_priority_spec.md`: Base = Claude; prepend Codex “lessons and analogies” context; keep Claude’s code/config.
- `testing/integration_tests.md`: Base = Claude; prepend Codex harness assumptions; reconcile Gemini scenarios to policy and keep perf targets.

This plan preserves the most technically rigorous material (primarily from Claude), enhances clarity and motivational framing (Codex/Gemini), and aligns tightly with the research corpus for correctness.

