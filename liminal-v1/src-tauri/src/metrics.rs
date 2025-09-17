use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub message_routing_latency_ms: f64,
    pub agent_spawn_time_ms: f64,
    pub lease_acquisition_time_ms: f64,
    pub total_messages_routed: u64,
    pub total_leases_acquired: u64,
    pub memory_usage_mb: f64,
}

#[derive(Debug)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    timers: Arc<RwLock<HashMap<String, Instant>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics {
                message_routing_latency_ms: 0.0,
                agent_spawn_time_ms: 0.0,
                lease_acquisition_time_ms: 0.0,
                total_messages_routed: 0,
                total_leases_acquired: 0,
                memory_usage_mb: 0.0,
            })),
            timers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn start_timer(&self, timer_name: &str) {
        let mut timers = self.timers.write().unwrap();
        timers.insert(timer_name.to_string(), Instant::now());
    }

    pub fn stop_timer(&self, timer_name: &str) -> Option<Duration> {
        let mut timers = self.timers.write().unwrap();
        timers.remove(timer_name).map(|start| start.elapsed())
    }

    pub fn record_message_routing(&self, duration_ms: f64) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.total_messages_routed += 1;
        metrics.message_routing_latency_ms = (metrics.message_routing_latency_ms
            * (metrics.total_messages_routed - 1) as f64
            + duration_ms)
            / metrics.total_messages_routed as f64;
    }

    pub fn record_agent_spawn(&self, duration_ms: f64) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.agent_spawn_time_ms = duration_ms;
    }

    pub fn record_lease_acquisition(&self, duration_ms: f64) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.total_leases_acquired += 1;
        metrics.lease_acquisition_time_ms = (metrics.lease_acquisition_time_ms
            * (metrics.total_leases_acquired - 1) as f64
            + duration_ms)
            / metrics.total_leases_acquired as f64;
    }

    pub fn update_memory_usage(&self) {
        let mut metrics = self.metrics.write().unwrap();
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let output = Command::new("ps")
                .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()
                .ok();

            if let Some(output) = output {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if let Ok(kb) = text.trim().parse::<f64>() {
                        metrics.memory_usage_mb = kb / 1024.0;
                    }
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            metrics.memory_usage_mb = 0.0;
        }
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.update_memory_usage();
        self.metrics.read().unwrap().clone()
    }

    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().unwrap();
        *metrics = PerformanceMetrics {
            message_routing_latency_ms: 0.0,
            agent_spawn_time_ms: 0.0,
            lease_acquisition_time_ms: 0.0,
            total_messages_routed: 0,
            total_leases_acquired: 0,
            memory_usage_mb: 0.0,
        };
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
