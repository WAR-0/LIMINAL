use crate::config::{
    DeadlockFrequencyConfig, EscalationRateConfig, HealthMonitoringConfig, QueueHealthConfig,
};
use crate::metrics::MetricsSnapshot;
use serde::Serialize;
use serde_json::json;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    Normal,
    Warning,
    Critical,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthAlert {
    pub severity: String,
    pub message: String,
    pub context: serde_json::Value,
}

#[derive(Debug)]
pub struct HealthMonitor {
    queue_warning: Option<usize>,
    queue_critical: Option<usize>,
    queue_stale: Option<Duration>,
    escalation_warning_per_min: Option<f64>,
    escalation_critical_per_min: Option<f64>,
    deadlock_warning_per_hour: Option<f64>,
    deadlock_critical_per_hour: Option<f64>,
    last_snapshot_at: Option<Instant>,
    last_rate_limited: u64,
    last_escalations: u64,
    queue_severity: Severity,
    rate_limit_severity: Severity,
    escalation_severity: Severity,
    deadlock_severity: Severity,
}

impl HealthMonitor {
    pub fn new(config: Option<&HealthMonitoringConfig>) -> Self {
        let mut monitor = Self {
            queue_warning: None,
            queue_critical: None,
            queue_stale: None,
            escalation_warning_per_min: None,
            escalation_critical_per_min: None,
            deadlock_warning_per_hour: None,
            deadlock_critical_per_hour: None,
            last_snapshot_at: None,
            last_rate_limited: 0,
            last_escalations: 0,
            queue_severity: Severity::Normal,
            rate_limit_severity: Severity::Normal,
            escalation_severity: Severity::Normal,
            deadlock_severity: Severity::Normal,
        };

        if let Some(cfg) = config {
            monitor.apply_queue_config(cfg.queue_health.as_ref());
            monitor.apply_escalation_config(cfg.escalation_rate.as_ref());
            monitor.apply_deadlock_config(cfg.deadlock_frequency.as_ref());
        }

        monitor
    }

    fn apply_queue_config(&mut self, config: Option<&QueueHealthConfig>) {
        if let Some(queue) = config {
            self.queue_warning = queue.warning_depth;
            self.queue_critical = queue.max_depth;
            self.queue_stale = queue
                .stale_threshold
                .as_deref()
                .and_then(crate::config::parse_duration);
        }
    }

    fn apply_escalation_config(&mut self, config: Option<&EscalationRateConfig>) {
        if let Some(rate) = config {
            self.escalation_warning_per_min = rate
                .warning
                .as_deref()
                .and_then(|value| parse_frequency_per_minute(value));
            self.escalation_critical_per_min = rate
                .critical
                .as_deref()
                .and_then(|value| parse_frequency_per_minute(value));
        }
    }

    fn apply_deadlock_config(&mut self, config: Option<&DeadlockFrequencyConfig>) {
        if let Some(deadlock) = config {
            self.deadlock_warning_per_hour = deadlock
                .warning
                .as_deref()
                .and_then(|value| parse_frequency_per_hour(value));
            self.deadlock_critical_per_hour = deadlock
                .critical
                .as_deref()
                .and_then(|value| parse_frequency_per_hour(value));
        }
    }

    pub fn evaluate(&mut self, snapshot: &MetricsSnapshot) -> Vec<HealthAlert> {
        let mut alerts = Vec::new();
        let now = Instant::now();
        let elapsed = self
            .last_snapshot_at
            .map(|previous| now.saturating_duration_since(previous))
            .unwrap_or_default();

        if let Some(alert) = self.evaluate_queue(snapshot) {
            alerts.push(alert);
        }

        if elapsed > Duration::from_secs(0) {
            if let Some(alert) = self.evaluate_rate_limit(snapshot, elapsed) {
                alerts.push(alert);
            }
            if let Some(alert) = self.evaluate_escalations(snapshot, elapsed) {
                alerts.push(alert);
            }
            if let Some(alert) = self.evaluate_deadlocks(snapshot, elapsed) {
                alerts.push(alert);
            }
        }

        self.last_snapshot_at = Some(now);
        self.last_rate_limited = snapshot.performance.rate_limited_messages;
        self.last_escalations = snapshot.leases.escalations;

        alerts
    }

