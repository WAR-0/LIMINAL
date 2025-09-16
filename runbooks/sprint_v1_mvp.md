# LIMINAL V1 Sprint Runbook: Minimum Viable Prototype

**Role:** Lead Project Manager and Technical Architect
**Mission:** This document provides a step-by-step guide for the developer to build the V1 MVP of the LIMINAL system. It is a focused effort to build a functional prototype by completing a series of well-defined phases.

## V1 Sprint Scope: A Ruthless Definition of "Done"

This runbook is strictly focused on delivering the MVP. The goal is to build the simplest possible version that proves the core concept works.

### In Scope (The "Done" Definition)
-   **A Working Unified Message Router (Rust Core):** Implements a basic, functional 4-level priority queue and can manage at least two sandboxed agent processes via PTY.
-   **A Basic "Territory Leasing" System:** A simple `HashMap` in Rust protected by `Arc<RwLock<T>>` that supports `acquire_lease` and `release_lease`. No complex negotiation logic.
-   **A Barebones Visual Interface (Tauri + React/Svelte):** Displays the status of two agents, has a "Start Scenario" button, and shows a console-like message output. No GPU physics visualization.
-   **One (1) Hardcoded End-to-End Scenario:** An automated test case that demonstrates the core lease-message-release workflow between two agents.

### Explicitly OUT OF SCOPE
-   The advanced clone-based discussion system.
-   The full lease negotiation and escalation logic.
-   GPU physics visualization from the UNCAN research.
-   Token buckets and priority inflation prevention.
-   A UI for creating custom workflows.

---

## Phase 1: Core Foundation & Rust Backend

**Objective:** Establish the Rust backend, including the message router and basic agent management.

### Task 1.1: Project Setup

**Action:** Set up the Tauri project with a Rust backend and a minimal React or Svelte frontend.

Follow the instructions in the official Tauri documentation and our internal setup guide.

-   **Reference:** `docs-canonical/how-to/01_setup_guide.md`

### Task 1.2: Implement the Message Router Stub

**Action:** Create the basic structure for the `UnifiedMessageRouter`. For V1, this will be a simplified version holding only the message queues.

-   **Reference:** `docs-canonical/conceptual/01_system_architecture.md`
-   **Reference:** `docs-canonical/reference/02_interaction_model.md`

Create a `router.rs` file and add the following code:

```rust
// src-tauri/src/router.rs

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

// Simplified from the canonical spec for V1
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Info = 0,       // Telemetry, logs
    Coordinate = 1, // Tasking, routine operations
    Blocking = 2,   // Sender blocked
    Critical = 3,   // System failures, alerts
}

pub struct Message {
    pub content: String,
    pub priority: Priority,
    pub sender: String,
    pub recipient: String,
}

// V1 Stub: Just the queues. No token buckets, pause detection, etc.
pub struct UnifiedMessageRouter {
    // 4 priority levels: Info, Coordinate, Blocking, Critical
    pub message_queues: [Arc<RwLock<VecDeque<Message>>>; 4],
}

impl UnifiedMessageRouter {
    pub fn new() -> Self {
        Self {
            message_queues: [
                Arc::new(RwLock::new(VecDeque::new())),
                Arc::new(RwLock::new(VecDeque::new())),
                Arc::new(RwLock::new(VecDeque::new())),
                Arc::new(RwLock::new(VecDeque::new())),
            ],
        }
    }

    pub async fn route_message(&self, msg: Message) {
        let queue_idx = msg.priority as usize;
        let mut queue = self.message_queues[queue_idx].write().await;
        queue.push_back(msg);
    }
}
```

### Task 1.3: Implement PTY Agent Management

**Action:** Implement the logic to spawn and manage two child processes (our "agents") using pseudo-terminals (PTY). This isolates them and allows us to capture their I/O. We will use the `portable-pty` crate.

Add `portable-pty` to your `Cargo.toml`:
```toml
[dependencies]
portable-pty = "0.1.0"
```

Create an `agent.rs` file:
```rust
// src-tauri/src/agent.rs

use portable_pty::{CommandBuilder, NativePtySystem, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct AgentProcess {
    pub id: String,
    writer: Arc<Mutex<Box<dyn Write + Send>>>, 
}

impl AgentProcess {
    pub fn spawn(id: &str, command: Vec<&str>) -> Self {
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }).unwrap();

        let mut cmd = CommandBuilder::new(command[0]);
        cmd.args(&command[1..]);

        let mut child = pair.slave.spawn_command(cmd).unwrap();
        let reader = pair.master.try_clone_reader().unwrap();
        let writer = Arc::new(Mutex::new(pair.master));

        let agent_id = id.to_string();
        thread::spawn(move || {
            let mut reader = reader;
            let mut buffer = [0u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(len) => {
                        let output = String::from_utf8_lossy(&buffer[..len]);
                        println!("[Agent {}]: {}", agent_id, output);
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            id: id.to_string(),
            writer,
        }
    }
}
```

