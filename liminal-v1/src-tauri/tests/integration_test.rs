use liminal_v1::router::{Message, Priority, UnifiedMessageRouter};
use liminal_v1::territory::TerritoryManager;
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_scenario_message_routing() {
    let router = Arc::new(UnifiedMessageRouter::new());

    let msg = Message {
        content: "Test message".to_string(),
        priority: Priority::Coordinate,
        sender: "Agent_A".to_string(),
        recipient: "Agent_B".to_string(),
    };

    router.route_message(msg.clone()).await;

    let pending = router.get_pending_messages().await;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].content, "Test message");
}

#[tokio::test]
async fn test_scenario_territory_management() {
    let territory_manager = Arc::new(TerritoryManager::new());
    let agent_a_id = "Agent_A".to_string();
    let agent_b_id = "Agent_B".to_string();
    let resource = "shared_file.txt".to_string();

    assert!(territory_manager.acquire_lease(&agent_a_id, &resource));

    assert!(!territory_manager.acquire_lease(&agent_b_id, &resource));

    territory_manager.release_lease(&agent_a_id, &resource);

    assert!(territory_manager.acquire_lease(&agent_b_id, &resource));

    territory_manager.release_lease(&agent_b_id, &resource);
}

#[tokio::test]
async fn test_full_scenario_workflow() {
    let router = Arc::new(UnifiedMessageRouter::new());
    let territory_manager = Arc::new(TerritoryManager::new());

    let agent_a_id = "Agent_A".to_string();
    let agent_b_id = "Agent_B".to_string();
    let resource = "shared_file.txt".to_string();

    let acquired_a = territory_manager.acquire_lease(&agent_a_id, &resource);
    assert!(acquired_a);

    let msg_a = Message {
        content: "Hello from Agent A! I have the lease.".to_string(),
        priority: Priority::Coordinate,
        sender: agent_a_id.clone(),
        recipient: agent_b_id.clone(),
    };

    router.route_message(msg_a).await;

    territory_manager.release_lease(&agent_a_id, &resource);

    let acquired_b = territory_manager.acquire_lease(&agent_b_id, &resource);
    assert!(acquired_b);

    let msg_b = Message {
        content: "Hello from Agent B! I have acquired the lease now.".to_string(),
        priority: Priority::Coordinate,
        sender: agent_b_id.clone(),
        recipient: agent_a_id.clone(),
    };

    router.route_message(msg_b).await;

    territory_manager.release_lease(&agent_b_id, &resource);

    let pending = router.get_pending_messages().await;
    assert_eq!(pending.len(), 2);
}

#[tokio::test]
async fn test_message_priority_ordering() {
    let router = Arc::new(UnifiedMessageRouter::new());

    let critical_msg = Message {
        content: "Critical message".to_string(),
        priority: Priority::Critical,
        sender: "Agent_A".to_string(),
        recipient: "Agent_B".to_string(),
    };

    let coordinate_msg = Message {
        content: "Coordinate message".to_string(),
        priority: Priority::Coordinate,
        sender: "Agent_A".to_string(),
        recipient: "Agent_B".to_string(),
    };

    let info_msg = Message {
        content: "Info message".to_string(),
        priority: Priority::Info,
        sender: "Agent_A".to_string(),
        recipient: "Agent_B".to_string(),
    };

    router.route_message(info_msg).await;
    router.route_message(coordinate_msg).await;
    router.route_message(critical_msg).await;

    let pending = router.get_pending_messages().await;
    assert_eq!(pending.len(), 3);
    assert_eq!(pending[0].content, "Critical message");
    assert_eq!(pending[1].content, "Coordinate message");
    assert_eq!(pending[2].content, "Info message");
}

#[tokio::test]
async fn test_concurrent_lease_attempts() {
    let territory_manager = Arc::new(TerritoryManager::new());
    let resource = "shared_resource.txt".to_string();

    let tm1 = Arc::clone(&territory_manager);
    let tm2 = Arc::clone(&territory_manager);
    let r1 = resource.clone();
    let r2 = resource.clone();

    let handle1 = tokio::spawn(async move { tm1.acquire_lease(&"Agent_1".to_string(), &r1) });

    let handle2 = tokio::spawn(async move { tm2.acquire_lease(&"Agent_2".to_string(), &r2) });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    assert!(result1 ^ result2);
}