    fn evaluate_queue(&mut self, snapshot: &MetricsSnapshot) -> Option<HealthAlert> {
        if self.queue_warning.is_none() && self.queue_critical.is_none() {
            return None;
        }
        let mut worst_depth = 0usize;
        let mut worst_priority = String::new();
        for (priority, depth) in snapshot.router.queue_depths.iter() {
            if *depth > worst_depth {
                worst_depth = *depth;
                worst_priority = priority.clone();
            }
        }
        let mut severity = Severity::Normal;
        if let Some(critical) = self.queue_critical {
            if worst_depth >= critical {
                severity = Severity::Critical;
            }
        }
        if severity != Severity::Critical {
            if let Some(warning) = self.queue_warning {
                if worst_depth >= warning {
                    severity = Severity::Warning;
                }
            }
        }
        if severity > self.queue_severity {
            self.queue_severity = severity;
            if severity != Severity::Normal {
                return Some(HealthAlert {
                    severity: severity_to_str(severity).to_string(),
                    message: format!(
                        "Queue depth {} for priority {} exceeded threshold",
                        worst_depth, worst_priority
                    ),
                    context: json!({
                        "priority": worst_priority,
                        "depth": worst_depth,
                        "warning": self.queue_warning,
                        "critical": self.queue_critical,
                        "queueDepths": snapshot.router.queue_depths,
                    }),
                });
            }
        } else if severity == Severity::Normal {
            self.queue_severity = Severity::Normal;
        }
        None
    }

    fn evaluate_rate_limit(
        &mut self,
        snapshot: &MetricsSnapshot,
        elapsed: Duration,
    ) -> Option<HealthAlert> {
        if self.escalation_warning_per_min.is_none() && self.escalation_critical_per_min.is_none() {
            return None;
        }
        let delta_hits = snapshot
            .performance
            .rate_limited_messages
            .saturating_sub(self.last_rate_limited);
        if delta_hits == 0 {
            if self.rate_limit_severity != Severity::Normal {
                self.rate_limit_severity = Severity::Normal;
            }
            return None;
        }
        let per_minute = rate_per_minute(delta_hits, elapsed);
        let mut severity = Severity::Normal;
        if let Some(critical) = self.escalation_critical_per_min {
            if per_minute >= critical {
                severity = Severity::Critical;
            }
        }
        if severity != Severity::Critical {
            if let Some(warning) = self.escalation_warning_per_min {
                if per_minute >= warning {
                    severity = Severity::Warning;
                }
            }
        }
        if severity > self.rate_limit_severity {
            self.rate_limit_severity = severity;
            return Some(HealthAlert {
                severity: severity_to_str(severity).to_string(),
                message: format!(
                    "Rate limiting at {:.2} hits/min exceeds threshold",
                    per_minute
                ),
                context: json!({
                    "ratePerMinute": per_minute,
                    "deltaHits": delta_hits,
                    "warning": self.escalation_warning_per_min,
                    "critical": self.escalation_critical_per_min,
                }),
            });
        } else if severity == Severity::Normal {
            self.rate_limit_severity = Severity::Normal;
        }
        None
    }

    fn evaluate_escalations(
        &mut self,
        snapshot: &MetricsSnapshot,
        elapsed: Duration,
    ) -> Option<HealthAlert> {
        if self.escalation_warning_per_min.is_none() && self.escalation_critical_per_min.is_none() {
            return None;
        }
        let delta = snapshot
            .leases
            .escalations
            .saturating_sub(self.last_escalations);
        if delta == 0 {
            if self.escalation_severity != Severity::Normal {
                self.escalation_severity = Severity::Normal;
            }
            return None;
        }
        let per_minute = rate_per_minute(delta, elapsed);
        let mut severity = Severity::Normal;
        if let Some(critical) = self.escalation_critical_per_min {
            if per_minute >= critical {
                severity = Severity::Critical;
            }
        }
        if severity != Severity::Critical {
            if let Some(warning) = self.escalation_warning_per_min {
                if per_minute >= warning {
                    severity = Severity::Warning;
                }
            }
        }
        if severity > self.escalation_severity {
            self.escalation_severity = severity;
            return Some(HealthAlert {
                severity: severity_to_str(severity).to_string(),
                message: format!(
                    "Lease escalations at {:.2} per min exceed threshold",
                    per_minute
                ),
                context: json!({
                    "ratePerMinute": per_minute,
                    "deltaEscalations": delta,
                    "warning": self.escalation_warning_per_min,
                    "critical": self.escalation_critical_per_min,
                }),
            });
        } else if severity == Severity::Normal {
            self.escalation_severity = Severity::Normal;
        }
        None
    }

