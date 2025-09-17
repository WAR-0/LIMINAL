#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;
mod metrics;
mod router;
mod territory;

use agent::AgentProcess;
use metrics::{MetricsCollector, PerformanceMetrics};
use router::{Message, Priority, UnifiedMessageRouter};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::Emitter;
use territory::TerritoryManager;

#[tauri::command]
async fn start_scenario(
    router: tauri::State<'_, UnifiedMessageRouter>,
    territory_manager: tauri::State<'_, TerritoryManager>,
    metrics: tauri::State<'_, MetricsCollector>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    println!("V1 Hardcoded Scenario Started!");

    let agent_a_id = "Agent_A".to_string();
    let agent_b_id = "Agent_B".to_string();
    let resource = "shared_file.txt".to_string();

    // --- Agent A's Turn ---
    // 1. Acquire lease
    let lease_start = Instant::now();
    let acquired = territory_manager.acquire_lease(&agent_a_id, &resource);
    metrics.record_lease_acquisition(lease_start.elapsed().as_millis() as f64);
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
        let route_start = Instant::now();
        router.route_message(msg.clone()).await;
        metrics.record_message_routing(route_start.elapsed().as_millis() as f64);

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
    let lease_start_b = Instant::now();
    let acquired_b = territory_manager.acquire_lease(&agent_b_id, &resource);
    metrics.record_lease_acquisition(lease_start_b.elapsed().as_millis() as f64);
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
        let route_start = Instant::now();
        router.route_message(msg.clone()).await;
        metrics.record_message_routing(route_start.elapsed().as_millis() as f64);

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

#[tauri::command]
async fn start_pty_scenario(
    router: tauri::State<'_, UnifiedMessageRouter>,
    territory_manager: tauri::State<'_, TerritoryManager>,
    agents: tauri::State<'_, Arc<Mutex<HashMap<String, AgentProcess>>>>,
    metrics: tauri::State<'_, MetricsCollector>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    println!("PTY Scenario Started with Real Processes!");

    let agent_a_id = "Agent_A".to_string();
    let agent_b_id = "Agent_B".to_string();
    let resource = "shared_file.txt".to_string();

    let spawn_start = Instant::now();
    let agent_a = AgentProcess::spawn(&agent_a_id, vec!["sh", "-c", "echo 'Agent A started'"]);
    metrics.record_agent_spawn(spawn_start.elapsed().as_millis() as f64);

    let spawn_start_b = Instant::now();
    let agent_b = AgentProcess::spawn(&agent_b_id, vec!["sh", "-c", "echo 'Agent B started'"]);
    metrics.record_agent_spawn(spawn_start_b.elapsed().as_millis() as f64);

    {
        let mut agents_map = agents.lock().unwrap();
        agents_map.insert(agent_a_id.clone(), agent_a);
        agents_map.insert(agent_b_id.clone(), agent_b);
    }

    app_handle
        .emit(
            "agent_status",
            "Spawned Agent A and Agent B as PTY processes",
        )
        .unwrap();

    let acquired = territory_manager.acquire_lease(&agent_a_id, &resource);
    app_handle
        .emit(
            "agent_status",
            format!(
                "Agent A (PTY): acquiring lease on {}. Success: {}",
                resource, acquired
            ),
        )
        .unwrap();

    if acquired {
        {
            let agents_map = agents.lock().unwrap();
            if let Some(agent) = agents_map.get(&agent_a_id) {
                agent.send_command("echo 'I have the lease!'").ok();
            }
        }

        let msg = Message {
            content: "Hello from PTY Agent A! I have the lease.".to_string(),
            priority: Priority::Coordinate,
            sender: agent_a_id.clone(),
            recipient: agent_b_id.clone(),
        };

        router.route_message(msg.clone()).await;

        app_handle
            .emit(
                "message_log",
                format!("[PTY {}->{}]: {}", msg.sender, msg.recipient, msg.content),
            )
            .unwrap();

        territory_manager.release_lease(&agent_a_id, &resource);
        app_handle
            .emit(
                "agent_status",
                format!("Agent A (PTY): released lease on {}.", resource),
            )
            .unwrap();
    }

    let acquired_b = territory_manager.acquire_lease(&agent_b_id, &resource);
    app_handle
        .emit(
            "agent_status",
            format!(
                "Agent B (PTY): acquiring lease on {}. Success: {}",
                resource, acquired_b
            ),
        )
        .unwrap();

    if acquired_b {
        {
            let agents_map = agents.lock().unwrap();
            if let Some(agent) = agents_map.get(&agent_b_id) {
                agent.send_command("echo 'Now I have the lease!'").ok();
            }
        }

        let msg = Message {
            content: "Hello from PTY Agent B! I have acquired the lease now.".to_string(),
            priority: Priority::Coordinate,
            sender: agent_b_id.clone(),
            recipient: agent_a_id.clone(),
        };

        router.route_message(msg.clone()).await;

        app_handle
            .emit(
                "message_log",
                format!("[PTY {}->{}]: {}", msg.sender, msg.recipient, msg.content),
            )
            .unwrap();

        territory_manager.release_lease(&agent_b_id, &resource);
        app_handle
            .emit(
                "agent_status",
                format!("Agent B (PTY): released lease on {}.", resource),
            )
            .unwrap();
    }

    app_handle
        .emit("scenario_complete", "PTY Scenario Finished.")
        .unwrap();

    Ok(())
}

#[tauri::command]
async fn get_performance_metrics(
    metrics: tauri::State<'_, MetricsCollector>,
) -> Result<PerformanceMetrics, String> {
    Ok(metrics.get_metrics())
}

#[tauri::command]
async fn reset_metrics(metrics: tauri::State<'_, MetricsCollector>) -> Result<(), String> {
    metrics.reset_metrics();
    Ok(())
}

fn main() {
    let router = UnifiedMessageRouter::new();
    let territory_manager = TerritoryManager::new();
    let agents: Arc<Mutex<HashMap<String, AgentProcess>>> = Arc::new(Mutex::new(HashMap::new()));
    let metrics_collector = MetricsCollector::new();

    tauri::Builder::default()
        .manage(router)
        .manage(territory_manager)
        .manage(agents)
        .manage(metrics_collector)
        .invoke_handler(tauri::generate_handler![
            start_scenario,
            start_pty_scenario,
            get_agent_status,
            get_performance_metrics,
            reset_metrics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
