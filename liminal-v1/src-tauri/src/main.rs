#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;
mod router;
mod territory;

use router::UnifiedMessageRouter;
use territory::TerritoryManager;

#[tauri::command]
fn start_scenario() {
    println!("Scenario started!");
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