    fn evaluate_deadlocks(
        &mut self,
        snapshot: &MetricsSnapshot,
        elapsed: Duration,
    ) -> Option<HealthAlert> {
        if self.deadlock_warning_per_hour.is_none() && self.deadlock_critical_per_hour.is_none() {
            return None;
        }
        let delta = snapshot
            .leases
            .escalations
            .saturating_sub(self.last_escalations);
        if delta == 0 {
            if self.deadlock_severity != Severity::Normal {
                self.deadlock_severity = Severity::Normal;
            }
            return None;
        }
        let per_hour = rate_per_hour(delta, elapsed);
        let mut severity = Severity::Normal;
        if let Some(critical) = self.deadlock_critical_per_hour {
            if per_hour >= critical {
                severity = Severity::Critical;
            }
        }
        if severity != Severity::Critical {
            if let Some(warning) = self.deadlock_warning_per_hour {
                if per_hour >= warning {
                    severity = Severity::Warning;
                }
            }
        }
        if severity > self.deadlock_severity {
            self.deadlock_severity = severity;
            return Some(HealthAlert {
                severity: severity_to_str(severity).to_string(),
                message: format!("Deadlock frequency {:.2} per hour is high", per_hour),
                context: json!({
                    "ratePerHour": per_hour,
                    "deltaEscalations": delta,
                    "warning": self.deadlock_warning_per_hour,
                    "critical": self.deadlock_critical_per_hour,
                }),
            });
        } else if severity == Severity::Normal {
            self.deadlock_severity = Severity::Normal;
        }
        None
    }
}

fn severity_to_str(severity: Severity) -> &'static str {
    match severity {
        Severity::Normal => "normal",
        Severity::Warning => "warning",
        Severity::Critical => "critical",
    }
}

fn rate_per_minute(delta: u64, elapsed: Duration) -> f64 {
    if elapsed.is_zero() {
        return delta as f64;
    }
    let per_second = delta as f64 / elapsed.as_secs_f64().max(1e-6);
    per_second * 60.0
}

fn rate_per_hour(delta: u64, elapsed: Duration) -> f64 {
    rate_per_minute(delta, elapsed) * 60.0
}

fn parse_frequency_per_minute(value: &str) -> Option<f64> {
    parse_frequency(value).map(|(amount, unit)| match unit {
        FrequencyUnit::PerHour => amount / 60.0,
        FrequencyUnit::PerMinute => amount,
    })
}

fn parse_frequency_per_hour(value: &str) -> Option<f64> {
    parse_frequency(value).map(|(amount, unit)| match unit {
        FrequencyUnit::PerHour => amount,
        FrequencyUnit::PerMinute => amount * 60.0,
    })
}

fn parse_frequency(value: &str) -> Option<(f64, FrequencyUnit)> {
    let unit = if value.contains("/hour") || value.contains("per hour") {
        FrequencyUnit::PerHour
    } else {
        FrequencyUnit::PerMinute
    };
    let mut max_value: Option<f64> = None;
    let mut current = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current.push(ch);
        } else if !current.is_empty() {
            if let Ok(parsed) = current.parse::<f64>() {
                max_value = Some(match max_value {
                    Some(existing) => existing.max(parsed),
                    None => parsed,
                });
            }
            current.clear();
        }
    }
    if !current.is_empty() {
        if let Ok(parsed) = current.parse::<f64>() {
            max_value = Some(match max_value {
                Some(existing) => existing.max(parsed),
                None => parsed,
            });
        }
    }
    max_value.map(|value| (value, unit))
}

#[derive(Debug, Clone, Copy)]
enum FrequencyUnit {
    PerMinute,
    PerHour,
}
