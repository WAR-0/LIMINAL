#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;
mod router;
mod territory;

use router::{Message, Priority, UnifiedMessageRouter};
use tauri::Emitter;
use territory::TerritoryManager;

#[tauri::command]
async fn start_scenario(
    router: tauri::State<'_, UnifiedMessageRouter>,
    territory_manager: tauri::State<'_, TerritoryManager>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    println!("V1 Hardcoded Scenario Started!");

    let agent_a_id = "Agent_A".to_string();
    let agent_b_id = "Agent_B".to_string();
    let resource = "shared_file.txt".to_string();

    // --- Agent A's Turn ---
    // 1. Acquire lease
    let acquired = territory_manager.acquire_lease(&agent_a_id, &resource);
    app_handle
        .emit(
            "agent_status",
            format!(
                "Agent A: acquiring lease on {}. Success: {}",
                resource, acquired
            ),
        )
        .unwrap();

    if acquired {
        // 2. Send message
        let msg = Message {
            content: "Hello from Agent A! I have the lease.".to_string(),
            priority: Priority::Coordinate,
            sender: agent_a_id.clone(),
            recipient: agent_b_id.clone(),
        };

        // Route the message
        router.route_message(msg.clone()).await;

        app_handle
            .emit(
                "message_log",
                format!("[{}->{}]: {}", msg.sender, msg.recipient, msg.content),
            )
            .unwrap();

        // 3. Release lease
        territory_manager.release_lease(&agent_a_id, &resource);
        app_handle
            .emit(
                "agent_status",
                format!("Agent A: released lease on {}.", resource),
            )
            .unwrap();
    }

    // --- Agent B's Turn ---
    // 1. Acquire lease
    let acquired_b = territory_manager.acquire_lease(&agent_b_id, &resource);
    app_handle
        .emit(
            "agent_status",
            format!(
                "Agent B: acquiring lease on {}. Success: {}",
                resource, acquired_b
            ),
        )
        .unwrap();

    if acquired_b {
        // 2. Send message
        let msg = Message {
            content: "Hello from Agent B! I have acquired the lease now.".to_string(),
            priority: Priority::Coordinate,
            sender: agent_b_id.clone(),
            recipient: agent_a_id.clone(),
        };

        // Route the message
        router.route_message(msg.clone()).await;

        app_handle
            .emit(
                "message_log",
                format!("[{}->{}]: {}", msg.sender, msg.recipient, msg.content),
            )
            .unwrap();

        // 3. Release lease
        territory_manager.release_lease(&agent_b_id, &resource);
        app_handle
            .emit(
                "agent_status",
                format!("Agent B: released lease on {}.", resource),
            )
            .unwrap();
    }

    app_handle
        .emit("scenario_complete", "Scenario Finished.")
        .unwrap();

    Ok(())
}

#[tauri::command]
async fn get_agent_status(agent_id: String) -> String {
    format!("Agent {} is idle.", agent_id)
}

fn main() {
    let router = UnifiedMessageRouter::new();
    let territory_manager = TerritoryManager::new();

    tauri::Builder::default()
        .manage(router)
        .manage(territory_manager)
        .invoke_handler(tauri::generate_handler![start_scenario, get_agent_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}