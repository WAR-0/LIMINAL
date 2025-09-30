use liminal_v1::config::{AppConfig, LedgerConfig};
use liminal_v1::consensus::{quorum_vote, ConsensusBroker};
use liminal_v1::executor::MaintenanceExecutor;
use liminal_v1::ledger::{
    ConsensusEvent, LeaseEvent, LeaseReplayState, LedgerEvent, LedgerReader, LedgerWriter,
    ReplayCoordinator, RouterEvent, RouterReplayState, StateCheckpoint,
};
use liminal_v1::metrics::MetricsCollector;
use liminal_v1::router::{DispatcherConfig, Message, Priority, UnifiedMessageRouter};
use liminal_v1::territory::{
    LeaseDecision, LeaseRequest, TerritoryEvent, TerritoryManager, TerritoryPolicy,
};
use serde_json;
use std::collections::BTreeMap;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::tempdir;
use tempfile::NamedTempFile;
use tokio::time;

#[tokio::test]
async fn router_dispatches_by_priority() {
    let metrics = MetricsCollector::new();
    let config = DispatcherConfig {
        idle_backoff: Duration::from_millis(5),
        ..DispatcherConfig::default()
    };
    let router = Arc::new(UnifiedMessageRouter::with_config(metrics, config));
    let executor = MaintenanceExecutor::new(2);
    router.set_maintenance_executor(executor).await;
    let mut deliveries = router.subscribe();

    let info = Message {
        content: "info".to_string(),
        priority: Priority::Info,
        sender: "agent".to_string(),
        recipient: "peer".to_string(),
    };
    let coordinate = Message {
        content: "coordinate".to_string(),
        priority: Priority::Coordinate,
        sender: "agent".to_string(),
        recipient: "peer".to_string(),
    };
    let critical = Message {
        content: "critical".to_string(),
        priority: Priority::Critical,
        sender: "agent".to_string(),
        recipient: "peer".to_string(),
    };

    router.route_message(info).await.unwrap();
    router.route_message(coordinate).await.unwrap();
    router.route_message(critical).await.unwrap();

    let first = time::timeout(Duration::from_millis(200), deliveries.recv())
        .await
        .unwrap()
        .unwrap();
    let second = time::timeout(Duration::from_millis(200), deliveries.recv())
        .await
        .unwrap()
        .unwrap();
    let third = time::timeout(Duration::from_millis(200), deliveries.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(first.message.content, "critical");
    assert_eq!(second.message.content, "coordinate");
    assert_eq!(third.message.content, "info");
}

#[tokio::test]
async fn router_applies_aging_boosts() {
    let metrics = MetricsCollector::new();
    let config = DispatcherConfig {
        aging_threshold: Duration::from_millis(60),
        idle_backoff: Duration::from_millis(5),
        token_capacity: 5.0,
        token_refill_rate: 10.0,
        initial_tokens: 0.0,
        max_aging_boosts: 1,
        ..DispatcherConfig::default()
    };
    let router = Arc::new(UnifiedMessageRouter::with_config(metrics, config));
    let executor = MaintenanceExecutor::new(2);
    router.set_maintenance_executor(executor).await;
    let mut deliveries = router.subscribe();

    let info = Message {
        content: "needs boost".to_string(),
        priority: Priority::Info,
        sender: "slow".to_string(),
        recipient: "peer".to_string(),
    };

    router.route_message(info).await.unwrap();

    let delivery = time::timeout(Duration::from_millis(1200), deliveries.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(delivery.message.content, "needs boost");
    assert_eq!(delivery.effective_priority, Priority::Coordinate);
    assert!(delivery.wait_time >= Duration::from_millis(60));
    assert!(delivery.aging_boosts >= 1);
}

#[tokio::test]
async fn router_enforces_token_quota() {
    let metrics = MetricsCollector::new();
    let config = DispatcherConfig {
        idle_backoff: Duration::from_millis(5),
        token_capacity: 2.0,
        token_refill_rate: 4.0,
        initial_tokens: 2.0,
        ..DispatcherConfig::default()
    };
    let router = Arc::new(UnifiedMessageRouter::with_config(metrics, config));
    let executor = MaintenanceExecutor::new(2);
    router.set_maintenance_executor(executor).await;
    let mut deliveries = router.subscribe();

    for label in ["first", "second"] {
        let message = Message {
            content: label.to_string(),
            priority: Priority::Info,
            sender: "quota".to_string(),
            recipient: "peer".to_string(),
        };
        router.route_message(message).await.unwrap();
    }

    let one = time::timeout(Duration::from_millis(200), deliveries.recv())
        .await
        .unwrap()
        .unwrap();
    let two = time::timeout(Duration::from_millis(200), deliveries.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(one.message.content, "first");
    assert_eq!(two.message.content, "second");

    let throttled = Message {
        content: "third".to_string(),
        priority: Priority::Info,
        sender: "quota".to_string(),
        recipient: "peer".to_string(),
    };
    router.route_message(throttled).await.unwrap();

    let immediate = time::timeout(Duration::from_millis(50), deliveries.recv()).await;
    assert!(immediate.is_err());

    let eventual = time::timeout(Duration::from_millis(400), deliveries.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(eventual.message.content, "third");
    assert!(eventual.retry_count > 0);
}

#[tokio::test]
async fn router_rate_limiting_updates_metrics_snapshot() {
    let metrics = MetricsCollector::new();
    let config = DispatcherConfig {
        token_capacity: 1.0,
        token_refill_rate: 0.0,
        initial_tokens: 1.0,
        idle_backoff: Duration::from_millis(5),
        ..DispatcherConfig::default()
    };
    let router = Arc::new(UnifiedMessageRouter::with_config(metrics.clone(), config));
    let executor = MaintenanceExecutor::new(2);
    router.set_maintenance_executor(executor).await;

    for index in 0..3 {
        let message = Message {
            content: format!("limited_{index}"),
            priority: Priority::Info,
            sender: "rate_limited_agent".to_string(),
            recipient: "observer".to_string(),
        };
        router.route_message(message).await.unwrap();
    }

    time::sleep(Duration::from_millis(30)).await;
    let snapshot = metrics.get_snapshot();
    let rate_limit_entry = snapshot
        .rate_limits
        .iter()
        .find(|entry| entry.sender == "rate_limited_agent")
        .cloned();

    assert!(snapshot.router.rate_limited_messages > 0);
    assert!(rate_limit_entry.as_ref().is_some());
    assert!(rate_limit_entry
        .as_ref()
        .map(|entry| entry.rate_limit_hits > 0)
        .unwrap_or(false));
    assert!(
        snapshot
            .router
            .queue_depths
            .get("info")
            .copied()
            .unwrap_or_default()
            >= 1
    );

    drop(router);
}

#[tokio::test]
async fn maintenance_executor_handles_router_and_territory_load() {
    let metrics = MetricsCollector::new();
    let dispatcher_config = DispatcherConfig {
        aging_threshold: Duration::from_millis(30),
        idle_backoff: Duration::from_millis(5),
        token_capacity: 2.0,
        token_refill_rate: 12.0,
        initial_tokens: 1.0,
        max_aging_boosts: 2,
        ..DispatcherConfig::default()
    };
    let router = Arc::new(UnifiedMessageRouter::with_config(
        metrics.clone(),
        dispatcher_config,
    ));
    let executor = MaintenanceExecutor::new(4);
    router.set_maintenance_executor(executor.clone()).await;
    let warmup = Message {
        content: "warmup".to_string(),
        priority: Priority::Info,
        sender: "quota_agent".to_string(),
        recipient: "peer".to_string(),
    };
    router.route_message(warmup).await.unwrap();

    for index in 0..4 {
        let message = Message {
            content: format!("load_{index}"),
            priority: Priority::Info,
            sender: format!("high_priority_{index}"),
            recipient: "peer".to_string(),
        };
        router.route_message(message).await.unwrap();
    }

    let maintenance_target = Message {
        content: "maintenance_target".to_string(),
        priority: Priority::Info,
        sender: "quota_agent".to_string(),
        recipient: "peer".to_string(),
    };
    router.route_message(maintenance_target).await.unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;
    let snapshot_precheck = metrics.get_snapshot();
    let quota_tokens = snapshot_precheck
        .rate_limits
        .iter()
        .find(|entry| entry.sender == "quota_agent")
        .map(|entry| entry.tokens_remaining)
        .unwrap_or(0.0);
    assert!(quota_tokens > 0.0, "token refill did not execute");
    assert!(
        snapshot_precheck.router.rate_limited_messages > 0,
        "router did not record any rate limiting"
    );

    let post_snapshot = metrics.get_snapshot();
    assert!(
        post_snapshot.router.rate_limited_messages > 0,
        "router should record rate limiting during maintenance"
    );
}

#[tokio::test]
async fn ledger_replay_matches_live_metrics() {
    let temp_dir = tempdir().expect("temp dir");
    let mut ledger_config = LedgerConfig::default();
    ledger_config.root_path = temp_dir.path().to_path_buf();
    ledger_config.current_epoch = Some("test-epoch".to_string());
    let ledger_writer = LedgerWriter::new(&ledger_config).expect("ledger writer");
    let ledger_reader = LedgerReader::new(ledger_config.root_path.clone());

    let metrics = MetricsCollector::new();
    let router = Arc::new(UnifiedMessageRouter::with_settings_and_ledger(
        metrics.clone(),
        None,
        Some(ledger_writer.clone()),
    ));
    let territory =
        TerritoryManager::new_with_ledger(metrics.clone(), None, Some(ledger_writer.clone()));

    let mut deliveries = router.subscribe();

    let messages = vec![
        Message {
            content: "coord".into(),
            priority: Priority::Coordinate,
            sender: "agent-a".into(),
            recipient: "agent-b".into(),
        },
        Message {
            content: "critical".into(),
            priority: Priority::Critical,
            sender: "agent-c".into(),
            recipient: "agent-d".into(),
        },
        Message {
            content: "info".into(),
            priority: Priority::Info,
            sender: "agent-a".into(),
            recipient: "agent-b".into(),
        },
    ];

    for msg in messages {
        router.route_message(msg).await.unwrap();
    }

    for _ in 0..3 {
        let _ = time::timeout(Duration::from_millis(500), deliveries.recv())
            .await
            .unwrap()
            .unwrap();
    }

    let lease_request = LeaseRequest::new(
        "agent-a".into(),
        "resource-path".into(),
        Priority::Coordinate,
    );
    let _ = territory.acquire_lease(lease_request).await;

    time::sleep(Duration::from_millis(50)).await;

    ledger_writer.flush().await.expect("flush ledger");

    let snapshot = metrics.get_snapshot();
    let router_state = RouterReplayState {
        total_dispatched: snapshot.performance.total_messages_routed,
        last_priority: snapshot.router.last_dispatched_priority.clone(),
        queue_depths: priority_vec_from_map(&snapshot.router.queue_depths),
    };
    let mut lease_state = LeaseReplayState::default();
    lease_state.deferrals = snapshot.leases.deferrals;
    lease_state.overrides = snapshot.leases.overrides;
    lease_state.escalations = snapshot.leases.escalations;

    let checkpoint = StateCheckpoint {
        checkpoint_id: format!(
            "checkpoint-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ),
        captured_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        router: router_state,
        leases: lease_state,
        metrics: snapshot.clone(),
    };

    ledger_writer
        .record_checkpoint(checkpoint)
        .await
        .expect("record checkpoint");
    ledger_writer
        .flush()
        .await
        .expect("flush ledger after checkpoint");

    let coordinator = ReplayCoordinator::new(ledger_reader);
    let outcome = coordinator
        .replay_epoch(&ledger_writer.epoch_id())
        .expect("replay to succeed");
    let replay_snapshot = outcome.metrics.expect("replay snapshot available");

    let baseline = serde_json::to_string(&snapshot).expect("serialize live snapshot");
    let replayed = serde_json::to_string(&replay_snapshot).expect("serialize replay snapshot");
    assert_eq!(baseline, replayed);
}

#[tokio::test]
async fn quorum_override_records_success() {
    let temp_dir = tempdir().expect("temp dir");
    let mut ledger_config = LedgerConfig::default();
    ledger_config.root_path = temp_dir.path().to_path_buf();
    ledger_config.current_epoch = Some("quorum-test".to_string());
    let ledger_writer = LedgerWriter::new(&ledger_config).expect("ledger writer");
    let ledger_reader = LedgerReader::new(ledger_config.root_path.clone());
    let metrics = MetricsCollector::new();
    let territory =
        TerritoryManager::new_with_ledger(metrics.clone(), None, Some(ledger_writer.clone()));

    let _ = territory
        .acquire_lease(LeaseRequest::new(
            "holder".into(),
            "shared-resource".into(),
            Priority::Coordinate,
        ))
        .await;

    let decision = territory
        .acquire_lease(LeaseRequest::new(
            "contender".into(),
            "shared-resource".into(),
            Priority::Critical,
        ))
        .await;

    assert!(matches!(decision, LeaseDecision::Overridden { .. }));

    time::sleep(Duration::from_millis(50)).await;
    ledger_writer.flush().await.expect("flush ledger");

    let events = ledger_reader
        .read_epoch(&ledger_writer.epoch_id())
        .expect("read ledger");
    let mut commits = Vec::new();
    for envelope in events {
        if let LedgerEvent::Consensus(ConsensusEvent::Commit(signal)) = envelope.event {
            commits.push(signal);
        }
    }
    assert!(!commits.is_empty());
    let last_signal = commits.last().expect("consensus commit present");
    let vector = last_signal.vector.as_ref().expect("vector attached");
    assert!(vector.achieved, "quorum should succeed on override");
    assert_eq!(vector.reason, "override");
    assert_eq!(vector.resource_id, "shared-resource");

    let snapshot = metrics.get_snapshot();
    assert!(snapshot.consensus.success >= 1);
    assert!(snapshot.consensus.success_ratio >= 1.0);
}

#[tokio::test]
async fn heat_map_decays_under_load() {
    let metrics = MetricsCollector::new();
    let mut policy = TerritoryPolicy::from_config(None);
    policy.heat_decay_per_second = 0.8;
    policy.heat_increment = 4.0;
    policy.heat_max = 10.0;
    let territory = TerritoryManager::with_policy(metrics.clone(), policy);

    let resource = "heat-resource".to_string();
    let _ = territory
        .acquire_lease(LeaseRequest::new(
            "heat-holder".into(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;

    let _ = territory
        .acquire_lease(LeaseRequest::new(
            "heat-contender".into(),
            resource.clone(),
            Priority::Critical,
        ))
        .await;

    time::sleep(Duration::from_millis(40)).await;
    let hot_snapshot = territory.heat_snapshot().await;
    assert_eq!(
        hot_snapshot.hottest_resource.as_deref(),
        Some(resource.as_str())
    );
    assert!(hot_snapshot.hottest_score > 0.0);

    time::sleep(Duration::from_millis(220)).await;
    let cooled_snapshot = territory.heat_snapshot().await;
    assert!(cooled_snapshot.hottest_score < hot_snapshot.hottest_score);
}

fn priority_vec_from_map(depths: &BTreeMap<String, usize>) -> Vec<usize> {
    let mut values = Vec::new();
    for index in 0..=Priority::DirectorOverride.as_index() {
        let key = Priority::from_index(index).as_str();
        values.push(*depths.get(key).unwrap_or(&0));
    }
    values
}

fn build_manager_with_policy(policy: TerritoryPolicy) -> TerritoryManager {
    TerritoryManager::with_policy(MetricsCollector::new(), policy)
}

#[tokio::test]
async fn territory_promotes_waiting_request_on_release() {
    let mut policy = TerritoryPolicy::default();
    policy.auto_extend_threshold = Duration::from_millis(1);
    let manager = build_manager_with_policy(policy);
    let resource = "shared_file.txt".to_string();

    let first = manager
        .acquire_lease(LeaseRequest::new(
            "Agent_A".to_string(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    assert!(matches!(first, LeaseDecision::Granted(_)));

    let second = manager
        .acquire_lease(LeaseRequest::new(
            "Agent_B".to_string(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    assert!(matches!(second, LeaseDecision::Queued(_)));

    let released = manager
        .release_lease(&"Agent_A".to_string(), &resource)
        .await;
    assert!(released.is_some());

    let active = manager.current_lease(&resource).await;
    assert_eq!(active.unwrap().holder_id, "Agent_B".to_string());
}

#[tokio::test]
async fn territory_defers_when_holder_near_expiry() {
    let mut policy = TerritoryPolicy::default();
    policy.default_lease_duration = Duration::from_millis(100);
    policy.max_lease_duration = Duration::from_millis(100);
    policy.auto_extend_threshold = Duration::from_secs(5);
    let manager = build_manager_with_policy(policy);
    let resource = "doc.md".to_string();

    let first = manager
        .acquire_lease(LeaseRequest::new(
            "Holder".to_string(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    assert!(matches!(first, LeaseDecision::Granted(_)));

    let second = manager
        .acquire_lease(LeaseRequest::new(
            "Contender".to_string(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    assert!(matches!(second, LeaseDecision::Deferred { .. }));
}

#[tokio::test]
async fn territory_overrides_on_priority_delta() {
    let mut policy = TerritoryPolicy::default();
    policy.override_priority_delta = 1;
    policy.auto_extend_threshold = Duration::from_millis(1);
    let manager = build_manager_with_policy(policy);
    let resource = "plan.json".to_string();

    let base = manager
        .acquire_lease(LeaseRequest::new(
            "Agent_Low".to_string(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    assert!(matches!(base, LeaseDecision::Granted(_)));

    let override_decision = manager
        .acquire_lease(LeaseRequest::new(
            "Agent_High".to_string(),
            resource.clone(),
            Priority::Critical,
        ))
        .await;
    assert!(matches!(
        override_decision,
        LeaseDecision::Overridden { .. }
    ));

    let holder = manager.current_lease(&resource).await.unwrap();
    assert_eq!(holder.holder_id, "Agent_High");
    assert_eq!(holder.priority, Priority::Critical);
}

#[tokio::test]
async fn territory_escalates_on_queue_pressure() {
    let mut policy = TerritoryPolicy::default();
    policy.auto_extend_threshold = Duration::from_millis(1);
    policy.escalation_queue_threshold = 2;
    let manager = build_manager_with_policy(policy);
    let resource = "shared_resource.txt".to_string();
    let mut events = manager.subscribe();

    let grant = manager
        .acquire_lease(LeaseRequest::new(
            "Primary".to_string(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    assert!(matches!(grant, LeaseDecision::Granted(_)));

    let queued_one = manager
        .acquire_lease(LeaseRequest::new(
            "Waiter_1".to_string(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    assert!(matches!(queued_one, LeaseDecision::Queued(_)));

    let queued_two = manager
        .acquire_lease(LeaseRequest::new(
            "Waiter_2".to_string(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    assert!(matches!(queued_two, LeaseDecision::Queued(_)));

    let mut escalated = false;
    for _ in 0..5 {
        let event = time::timeout(Duration::from_millis(200), events.recv())
            .await
            .unwrap()
            .unwrap();
        if matches!(event, TerritoryEvent::Escalated { .. }) {
            escalated = true;
            break;
        }
    }

    assert!(escalated);
}

#[tokio::test]
async fn territory_metrics_reflect_escalation() {
    let metrics = MetricsCollector::new();
    let mut policy = TerritoryPolicy::default();
    policy.auto_extend_threshold = Duration::from_millis(1);
    policy.escalation_queue_threshold = 2;
    let manager = TerritoryManager::with_policy(metrics.clone(), policy);
    let resource = "metrics_resource.txt".to_string();

    let _ = manager
        .acquire_lease(LeaseRequest::new(
            "Primary".to_string(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;

    for idx in 0..2 {
        let _ = manager
            .acquire_lease(LeaseRequest::new(
                format!("Waiter_{idx}"),
                resource.clone(),
                Priority::Coordinate,
            ))
            .await;
    }

    time::sleep(Duration::from_millis(10)).await;
    let snapshot = metrics.get_snapshot();
    assert!(snapshot.leases.escalations > 0);
    assert!(snapshot.leases.total_pending >= 2);
    assert!(
        snapshot
            .leases
            .pending_by_resource
            .get(&resource)
            .copied()
            .unwrap_or_default()
            >= 2
    );
}

#[test]
fn config_overrides_apply_to_router_and_territory() {
    let mut temp_config = NamedTempFile::new().expect("create temp config");
    writeln!(
        temp_config,
        "{}",
        r#"
territory_config:
  default_lease_duration: 90s
  auto_extend_threshold: 20s
  negotiation:
    max_rounds: 6
    timeout: 12s
  escalation:
    queue_threshold: 5
    deadlock_timeout: 240s
  fairness:
    priority_boost_after: 75s
    starvation_threshold: 360s
performance_slas:
  queue_depths:
    blocking_max: 8
    critical_max: 12
  message_routing:
    p50: 0.2s
health_monitoring_kpis:
  queue_health:
    warning_depth: 3
    max_depth: 7
"#
    )
    .expect("write config");

    let previous = std::env::var("LIMINAL_CONFIG_PATH").ok();
    std::env::set_var("LIMINAL_CONFIG_PATH", temp_config.path());

    let app_config = AppConfig::load();
    let metrics = MetricsCollector::new();
    let router = UnifiedMessageRouter::with_settings(metrics.clone(), app_config.router.as_ref());
    let dispatcher = router.dispatcher_config();
    assert_eq!(dispatcher.aging_threshold, Duration::from_millis(200));
    assert_eq!(dispatcher.max_aging_boosts, 2);

    let territory_manager = TerritoryManager::new(metrics, app_config.territory.as_ref());
    let policy = territory_manager.policy();
    assert_eq!(policy.default_lease_duration, Duration::from_secs(90));
    assert_eq!(policy.auto_extend_threshold, Duration::from_secs(20));
    assert_eq!(policy.negotiation_max_rounds, 6);
    assert_eq!(policy.negotiation_timeout, Duration::from_secs(12));
    assert_eq!(policy.escalation_queue_threshold, 5);
    assert_eq!(policy.escalation_deadlock_timeout, Duration::from_secs(240));
    assert_eq!(
        policy.fairness_priority_boost_after,
        Duration::from_secs(75)
    );
    assert_eq!(
        policy.fairness_starvation_threshold,
        Duration::from_secs(360)
    );

    if let Some(value) = previous {
        std::env::set_var("LIMINAL_CONFIG_PATH", value);
    } else {
        std::env::remove_var("LIMINAL_CONFIG_PATH");
    }
}

#[test]
fn pty_metrics_capture_structured_events() {
    let metrics = MetricsCollector::new();
    metrics.record_agent_event("Agent_A", Some("forgeEvent"));
    metrics.record_agent_event("Agent_B", None);

    let snapshot = metrics.get_snapshot();
    assert_eq!(snapshot.pty.total_events, 2);
    assert_eq!(snapshot.pty.events_by_name.get("forgeEvent"), Some(&1));
    assert!(
        snapshot
            .pty
            .events_by_name
            .get("unknown")
            .copied()
            .unwrap_or_default()
            >= 1
    );
    let last_event = snapshot.pty.last_event.expect("last event present");
    assert!(last_event.agent_id == "Agent_A" || last_event.agent_id == "Agent_B");
}

#[tokio::test]
async fn ledger_captures_router_messages() {
    let temp_dir = tempdir().expect("temp dir");
    let mut ledger_config = LedgerConfig::default();
    ledger_config.root_path = temp_dir.path().to_path_buf();
    ledger_config.current_epoch = Some("router-capture-test".to_string());
    let ledger_writer = LedgerWriter::new(&ledger_config).expect("ledger writer");
    let ledger_reader = LedgerReader::new(ledger_config.root_path.clone());

    let metrics = MetricsCollector::new();
    let router = Arc::new(UnifiedMessageRouter::with_settings_and_ledger(
        metrics,
        None,
        Some(ledger_writer.clone()),
    ));
    let mut deliveries = router.subscribe();

    let priorities = [
        Priority::Info,
        Priority::Coordinate,
        Priority::Critical,
        Priority::Info,
        Priority::Coordinate,
        Priority::Info,
        Priority::Critical,
        Priority::Info,
        Priority::Coordinate,
        Priority::Info,
    ];

    for (idx, priority) in priorities.iter().enumerate() {
        router
            .route_message(Message {
                content: format!("message_{idx}"),
                priority: *priority,
                sender: format!("agent_{idx}"),
                recipient: "target".to_string(),
            })
            .await
            .unwrap();
    }

    for _ in 0..10 {
        let _ = time::timeout(Duration::from_millis(500), deliveries.recv()).await;
    }

    ledger_writer.flush().await.expect("flush ledger");

    let events = ledger_reader
        .read_epoch(&ledger_writer.epoch_id())
        .expect("read ledger");

    let mut dispatch_count = 0;
    let mut last_sequence = 0;
    for envelope in events {
        assert!(envelope.sequence > last_sequence);
        last_sequence = envelope.sequence;
        if let LedgerEvent::Router(RouterEvent::Dispatched(_)) = envelope.event {
            dispatch_count += 1;
        }
    }
    assert_eq!(dispatch_count, 10);
}

#[tokio::test]
async fn ledger_records_territory_lease_lifecycle() {
    let temp_dir = tempdir().expect("temp dir");
    let mut ledger_config = LedgerConfig::default();
    ledger_config.root_path = temp_dir.path().to_path_buf();
    ledger_config.current_epoch = Some("lease-lifecycle-test".to_string());
    let ledger_writer = LedgerWriter::new(&ledger_config).expect("ledger writer");
    let ledger_reader = LedgerReader::new(ledger_config.root_path.clone());

    let metrics = MetricsCollector::new();
    let territory =
        TerritoryManager::new_with_ledger(metrics.clone(), None, Some(ledger_writer.clone()));

    let resource = "test-resource".to_string();
    let holder = "lease-holder".to_string();

    let granted = territory
        .acquire_lease(LeaseRequest::new(
            holder.clone(),
            resource.clone(),
            Priority::Coordinate,
        ))
        .await;
    assert!(matches!(granted, LeaseDecision::Granted(_)));

    territory
        .release_lease(&holder, &resource)
        .await
        .expect("release lease");

    time::sleep(Duration::from_millis(50)).await;
    ledger_writer.flush().await.expect("flush ledger");

    let events = ledger_reader
        .read_epoch(&ledger_writer.epoch_id())
        .expect("read ledger");

    let mut granted_count = 0;
    let mut released_count = 0;
    let mut found_heat_score = false;

    for envelope in events {
        if let LedgerEvent::Lease(lease_event) = &envelope.event {
            match lease_event {
                LeaseEvent::Granted(record) => {
                    assert_eq!(record.holder_id, holder);
                    assert_eq!(record.resource_id, resource);
                    granted_count += 1;
                }
                LeaseEvent::Released(record) => {
                    assert_eq!(record.resource_id, resource);
                    released_count += 1;
                }
                _ => {}
            }
        }
        if envelope.metadata.territory_id.is_some() {
            found_heat_score = true;
        }
    }

    assert_eq!(granted_count, 1);
    assert_eq!(released_count, 1);
    assert!(found_heat_score);
}

#[tokio::test]
async fn ledger_records_consensus_quorum_events() {
    let temp_dir = tempdir().expect("temp dir");
    let mut ledger_config = LedgerConfig::default();
    ledger_config.root_path = temp_dir.path().to_path_buf();
    ledger_config.current_epoch = Some("consensus-quorum-test".to_string());
    let ledger_writer = LedgerWriter::new(&ledger_config).expect("ledger writer");
    let ledger_reader = LedgerReader::new(ledger_config.root_path.clone());

    let metrics = MetricsCollector::new();
    let consensus = ConsensusBroker::new(Some(ledger_writer.clone()), metrics, 0.66);

    let votes = vec![
        quorum_vote("agent_a", 2.0, true),
        quorum_vote("agent_b", 1.5, true),
        quorum_vote("agent_c", 1.0, false),
    ];

    let achieved = consensus
        .record_quorum("shared-resource", votes, "priority-override")
        .await;

    time::sleep(Duration::from_millis(50)).await;
    ledger_writer.flush().await.expect("flush ledger");

    let events = ledger_reader
        .read_epoch(&ledger_writer.epoch_id())
        .expect("read ledger");

    let mut proposal_count = 0;
    let mut vote_count = 0;
    let mut commit_count = 0;

    for envelope in events {
        if let LedgerEvent::Consensus(consensus_event) = &envelope.event {
            match consensus_event {
                ConsensusEvent::Proposal(signal) => {
                    assert_eq!(signal.phase, "proposal");
                    if let Some(vector) = &signal.vector {
                        assert_eq!(vector.resource_id, "shared-resource");
                        assert_eq!(vector.threshold, 0.66);
                        assert_eq!(vector.total_weight, 4.5);
                        assert_eq!(vector.agree_weight, 3.5);
                        assert_eq!(vector.achieved, achieved);
                        assert_eq!(vector.reason, "priority-override");
                    }
                    proposal_count += 1;
                }
                ConsensusEvent::Vote(signal) => {
                    assert_eq!(signal.phase, "vote");
                    vote_count += 1;
                }
                ConsensusEvent::Commit(signal) => {
                    assert_eq!(signal.phase, "commit");
                    if let Some(vector) = &signal.vector {
                        assert!(vector.achieved);
                    }
                    commit_count += 1;
                }
                ConsensusEvent::Idle => {}
            }
        }
    }

    assert_eq!(proposal_count, 1);
    assert_eq!(vote_count, 1);
    assert_eq!(commit_count, 1);
}

#[tokio::test]
async fn ledger_replay_rebuilds_state_deterministically() {
    let temp_dir = tempdir().expect("temp dir");
    let mut ledger_config = LedgerConfig::default();
    ledger_config.root_path = temp_dir.path().to_path_buf();
    ledger_config.current_epoch = Some("replay-deterministic-test".to_string());
    let ledger_writer = LedgerWriter::new(&ledger_config).expect("ledger writer");
    let ledger_reader = LedgerReader::new(ledger_config.root_path.clone());

    let metrics = MetricsCollector::new();
    let router = Arc::new(UnifiedMessageRouter::with_settings_and_ledger(
        metrics.clone(),
        None,
        Some(ledger_writer.clone()),
    ));
    let mut deliveries = router.subscribe();

    let messages = vec![
        Message {
            content: "msg1".into(),
            priority: Priority::Critical,
            sender: "agent1".into(),
            recipient: "target".into(),
        },
        Message {
            content: "msg2".into(),
            priority: Priority::Coordinate,
            sender: "agent2".into(),
            recipient: "target".into(),
        },
        Message {
            content: "msg3".into(),
            priority: Priority::Info,
            sender: "agent1".into(),
            recipient: "target".into(),
        },
    ];

    for msg in messages {
        router.route_message(msg).await.unwrap();
    }

    for _ in 0..3 {
        let _ = time::timeout(Duration::from_millis(500), deliveries.recv())
            .await
            .unwrap()
            .unwrap();
    }

    time::sleep(Duration::from_millis(50)).await;
    ledger_writer.flush().await.expect("flush ledger");

    let coordinator = ReplayCoordinator::new(ledger_reader);
    let outcome = coordinator
        .replay_epoch(&ledger_writer.epoch_id())
        .expect("replay");

    assert_eq!(outcome.router.total_dispatched, 3);
    assert!(outcome.router.last_priority.is_some());

    let snapshot = metrics.get_snapshot();
    assert_eq!(
        outcome.router.total_dispatched,
        snapshot.performance.total_messages_routed
    );

    let mut queue_depths_match = true;
    for (idx, depth) in outcome.router.queue_depths.iter().enumerate() {
        let key = Priority::from_index(idx).as_str();
        let live_depth = snapshot.router.queue_depths.get(key).copied().unwrap_or(0);
        if *depth != live_depth {
            queue_depths_match = false;
            break;
        }
    }
    assert!(queue_depths_match);
}
