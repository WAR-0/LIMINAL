# LIMINAL System Data Flow Diagram

## High-Level Request Flow Architecture

```mermaid
graph TD
    %% Primary Components
    HD[HumanDirector]
    UI[UserInterface]
    UMR[UnifiedMessageRouter]
    AP[AgentProcess<br/>e.g., Frontend Agent]
    PL[PersistenceLayer<br/>Turn Database]
    DA[DirectorAgent]

    %% Primary Data Flow
    HD -->|1. User Request| UI
    UI -->|2. Processed Request| UMR
    UMR -->|3. Route Task| AP
    AP -->|4a. Read State| PL
    AP -->|4b. Write State| PL
    AP -->|8. Completion Update| UMR
    UMR -->|9. Status Update| UI
    UI -->|10. Final Result| HD

    %% Escalation Paths (dotted lines)
    AP -.->|5. Escalation Request<br/>Problem/Conflict| DA
    DA -.->|6. Critical Decision<br/>Required| HD
    HD -.->|7. Decision Response| DA
    DA -.->|Resolution| AP

    %% Styling
    style UMR fill:#ff9999,stroke:#333,stroke-width:3px
    style HD fill:#99ccff,stroke:#333,stroke-width:2px
    style UI fill:#99ccff,stroke:#333,stroke-width:2px
    style PL fill:#ffcc99,stroke:#333,stroke-width:2px
    style DA fill:#ccffcc,stroke:#333,stroke-width:2px
    style AP fill:#ffffcc,stroke:#333,stroke-width:2px

    %% Legend
    subgraph Legend
        L1[Primary Flow]
        L2[Escalation Path]
    end

    style Legend fill:#f9f9f9,stroke:#666,stroke-width:1px
```

## Data Flow Description

### Primary Request Path
1. **User Request**: HumanDirector initiates a request
2. **UI Processing**: UserInterface processes and formats the request
3. **Routing**: UnifiedMessageRouter determines appropriate agent
4. **Execution**: AgentProcess executes task with persistence layer support
   - 4a: Reads current state from Turn Database
   - 4b: Writes updated state to Turn Database
5-7. **Escalation** (when needed): Problems escalate through DirectorAgent to HumanDirector
8. **Completion**: AgentProcess sends completion status
9. **Status Update**: Router forwards status to UI
10. **Result Display**: UserInterface presents final result to HumanDirector

### Key Component Roles

- **HumanDirector**: System entry/exit point for human oversight
- **UserInterface**: Request processing and result presentation layer
- **UnifiedMessageRouter** (Central Hub): Core routing and orchestration engine
- **AgentProcess**: Task execution engines (e.g., Frontend, Backend agents)
- **PersistenceLayer**: State management and turn history
- **DirectorAgent**: Escalation handler for complex decisions

### Escalation Protocol
Dotted lines indicate the escalation path activated when:
- AgentProcess encounters unresolvable conflicts
- Ambiguous requirements need clarification
- Critical decisions require human judgment

The escalation flow ensures system resilience while maintaining human oversight for critical decisions.