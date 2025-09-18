#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;
mod config;
mod health;
mod metrics;
mod router;
mod territory;

use agent::{AgentEvent, AgentEventSender, AgentProcess};
use config::AppConfig;
use health::HealthMonitor;
use metrics::{MetricsCollector, MetricsSnapshot, PerformanceMetrics};
use router::{Message, Priority, UnifiedMessageRouter};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tauri::async_runtime::JoinHandle;
use tauri::Emitter;
use territory::{LeaseDecision, LeaseRequest, TerritoryManager};
use tokio::sync::{mpsc, Mutex as AsyncMutex};

type SharedHealthMonitor = Arc<AsyncMutex<HealthMonitor>>;

struct MetricsStreamState {
    handle: AsyncMutex<Option<JoinHandle<()>>>,
}

impl MetricsStreamState {
    fn new() -> Self {
        Self {
            handle: AsyncMutex::new(None),
        }
    }

    async fn ensure_running(
        &self,
        metrics: MetricsCollector,
        app_handle: tauri::AppHandle,
        health_monitor: SharedHealthMonitor,
    ) {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return;
        }
        let metrics_clone = metrics.clone();
        let emitter = app_handle.clone();
        let health_monitor_clone = health_monitor.clone();
        let handle = tauri::async_runtime::spawn(async move {
            loop {
                let snapshot = metrics_clone.get_snapshot();
                let alerts = {
                    let mut monitor = health_monitor_clone.lock().await;
                    monitor.evaluate(&snapshot)
                };
                for alert in alerts {
                    println!("[HealthAlert {}]: {}", alert.severity, alert.message);
                    if let Err(err) = emitter.emit("health_alert", alert) {
                        println!("[HealthAlert emit error]: {}", err);
                    }
                }
                if let Err(err) = emitter.emit("metrics_snapshot", snapshot.clone()) {
                    println!("[MetricsStream emit error]: {}", err);
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
        *guard = Some(handle);
    }
}

impl Default for MetricsStreamState {
    fn default() -> Self {
        Self::new()
    }
}

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
async fn get_metrics_snapshot(
    metrics: tauri::State<'_, MetricsCollector>,
) -> Result<MetricsSnapshot, String> {
    Ok(metrics.get_snapshot())
}

#[tauri::command]
async fn start_metrics_stream(
    metrics: tauri::State<'_, MetricsCollector>,
    stream_state: tauri::State<'_, MetricsStreamState>,
    health_monitor: tauri::State<'_, SharedHealthMonitor>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    stream_state
        .ensure_running(
            metrics.inner().clone(),
            app_handle,
            health_monitor.inner().clone(),
        )
        .await;
    Ok(())
}

#[tauri::command]
async fn simulate_router_load(
    router: tauri::State<'_, UnifiedMessageRouter>,
) -> Result<(), String> {
    let priorities = [
        Priority::Info,
        Priority::Coordinate,
        Priority::Blocking,
        Priority::Critical,
        Priority::DirectorOverride,
    ];
    for index in 0..40u32 {
        let priority = priorities[(index as usize) % priorities.len()];
        let message = Message {
            content: format!("Synthetic message {}", index),
            priority,
            sender: format!("synthetic_sender_{}", index % 5),
            recipient: format!("synthetic_recipient_{}", index % 3),
        };
        router
            .route_message(message)
            .await
            .map_err(|err| format!("failed to route synthetic message: {:?}", err))?;
    }
    Ok(())
}

#[tauri::command]
async fn simulate_lease_contention(
    territory_manager: tauri::State<'_, TerritoryManager>,
) -> Result<(), String> {
    let manager = territory_manager.inner().clone();
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let resource_id = format!("synthetic_resource_{}", timestamp);
    let mut workers = Vec::new();
    for index in 0..6u32 {
        let manager_clone = manager.clone();
        let resource = resource_id.clone();
        workers.push(tauri::async_runtime::spawn(async move {
            let request = LeaseRequest::new(
                format!("SyntheticAgent_{}", index),
                resource.clone(),
                if index % 2 == 0 {
                    Priority::Coordinate
                } else {
                    Priority::Blocking
                },
            );
            let _ = manager_clone.acquire_lease(request).await;
        }));
    }
    for worker in workers {
        let _ = worker.await;
    }
    for index in 0..6u32 {
        let manager_clone = manager.clone();
        let resource = resource_id.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_secs(3 + index as u64)).await;
            let agent_id = format!("SyntheticAgent_{}", index);
            let _ = manager_clone.release_lease(&agent_id, &resource).await;
        });
    }
    Ok(())
}

#[tauri::command]
async fn reset_metrics(metrics: tauri::State<'_, MetricsCollector>) -> Result<(), String> {
    metrics.reset_metrics();
    Ok(())
}

fn main() {
    let app_config = AppConfig::load();
    let metrics_collector = MetricsCollector::new();
    let router =
        UnifiedMessageRouter::with_settings(metrics_collector.clone(), app_config.router.as_ref());
    let territory_manager =
        TerritoryManager::new(metrics_collector.clone(), app_config.territory.as_ref());
    let agents: Arc<Mutex<HashMap<String, AgentProcess>>> = Arc::new(Mutex::new(HashMap::new()));
    let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let event_sender = AgentEventSender::new(event_tx);
    let mut event_rx = Some(event_rx);
    let metrics_for_setup = metrics_collector.clone();
    let health_monitor: SharedHealthMonitor = Arc::new(AsyncMutex::new(HealthMonitor::new(
        app_config.health_monitoring_kpis.as_ref(),
    )));
    let metrics_stream_state = MetricsStreamState::new();
    let app_config_state = app_config.clone();

    tauri::Builder::default()
        .manage(router)
        .manage(territory_manager)
        .manage(agents)
        .manage(metrics_collector)
        .manage(event_sender)
        .manage(metrics_stream_state)
        .manage(health_monitor.clone())
        .manage(app_config_state)
        .setup(move |_app| {
            let mut rx = event_rx.take().expect("agent event receiver missing");
            let metrics = metrics_for_setup.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    metrics.record_agent_event(&event.agent_id, event.event_name.as_deref());
                    let name = event
                        .event_name
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());
                    println!("[AgentEvent {} - {}]: {}", event.agent_id, name, event.raw);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_scenario,
            start_pty_scenario,
            get_agent_status,
            get_performance_metrics,
            get_metrics_snapshot,
            start_metrics_stream,
            simulate_router_load,
            simulate_lease_contention,
            reset_metrics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
