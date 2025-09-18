use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RouterConfig {
    #[serde(default)]
    pub token_bucket_capacity: Option<f64>,
    #[serde(default)]
    pub token_bucket_refill_rate: Option<f64>,
    #[serde(default)]
    pub token_bucket_initial: Option<f64>,
    #[serde(default)]
    pub aging_threshold: Option<String>,
    #[serde(default)]
    pub max_aging_boosts: Option<u8>,
    #[serde(default)]
    pub idle_backoff: Option<String>,
    #[serde(default)]
    pub queue_depth_warning: Option<usize>,
    #[serde(default)]
    pub queue_depth_critical: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TerritoryConfig {
    #[serde(default)]
    pub default_lease_duration: Option<String>,
    #[serde(default)]
    pub max_lease_duration: Option<String>,
    #[serde(default)]
    pub auto_extend_threshold: Option<String>,
    #[serde(default)]
    pub negotiation_timeout: Option<String>,
    #[serde(default)]
    pub negotiation_max_rounds: Option<u32>,
    #[serde(default)]
    pub escalation_queue_threshold: Option<usize>,
    #[serde(default)]
    pub escalation_deadlock_timeout: Option<String>,
    #[serde(default)]
    pub fairness_starvation_threshold: Option<String>,
    #[serde(default)]
    pub fairness_priority_boost_after: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct HealthMonitoringConfig {
    #[serde(default)]
    pub queue_health: Option<QueueHealthConfig>,
    #[serde(default)]
    pub escalation_rate: Option<EscalationRateConfig>,
    #[serde(default)]
    pub deadlock_frequency: Option<DeadlockFrequencyConfig>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct QueueHealthConfig {
    #[serde(default)]
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub warning_depth: Option<usize>,
    #[serde(default)]
    pub stale_threshold: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct EscalationRateConfig {
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub critical: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct DeadlockFrequencyConfig {
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub critical: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default)]
    pub router: Option<RouterConfig>,
    #[serde(default)]
    pub territory: Option<TerritoryConfig>,
    #[serde(default)]
    pub health_monitoring_kpis: Option<HealthMonitoringConfig>,
}

impl AppConfig {
    pub fn load() -> Self {
        resolve_config_path()
            .and_then(|path| fs::read_to_string(&path).ok())
            .and_then(|raw| serde_yaml::from_str::<RawConfig>(&raw).ok())
            .map(|raw| raw.into())
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawConfig {
    #[serde(default)]
    territory_config: Option<RawTerritoryConfig>,
    #[serde(default)]
    performance_slas: Option<RawPerformanceSlas>,
    #[serde(default)]
    health_monitoring_kpis: Option<HealthMonitoringConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawTerritoryConfig {
    #[serde(default)]
    default_lease_duration: Option<String>,
    #[serde(default)]
    max_lease_duration: Option<String>,
    #[serde(default)]
    auto_extend_threshold: Option<String>,
    #[serde(default)]
    negotiation: Option<RawNegotiationConfig>,
    #[serde(default)]
    escalation: Option<RawEscalationConfig>,
    #[serde(default)]
    fairness: Option<RawFairnessConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawNegotiationConfig {
    #[serde(default)]
    timeout: Option<String>,
    #[serde(default)]
    max_rounds: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawEscalationConfig {
    #[serde(default)]
    queue_threshold: Option<usize>,
    #[serde(default)]
    deadlock_timeout: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawFairnessConfig {
    #[serde(default)]
    starvation_threshold: Option<String>,
    #[serde(default)]
    priority_boost_after: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawPerformanceSlas {
    #[serde(default)]
    queue_depths: Option<RawQueueDepths>,
    #[serde(default)]
    message_routing: Option<RawLatencyTargets>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawQueueDepths {
    #[serde(default)]
    critical_max: Option<usize>,
    #[serde(default)]
    blocking_max: Option<usize>,
    #[serde(default)]
    coordinate_max: Option<usize>,
    #[serde(default)]
    info_max: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawLatencyTargets {
    #[serde(default)]
    p50: Option<String>,
    #[serde(default)]
    p99: Option<String>,
    #[serde(default)]
    p999: Option<String>,
}

impl From<RawConfig> for AppConfig {
    fn from(raw: RawConfig) -> Self {
        let territory = raw.territory_config.map(|config| {
            let negotiation_timeout = config.negotiation.as_ref().and_then(|n| n.timeout.clone());
            let negotiation_max_rounds = config.negotiation.as_ref().and_then(|n| n.max_rounds);
            let escalation_queue_threshold =
                config.escalation.as_ref().and_then(|e| e.queue_threshold);
            let escalation_deadlock_timeout = config
                .escalation
                .as_ref()
                .and_then(|e| e.deadlock_timeout.clone());
            let fairness_starvation_threshold = config
                .fairness
                .as_ref()
                .and_then(|f| f.starvation_threshold.clone());
            let fairness_priority_boost_after = config
                .fairness
                .as_ref()
                .and_then(|f| f.priority_boost_after.clone());

            TerritoryConfig {
                default_lease_duration: config.default_lease_duration,
                max_lease_duration: config.max_lease_duration,
                auto_extend_threshold: config.auto_extend_threshold,
                negotiation_timeout,
                negotiation_max_rounds,
                escalation_queue_threshold,
                escalation_deadlock_timeout,
                fairness_starvation_threshold,
                fairness_priority_boost_after,
            }
        });

        let router = raw.performance_slas.map(|slas| RouterConfig {
            token_bucket_capacity: None,
            token_bucket_refill_rate: None,
            token_bucket_initial: None,
            aging_threshold: slas
                .message_routing
                .as_ref()
                .and_then(|latency| latency.p50.clone()),
            max_aging_boosts: None,
            idle_backoff: None,
            queue_depth_warning: slas
                .queue_depths
                .as_ref()
                .and_then(|depths| depths.blocking_max),
            queue_depth_critical: slas
                .queue_depths
                .as_ref()
                .and_then(|depths| depths.critical_max),
        });

        Self {
            router,
            territory,
            health_monitoring_kpis: raw.health_monitoring_kpis,
        }
    }
}

fn resolve_config_path() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("LIMINAL_CONFIG_PATH") {
        let path = PathBuf::from(custom);
        if path.exists() {
            return Some(path);
        }
    }
    let cwd = std::env::current_dir().ok()?;
    let direct = cwd.join("../config/liminal.config.yaml");
    if direct.exists() {
        return Some(direct);
    }
    let workspace = cwd.join("config/liminal.config.yaml");
    if workspace.exists() {
        return Some(workspace);
    }
    None
}

pub fn parse_duration(value: &str) -> Option<std::time::Duration> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (number_part, unit) = if let Some(stripped) = trimmed.strip_suffix("ms") {
        (stripped, "ms")
    } else if let Some(stripped) = trimmed.strip_suffix('s') {
        (stripped, "s")
    } else if let Some(stripped) = trimmed.strip_suffix('m') {
        (stripped, "m")
    } else if let Some(stripped) = trimmed.strip_suffix('h') {
        (stripped, "h")
    } else {
        (trimmed, "s")
    };

    let number = number_part.trim().parse::<f64>().ok()?;
    let seconds = match unit {
        "ms" => number / 1000.0,
        "s" => number,
        "m" => number * 60.0,
        "h" => number * 3600.0,
        _ => number,
    };
    Some(std::time::Duration::from_secs_f64(seconds))
}

pub fn parse_f64(value: &Option<String>) -> Option<f64> {
    value.as_ref()?.trim().parse::<f64>().ok()
}

pub fn config_path() -> Option<PathBuf> {
    resolve_config_path()
}
