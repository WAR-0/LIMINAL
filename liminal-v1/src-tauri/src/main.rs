#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;
mod config;
mod director;
mod executor;
mod metrics;
mod router;
mod territory;

#[allow(dead_code)]
mod consensus;

#[allow(dead_code)]
mod health;

#[allow(dead_code)]
mod ledger;

use agent::{AgentEvent, AgentEventSender, AgentProcess};
use config::{AppConfig, LedgerConfig};
use consensus::ConsensusBroker;
use director::{DirectorAgent, RunbookSummary, TurnUpdate};
use executor::MaintenanceExecutor;
use metrics::{MetricsCollector, MetricsSnapshot, PerformanceMetrics};

#[allow(unused_imports)]
use health::HealthMonitor;

#[allow(unused_imports)]
use ledger::{
    EventEnvelope, HealthEvent, LeaseReplayState, LedgerEvent, LedgerReader, LedgerWriter,
    PtyEvent, ReplayCoordinator, ReplayOutcome, RouterReplayState, StateCheckpoint,
};
use router::{Message, Priority, UnifiedMessageRouter};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::async_runtime::JoinHandle;
use tauri::Emitter;
use territory::{LeaseDecision, LeaseRequest, TerritoryManager};
use tokio::sync::{mpsc, Mutex as AsyncMutex};

type SharedHealthMonitor = Arc<AsyncMutex<HealthMonitor>>;

struct MetricsStreamState {
    handle: AsyncMutex<Option<JoinHandle<()>>>,
    last_checkpoint: Arc<AsyncMutex<Option<Instant>>>,
}

fn submit_checkpoint_task(
    maintenance: &MaintenanceExecutor,
    ledger: LedgerWriter,
    metrics: MetricsCollector,
    checkpoint_id: String,
    captured_at_ms: u64,
    router_state: RouterReplayState,
    lease_state: LeaseReplayState,
    snapshot: MetricsSnapshot,
) {
    let maintenance = maintenance.clone();
    maintenance.spawn(async move {
        let checkpoint = StateCheckpoint {
            checkpoint_id,
            captured_at_ms,
            router: router_state,
            leases: lease_state,
            metrics: snapshot,
        };
        let start = Instant::now();
        if ledger.record_checkpoint(checkpoint).await.is_ok() {
            metrics.record_ledger_append(start.elapsed());
        } else {
            metrics.record_ledger_error();
        }
    });
}

impl MetricsStreamState {
    fn new() -> Self {
        Self {
            handle: AsyncMutex::new(None),
            last_checkpoint: Arc::new(AsyncMutex::new(None)),
        }
    }