### Task 1.4: Basic Tauri Commands

**Action:** Define the initial Tauri commands in `main.rs` to allow the frontend to interact with the backend.

```rust
// src-tauri/src/main.rs

// ... other imports
mod agent;
mod router;
mod territory;

// Tauri commands
#[tauri::command]
fn start_scenario() {
    println!("Scenario started!");
    // This will be filled out in Phase 2
}

#[tauri::command]
async fn get_agent_status(agent_id: String) -> String {
    // Placeholder logic
    format!("Agent {} is idle.", agent_id)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_scenario, get_agent_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Phase 1 Deliverable
A running Tauri application with a Rust backend that can spawn two child processes (e.g., simple scripts) and has placeholders for the message router and Tauri commands. The UI will be minimal, perhaps just showing that the application has started.

---

## Phase 2: State Management & Core Logic

**Objective:** Implement the state management for leases and the logic for the hardcoded scenario.

### Task 2.1: Implement the Territory Manager Stub

**Action:** Create a simple `TerritoryManager` to handle resource leases. For V1, this will use a `HashMap` to track leases and will not have any complex negotiation logic.

-   **Reference:** `docs-canonical/reference/01_agent_capabilities.md` (Simplified Logic)

Create a `territory.rs` file:
```rust
// src-tauri/src/territory.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type ResourcePath = String;
pub type AgentId = String;

#[derive(Default)]
pub struct TerritoryManager {
    // Simple V1 implementation: Resource -> Agent holding the lease
    leases: Arc<RwLock<HashMap<ResourcePath, AgentId>>>, 
}

impl TerritoryManager {
    pub fn new() -> Self {
        Default::default()
    }

    // Tries to acquire a lease. Fails if already taken.
    pub fn acquire_lease(&self, agent_id: &AgentId, resource: &ResourcePath) -> bool {
        let mut leases = self.leases.write().unwrap();
        if leases.contains_key(resource) {
            return false; // Lease already taken
        }
        leases.insert(resource.clone(), agent_id.clone());
        true
    }

    // Releases a lease.
    pub fn release_lease(&self, agent_id: &AgentId, resource: &ResourcePath) {
        let mut leases = self.leases.write().unwrap();
        if let Some(holder) = leases.get(resource) {
            if holder == agent_id {
                leases.remove(resource);
            }
        }
    }
}
```

### Task 2.2: Implement the Hardcoded Scenario Logic

**Action:** In your `main.rs`, implement the logic for the hardcoded end-to-end scenario within the `start_scenario` command. This involves two agents interacting with a shared resource.

```rust
// src-tauri/src/main.rs

// ... (update the start_scenario command)

#[tauri::command]
fn start_scenario(
    router: tauri::State<router::UnifiedMessageRouter>,
    territory_manager: tauri::State<territory::TerritoryManager>,
    app_handle: tauri::AppHandle,
) {
    println!("V1 Hardcoded Scenario Started!");

    let agent_a_id = "Agent_A".to_string();
    let agent_b_id = "Agent_B".to_string();
    let resource = "shared_file.txt".to_string();

    // --- Agent A's Turn ---
    // 1. Acquire lease
    let acquired = territory_manager.acquire_lease(&agent_a_id, &resource);
    app_handle.emit_all("agent_status", format!("Agent A: acquiring lease on {}. Success: {}", resource, acquired)).unwrap();

    if acquired {
        // 2. Send message
        let msg = router::Message {
            content: "Hello from Agent A! I have the lease.".to_string(),
            priority: router::Priority::Coordinate,
            sender: agent_a_id.clone(),
            recipient: agent_b_id.clone(),
        };
        app_handle.emit_all("message_log", format!("[{{}}]: {{}}", msg.sender, msg.recipient, msg.content)).unwrap();
        
        // 3. Release lease
        territory_manager.release_lease(&agent_a_id, &resource);
        app_handle.emit_all("agent_status", format!("Agent A: released lease on {}.", resource)).unwrap();
    }

    // --- Agent B's Turn ---
    // 1. Acquire lease
    let acquired_b = territory_manager.acquire_lease(&agent_b_id, &resource);
    app_handle.emit_all("agent_status", format!("Agent B: acquiring lease on {}. Success: {}", resource, acquired_b)).unwrap();

    if acquired_b {
        // 2. Send message
        let msg = router::Message {
            content: "Hello from Agent B! I have acquired the lease now.".to_string(),
            priority: router::Priority::Coordinate,
            sender: agent_b_id.clone(),
            recipient: agent_a_id.clone(),
        };
        app_handle.emit_all("message_log", format!("[{{}}]: {{}}", msg.sender, msg.recipient, msg.content)).unwrap();

        // 3. Release lease
        territory_manager.release_lease(&agent_b_id, &resource);
        app_handle.emit_all("agent_status", format!("Agent B: released lease on {}.", resource)).unwrap();
    }
    
    app_handle.emit_all("agent_status", "Scenario Finished.".to_string()).unwrap();
}
```

### Task 2.3: Integrate State with Tauri

**Action:** Use Tauri's state management to make single instances of your `UnifiedMessageRouter` and `TerritoryManager` available to your commands.

```rust
// src-tauri/src/main.rs

