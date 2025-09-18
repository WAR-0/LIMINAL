#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;
mod metrics;
mod router;
mod territory;

use agent::{AgentEvent, AgentEventSender, AgentProcess};
use metrics::{MetricsCollector, PerformanceMetrics};
use router::{Message, Priority, UnifiedMessageRouter};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::Emitter;
use territory::{LeaseDecision, LeaseRequest, TerritoryManager};
use tokio::sync::mpsc;

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
    let decision = territory_manager
        .acquire_lease(LeaseRequest::new(
            agent_a_id.clone(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    let acquired = matches!(
        decision,
        LeaseDecision::Granted(_) | LeaseDecision::Overridden { .. }
    );
    if acquired {
        metrics.record_lease_acquisition(lease_start.elapsed().as_millis() as f64);
    }
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
        let _ = router.route_message(msg.clone()).await;
        metrics.record_message_routing(route_start.elapsed().as_millis() as f64);

        app_handle
            .emit(
                "message_log",
                format!("[{}->{}]: {}", msg.sender, msg.recipient, msg.content),
            )
            .unwrap();

        // 3. Release lease
        let _ = territory_manager
            .release_lease(&agent_a_id, &resource)
            .await;
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
    let decision_b = territory_manager
        .acquire_lease(LeaseRequest::new(
            agent_b_id.clone(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    let acquired_b = matches!(
        decision_b,
        LeaseDecision::Granted(_) | LeaseDecision::Overridden { .. }
    );
    if acquired_b {
        metrics.record_lease_acquisition(lease_start_b.elapsed().as_millis() as f64);
    }
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
        let _ = router.route_message(msg.clone()).await;
        metrics.record_message_routing(route_start.elapsed().as_millis() as f64);

        app_handle
            .emit(
                "message_log",
                format!("[{}->{}]: {}", msg.sender, msg.recipient, msg.content),
            )
            .unwrap();

        // 3. Release lease
        let _ = territory_manager
            .release_lease(&agent_b_id, &resource)
            .await;
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
    event_sender: tauri::State<'_, AgentEventSender>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    println!("PTY Scenario Started with Real Processes!");

    let agent_a_id = "Agent_A".to_string();
    let agent_b_id = "Agent_B".to_string();
    let resource = "shared_file.txt".to_string();
    let pipe = event_sender.sender();

    let spawn_start = Instant::now();
    let agent_a = AgentProcess::spawn(
        &agent_a_id,
        vec!["sh", "-c", "echo 'Agent A started'"],
        pipe.clone(),
    );
    metrics.record_agent_spawn(spawn_start.elapsed().as_millis() as f64);

    let spawn_start_b = Instant::now();
    let agent_b = AgentProcess::spawn(
        &agent_b_id,
        vec!["sh", "-c", "echo 'Agent B started'"],
        pipe.clone(),
    );
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

    let decision = territory_manager
        .acquire_lease(LeaseRequest::new(
            agent_a_id.clone(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    let acquired = matches!(
        decision,
        LeaseDecision::Granted(_) | LeaseDecision::Overridden { .. }
    );
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

        let _ = router.route_message(msg.clone()).await;

        app_handle
            .emit(
                "message_log",
                format!("[PTY {}->{}]: {}", msg.sender, msg.recipient, msg.content),
            )
            .unwrap();

        let _ = territory_manager
            .release_lease(&agent_a_id, &resource)
            .await;
        app_handle
            .emit(
                "agent_status",
                format!("Agent A (PTY): released lease on {}.", resource),
            )
            .unwrap();
    }

    let decision_b = territory_manager
        .acquire_lease(LeaseRequest::new(
            agent_b_id.clone(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    let acquired_b = matches!(
        decision_b,
        LeaseDecision::Granted(_) | LeaseDecision::Overridden { .. }
    );
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

        let _ = router.route_message(msg.clone()).await;

        app_handle
            .emit(
                "message_log",
                format!("[PTY {}->{}]: {}", msg.sender, msg.recipient, msg.content),
            )
            .unwrap();

        let _ = territory_manager
            .release_lease(&agent_b_id, &resource)
            .await;
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
    let metrics_collector = MetricsCollector::new();
    let router = UnifiedMessageRouter::with_metrics(metrics_collector.clone());
    let territory_manager = TerritoryManager::new(metrics_collector.clone());
    let agents: Arc<Mutex<HashMap<String, AgentProcess>>> = Arc::new(Mutex::new(HashMap::new()));
    let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let event_sender = AgentEventSender::new(event_tx);
    let mut event_rx = Some(event_rx);
    let metrics_for_setup = metrics_collector.clone();

    tauri::Builder::default()
        .manage(router)
        .manage(territory_manager)
        .manage(agents)
        .manage(metrics_collector)
        .manage(event_sender)
        .setup(move |_app| {
            let mut rx = event_rx.take().expect("agent event receiver missing");
            let metrics = metrics_for_setup.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    let name = event
                        .event_name
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());
                    metrics.record_agent_event(&name);
                    println!("[AgentEvent {}]: {}", event.agent_id, event.raw);
                }
            });
            Ok(())
        })
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