    async fn ensure_running(
        &self,
        metrics: MetricsCollector,
        app_handle: tauri::AppHandle,
        health_monitor: SharedHealthMonitor,
        ledger: LedgerWriter,
        checkpoint_interval: Duration,
        maintenance: MaintenanceExecutor,
    ) {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return;
        }
        let metrics_clone = metrics.clone();
        let emitter = app_handle.clone();
        let health_monitor_clone = health_monitor.clone();
        let ledger_clone = ledger.clone();
        let checkpoint_interval = checkpoint_interval;
        let checkpoint_tracker = self.last_checkpoint.clone();
        let maintenance_clone = maintenance.clone();
        let handle = tauri::async_runtime::spawn(async move {
            loop {
                let snapshot = metrics_clone.get_snapshot();
                let alerts = {
                    let mut monitor = health_monitor_clone.lock().await;
                    monitor.evaluate(&snapshot)
                };
                for alert in alerts {
                    println!("[HealthAlert {}]: {}", alert.severity, alert.message);
                    if let Err(err) = emitter.emit("health_alert", alert.clone()) {
                        println!("[HealthAlert emit error]: {}", err);
                    }
                    let health_event = LedgerEvent::Health(HealthEvent {
                        severity: alert.severity.clone(),
                        message: alert.message.clone(),
                        timestamp_ms: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64,
                    });
                    let start = Instant::now();
                    if ledger_clone
                        .clone()
                        .append_async(health_event)
                        .await
                        .is_ok()
                    {
                        metrics_clone.record_ledger_append(start.elapsed());
                    } else {
                        metrics_clone.record_ledger_error();
                    }
                }
                if let Err(err) = emitter.emit("metrics_snapshot", snapshot.clone()) {
                    println!("[MetricsStream emit error]: {}", err);
                }
                let mut last_checkpoint = checkpoint_tracker.lock().await;
                let should_checkpoint = last_checkpoint
                    .map(|previous| previous.elapsed() >= checkpoint_interval)
                    .unwrap_or(true);
                if should_checkpoint {
                    let router_state = RouterReplayState {
                        total_dispatched: snapshot.performance.total_messages_routed,
                        last_priority: snapshot.router.last_dispatched_priority.clone(),
                        queue_depths: priority_vec_from_map(&snapshot.router.queue_depths),
                    };
                    let mut lease_state = LeaseReplayState::default();
                    lease_state.deferrals = snapshot.leases.deferrals;
                    lease_state.overrides = snapshot.leases.overrides;
                    lease_state.escalations = snapshot.leases.escalations;
                    let checkpoint_metrics = snapshot.clone();
                    let checkpoint_id = format!(
                        "checkpoint-{}",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis()
                    );
                    let captured_at_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    submit_checkpoint_task(
                        &maintenance_clone,
                        ledger_clone.clone(),
                        metrics_clone.clone(),
                        checkpoint_id,
                        captured_at_ms,
                        router_state,
                        lease_state,
                        checkpoint_metrics,
                    );
                    *last_checkpoint = Some(Instant::now());
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
    ledger: tauri::State<'_, LedgerWriter>,
    app_config: tauri::State<'_, AppConfig>,
    maintenance: tauri::State<'_, MaintenanceExecutor>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let checkpoint_secs = app_config
        .ledger
        .as_ref()
        .map(|cfg| cfg.checkpoint_interval_secs)
        .unwrap_or_else(|| LedgerConfig::default().checkpoint_interval_secs);
    let checkpoint_interval = Duration::from_secs(checkpoint_secs.max(1));
    stream_state
        .ensure_running(
            metrics.inner().clone(),
            app_handle,
            health_monitor.inner().clone(),
            ledger.inner().clone(),
            checkpoint_interval,
            maintenance.inner().clone(),
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

fn priority_vec_from_map(depths: &BTreeMap<String, usize>) -> Vec<usize> {
    let mut values = Vec::new();
    for index in 0..=Priority::DirectorOverride.as_index() {
        let key = Priority::from_index(index).as_str();
        values.push(*depths.get(key).unwrap_or(&0));
    }
    values
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

#[tauri::command]
async fn ledger_replay(
    ledger_reader: tauri::State<'_, LedgerReader>,
    ledger_writer: tauri::State<'_, LedgerWriter>,
    metrics: tauri::State<'_, MetricsCollector>,
    epoch_id: Option<String>,
) -> Result<ReplayOutcome, String> {
    let epoch = epoch_id.unwrap_or_else(|| ledger_writer.epoch_id());
    match ledger_reader.verify_epoch(&epoch) {
        Ok(true) => {}
        Ok(false) => {
            metrics.record_ledger_integrity_failure();
        }
        Err(err) => {
            metrics.record_ledger_integrity_failure();
            return Err(err.to_string());
        }
    }
    let coordinator = ReplayCoordinator::new(ledger_reader.inner().clone());
    coordinator
        .replay_epoch(&epoch)
        .map_err(|err| err.to_string())
}

#[derive(serde::Serialize)]
struct LedgerStatus {
    epoch_id: String,
    event_count: usize,
    verified: bool,
}

#[tauri::command]
async fn ledger_status(
    ledger_reader: tauri::State<'_, LedgerReader>,
    ledger_writer: tauri::State<'_, LedgerWriter>,
) -> Result<LedgerStatus, String> {
    let epoch = ledger_writer.epoch_id();
    let events = ledger_reader
        .inner()
        .read_epoch(&epoch)
        .map_err(|err| err.to_string())?;
    let verified = ledger_reader
        .verify_epoch(&epoch)
        .map_err(|err| err.to_string())?;
    Ok(LedgerStatus {
        epoch_id: epoch,
        event_count: events.len(),
        verified,
    })
}

#[tauri::command]
async fn ledger_tail(
    ledger_reader: tauri::State<'_, LedgerReader>,
    ledger_writer: tauri::State<'_, LedgerWriter>,
    limit: Option<usize>,
    epoch_id: Option<String>,
) -> Result<Vec<EventEnvelope>, String> {
    let epoch = epoch_id.unwrap_or_else(|| ledger_writer.epoch_id());
    let mut events = ledger_reader
        .inner()
        .read_epoch(&epoch)
        .map_err(|err| err.to_string())?;
    if let Some(limit) = limit {
        if events.len() > limit {
            events = events.split_off(events.len() - limit);
        }
    }
    Ok(events)
}

#[tauri::command]
async fn director_load_runbook(
    director: tauri::State<'_, Arc<DirectorAgent>>,
    path: String,
) -> Result<RunbookSummary, String> {
    director
        .load_runbook(std::path::Path::new(&path))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn director_start_runbook(
    director: tauri::State<'_, Arc<DirectorAgent>>,
) -> Result<(), String> {
    director.start_execution().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn director_get_turn_status(
    director: tauri::State<'_, Arc<DirectorAgent>>,
) -> Result<Vec<TurnUpdate>, String> {
    Ok(director.get_turn_status())
}

#[tauri::command]
async fn director_get_summary(
    director: tauri::State<'_, Arc<DirectorAgent>>,
) -> Result<Option<RunbookSummary>, String> {
    Ok(director.get_summary())
}

#[tauri::command]
async fn director_pause_execution(
    director: tauri::State<'_, Arc<DirectorAgent>>,
) -> Result<(), String> {
    director.pause_execution().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn director_resume_execution(
    director: tauri::State<'_, Arc<DirectorAgent>>,
) -> Result<(), String> {
    director.resume_execution().await.map_err(|e| e.to_string())
}

fn main() {
    let app_config = AppConfig::load();
    let ledger_config = app_config.ledger.clone().unwrap_or_default();
    let ledger_writer =
        LedgerWriter::new(&ledger_config).expect("failed to initialize ledger writer");
    let ledger_reader = LedgerReader::new(ledger_config.root_path.clone());
    let metrics_collector = MetricsCollector::new();
    let available_workers = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(4);
    let maintenance_workers = available_workers.clamp(2, 8);
    let maintenance_executor = MaintenanceExecutor::new(maintenance_workers);
    let consensus_broker =
        ConsensusBroker::new(Some(ledger_writer.clone()), metrics_collector.clone(), 0.66);
    let router = UnifiedMessageRouter::with_settings_ledger_and_consensus(
        metrics_collector.clone(),
        app_config.router.as_ref(),
        Some(ledger_writer.clone()),
        Some(consensus_broker.clone()),
    );
    let territory_manager = TerritoryManager::new_with_ledger(
        metrics_collector.clone(),
        app_config.territory.as_ref(),
        Some(ledger_writer.clone()),
    );
    tauri::async_runtime::block_on(router.set_maintenance_executor(maintenance_executor.clone()));
    tauri::async_runtime::block_on(
        territory_manager.set_maintenance_executor(maintenance_executor.clone()),
    );
    let working_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let director_router = UnifiedMessageRouter::with_metrics(metrics_collector.clone());
    let director_agent = Arc::new(DirectorAgent::new(
        working_dir,
        metrics_collector.clone(),
        director_router,
    ));
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
    let ledger_for_setup = ledger_writer.clone();

    tauri::Builder::default()
        .manage(router)
        .manage(territory_manager)
        .manage(director_agent)
        .manage(agents)
        .manage(metrics_collector)
        .manage(event_sender)
        .manage(metrics_stream_state)
        .manage(health_monitor.clone())
        .manage(app_config_state)
        .manage(ledger_writer.clone())
        .manage(ledger_reader.clone())
        .manage(maintenance_executor.clone())
        .setup(move |_app| {
            let mut rx = event_rx.take().expect("agent event receiver missing");
            let metrics = metrics_for_setup.clone();
            let ledger = ledger_for_setup.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    metrics.record_agent_event(&event.agent_id, event.event_name.as_deref());
                    let name = event
                        .event_name
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());
                    println!("[AgentEvent {} - {}]: {}", event.agent_id, name, event.raw);
                    let pty_event = LedgerEvent::Pty(PtyEvent {
                        agent_id: event.agent_id.clone(),
                        event_name: event.event_name.clone(),
                        timestamp_ms: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64,
                    });
                    let start = Instant::now();
                    if ledger.clone().append_async(pty_event).await.is_ok() {
                        metrics.record_ledger_append(start.elapsed());
                    } else {
                        metrics.record_ledger_error();
                    }
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
            reset_metrics,
            ledger_replay,
            ledger_status,
            ledger_tail,
            director_load_runbook,
            director_start_runbook,
            director_get_turn_status,
            director_get_summary,
            director_pause_execution,
            director_resume_execution
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::submit_checkpoint_task;
    use crate::config::LedgerConfig;
    use crate::executor::MaintenanceExecutor;
    use crate::ledger::{
        LeaseReplayState, LedgerEvent, LedgerReader, LedgerWriter, RouterReplayState,
    };
    use crate::metrics::MetricsCollector;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tempfile::tempdir;

    #[tokio::test]
    async fn checkpoint_submission_runs_on_executor() {
        let executor = MaintenanceExecutor::new(2);
        let temp_dir = tempdir().expect("temp dir");
        let mut ledger_config = LedgerConfig::default();
        ledger_config.root_path = temp_dir.path().to_path_buf();
        ledger_config.current_epoch = Some("test-epoch".to_string());
        let ledger_writer = LedgerWriter::new(&ledger_config).expect("ledger writer");
        let ledger_reader = LedgerReader::new(ledger_config.root_path.clone());
        let metrics = MetricsCollector::new();

        submit_checkpoint_task(
            &executor,
            ledger_writer.clone(),
            metrics.clone(),
            "test-checkpoint".to_string(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            RouterReplayState::default(),
            LeaseReplayState::default(),
            metrics.get_snapshot(),
        );

        tokio::time::sleep(Duration::from_millis(150)).await;
        ledger_writer.flush().await.expect("flush ledger");
        let events = ledger_reader
            .read_epoch(&ledger_writer.epoch_id())
            .expect("read epoch");
        assert!(events
            .iter()
            .any(|event| matches!(event.event, LedgerEvent::Checkpoint(_))));
    }
}