// ... (update the main function)

fn main() {
    let router = router::UnifiedMessageRouter::new();
    let territory_manager = territory::TerritoryManager::new();

    tauri::Builder::default()
        .manage(router)
        .manage(territory_manager)
        .invoke_handler(tauri::generate_handler![start_scenario, get_agent_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Phase 2 Deliverable
The backend logic is complete. The hardcoded scenario can be triggered by calling the `start_scenario` command and runs successfully on the backend, printing logs to the console.

---

## Phase 3: Frontend Interface & Final Integration

**Objective:** Build the minimal UI and connect it to the backend to visualize the V1 scenario.

### Task 3.1: Build the UI Components

**Action:** Create the three required UI components in your chosen frontend framework (React/Svelte).

1.  **Agent Status Panel:** A simple `div` for each agent (`Agent A`, `Agent B`) that displays their current status string.
2.  **Start Scenario Button:** A single `<button>` that, when clicked, invokes the `start_scenario` Tauri command.
3.  **Message Log Panel:** A `div` or `textarea` that appends messages received from the backend, showing the flow of communication.

Example (React):
```jsx
// src/App.jsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

function App() {
  const [agentAStatus, setAgentAStatus] = useState('Idle');
  const [agentBStatus, setAgentBStatus] = useState('Idle');
  const [messages, setMessages] = useState([]);

  useEffect(() => {
    const unlistenStatus = listen('agent_status', (event) => {
      const payload = event.payload;
      if (payload.includes('Agent A:')) {
        setAgentAStatus(payload);
      } else if (payload.includes('Agent B:')) {
        setAgentBStatus(payload);
      } else {
         // General status
      }
    });

    const unlistenLog = listen('message_log', (event) => {
      setMessages(prev => [...prev, event.payload]);
    });

    return () => {
      unlistenStatus.then(f => f());
      unlistenLog.then(f => f());
    };
  }, []);

  return (
    <div>
      <h1>LIMINAL V1 MVP</h1>
      <button onClick={() => invoke('start_scenario')}>Start Scenario</button>
      
      <div>
        <h2>Agent Status</h2>
        <p><strong>Agent A:</strong> {agentAStatus}</p>
        <p><strong>Agent B:</strong> {agentBStatus}</p>
      </div>

      <div>
        <h2>Message Log</h2>
        <pre>{messages.join('\n')}</pre>
      </div>
    </div>
  );
}

export default App;
```

### Task 3.2: Implement Frontend State Management

**Action:** Use a simple state management solution to handle the agent statuses and message log. The example above uses React's built-in `useState`. For more complex applications, you might use Zustand (React) or Svelte Stores. The key is to have a central place to hold the data that comes from the backend.

### Task 3.3: Connect UI to Backend

**Action:** Use the Tauri API to invoke commands and listen for events from the Rust backend.

-   **Invoking Commands:** Use `invoke('command_name', { args })` to call your Rust functions.
-   **Listening for Events:** Use `listen('event_name', callback)` to subscribe to events emitted from Rust with `app_handle.emit_all()`.

The React code in **Task 3.1** demonstrates this pattern.

### Task 3.4: End-to-End Test

**Action:** Perform the final manual test to verify the entire workflow.

-   **Reference:** `docs-canonical/reference/03_testing_framework.md` (for success criteria)

**Test Steps:**
1.  Run the application (`npm run tauri dev`).
2.  The UI should load, showing both agents as "Idle".
3.  Click the "Start Scenario" button.
4.  **Verify:** The Agent Status panel updates in real-time, showing Agent A acquiring and releasing the lease, followed by Agent B.
5.  **Verify:** The Message Log panel displays the two messages exchanged between the agents in the correct order.
6.  **Verify:** The final status indicates the scenario is complete.

**Success Criteria:**
-   The UI correctly reflects the state changes occurring in the backend.
-   Messages are displayed in the correct sequence.
-   The application does not crash and completes the scenario cleanly.

### Phase 3 Deliverable
A fully functional, demonstrable V1 prototype of LIMINAL that executes the core lease-message-release workflow, with the results visualized in the frontend.
