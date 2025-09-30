use liminal_v1::director::{
    AgentRole, ExecutionEvent, ExecutionSummary, RunbookExecutor, TurnStatus,
};
use liminal_v1::metrics::MetricsCollector;
use liminal_v1::router::UnifiedMessageRouter;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_runbook(temp_dir: &TempDir) -> PathBuf {
    let runbook_content = r#"# Runbook: Test Epoch Phase 1

**Epoch Goal:** Test basic runbook execution

## Turn 1 — Systems Agent
**Specialist:** Systems
**Parallel Group:** N/A (Sequential)
**Dependencies:** None

**Prompt to Delegate:**
> This is turn 1 test prompt.
> It should execute first.

**Acceptance:**
- ✅ Turn 1 complete

## Turn 2 — Interface Agent
**Specialist:** Interface
**Parallel Group:** N/A (Sequential)
**Dependencies:** Turn 1

**Prompt to Delegate:**
> This is turn 2 test prompt.
> It should execute after turn 1.

**Acceptance:**
- ✅ Turn 2 complete
"#;

    let runbook_path = temp_dir.path().join("test_runbook.md");
    fs::write(&runbook_path, runbook_content).expect("Failed to write test runbook");
    runbook_path
}

#[tokio::test]
async fn test_runbook_executor_basic() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_path_buf();

    let metrics = MetricsCollector::new();
    let router = UnifiedMessageRouter::new();

    let executor = RunbookExecutor::new(working_dir.clone(), metrics, router, 3);
    let _rx = executor.subscribe();

    let runbook_path = create_test_runbook(&temp_dir);
    let load_result = executor.load_runbook(&runbook_path).await;
    assert!(load_result.is_ok());

    let summary = load_result.unwrap();
    assert_eq!(summary.epoch_id, "Test Epoch Phase 1");
    assert_eq!(summary.total_turns, 2);
}

#[tokio::test]
async fn test_runbook_executor_events() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_path_buf();

    let metrics = MetricsCollector::new();
    let router = UnifiedMessageRouter::new();

    let executor = RunbookExecutor::new(working_dir.clone(), metrics, router, 3);
    let mut rx = executor.subscribe();

    let runbook_path = create_test_runbook(&temp_dir);
    let _ = executor.load_runbook(&runbook_path).await;

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    });

    tokio::time::timeout(tokio::time::Duration::from_secs(1), async {
        let mut events = Vec::new();
        while let Ok(event) = rx.recv().await {
            events.push(event);
            if events.len() >= 1 {
                break;
            }
        }
    })
    .await
    .ok();
}

#[tokio::test]
async fn test_runbook_executor_cancellation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_path_buf();

    let metrics = MetricsCollector::new();
    let router = UnifiedMessageRouter::new();

    let executor = RunbookExecutor::new(working_dir.clone(), metrics, router, 3);

    let result = executor.cancel(false).await;
    assert!(result.is_ok());

    let result = executor.cancel(true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_runbook_executor_max_parallel() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_path_buf();

    let metrics = MetricsCollector::new();
    let router = UnifiedMessageRouter::new();

    let executor = RunbookExecutor::new(working_dir.clone(), metrics, router, 5);

    let runbook_path = create_test_runbook(&temp_dir);
    let load_result = executor.load_runbook(&runbook_path).await;
    assert!(load_result.is_ok());
}

#[test]
fn test_execution_event_serialization() {
    let event = ExecutionEvent::RunbookStarted {
        epoch_id: "test".to_string(),
        total_turns: 2,
        timestamp: 1234567890,
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(json.contains("\"type\":\"runbookStarted\""));
    assert!(json.contains("\"epochId\":\"test\"") || json.contains("\"epoch_id\":\"test\""));
    assert!(json.contains("\"totalTurns\":2") || json.contains("\"total_turns\":2"));
}

#[test]
fn test_execution_summary_creation() {
    let summary = ExecutionSummary {
        epoch_id: "test-epoch".to_string(),
        total_turns: 3,
        completed_turns: 2,
        failed_turns: 1,
        total_duration_ms: 5000,
        turn_summaries: vec![],
    };

    assert_eq!(summary.epoch_id, "test-epoch");
    assert_eq!(summary.total_turns, 3);
    assert_eq!(summary.completed_turns, 2);
    assert_eq!(summary.failed_turns, 1);
}

#[tokio::test]
async fn test_runbook_executor_status_queries() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_path_buf();

    let metrics = MetricsCollector::new();
    let router = UnifiedMessageRouter::new();

    let executor = RunbookExecutor::new(working_dir.clone(), metrics, router, 3);

    let status = executor.get_status();
    assert!(status.is_empty());

    let summary = executor.get_summary();
    assert!(summary.is_none());

    let runbook_path = create_test_runbook(&temp_dir);
    let _ = executor.load_runbook(&runbook_path).await;

    let summary = executor.get_summary();
    assert!(summary.is_some());
    assert_eq!(summary.unwrap().total_turns, 2);
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_path_buf();

    let metrics = MetricsCollector::new();
    let router = UnifiedMessageRouter::new();

    let executor = RunbookExecutor::new(working_dir.clone(), metrics, router, 3);

    let mut rx1 = executor.subscribe();
    let mut rx2 = executor.subscribe();
    let mut rx3 = executor.subscribe();

    assert!(rx1.try_recv().is_err());
    assert!(rx2.try_recv().is_err());
    assert!(rx3.try_recv().is_err());
}
