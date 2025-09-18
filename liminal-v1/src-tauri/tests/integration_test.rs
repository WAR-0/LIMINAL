use liminal_v1::config::AppConfig;
use liminal_v1::metrics::MetricsCollector;
use liminal_v1::router::{DispatcherConfig, Message, Priority, UnifiedMessageRouter};
use liminal_v1::territory::{
    LeaseDecision, LeaseRequest, TerritoryEvent, TerritoryManager, TerritoryPolicy,
};
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
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
