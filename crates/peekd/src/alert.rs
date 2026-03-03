use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// A metric that can be monitored.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertMetric {
    /// CPU usage percentage (0–100 * nCPUs).
    CpuPercent,
    /// RSS memory in megabytes.
    MemoryMb,
    /// Open file descriptor count.
    FdCount,
    /// Thread count.
    ThreadCount,
}

impl std::fmt::Display for AlertMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertMetric::CpuPercent => write!(f, "cpu_percent"),
            AlertMetric::MemoryMb => write!(f, "memory_mb"),
            AlertMetric::FdCount => write!(f, "fd_count"),
            AlertMetric::ThreadCount => write!(f, "thread_count"),
        }
    }
}

/// Comparison operator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    GreaterThan,
    LessThan,
}

/// How to notify when an alert fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum NotifyMethod {
    /// Write to tracing log (always available).
    Log,
    /// Write to stderr.
    Stderr,
    /// Execute an arbitrary shell command; `{pid}`, `{metric}`, `{value}` are substituted.
    Script { command: String },
}

/// A single monitoring rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub pid: i32,
    pub metric: AlertMetric,
    pub comparison: Comparison,
    pub threshold: f64,
    pub notify: NotifyMethod,
    /// Minimum seconds between repeated firings (default: 60).
    #[serde(default = "default_cooldown")]
    pub cooldown_secs: u64,
    #[serde(skip)]
    pub last_triggered: Option<DateTime<Local>>,
}

fn default_cooldown() -> u64 {
    60
}

/// A fired alert event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub rule_id: String,
    pub pid: i32,
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub ts: DateTime<Local>,
}

/// Engine that holds a set of rules and evaluates them against a new sample.
#[derive(Debug, Default)]
pub struct AlertEngine {
    pub rules: Vec<AlertRule>,
}

impl AlertEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    pub fn remove_rules_for_pid(&mut self, pid: i32) {
        self.rules.retain(|r| r.pid != pid);
    }

    /// Remove a single rule by ID. Returns true if a rule was removed.
    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        let len_before = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);
        self.rules.len() < len_before
    }

    /// Evaluate all rules against the latest process snapshot.
    ///
    /// Returns the list of events that fired.
    pub fn evaluate(&mut self, info: &peek_core::ProcessInfo) -> Vec<AlertEvent> {
        let now = Local::now();
        let mut events = Vec::new();

        for rule in &mut self.rules {
            if rule.pid != info.pid {
                continue;
            }

            // Enforce cooldown
            if let Some(last) = rule.last_triggered {
                let elapsed = now.signed_duration_since(last).num_seconds() as u64;
                if elapsed < rule.cooldown_secs {
                    continue;
                }
            }

            let value = match rule.metric {
                AlertMetric::CpuPercent => info.cpu_percent.unwrap_or(0.0),
                AlertMetric::MemoryMb => info.rss_kb as f64 / 1024.0,
                AlertMetric::FdCount => info.fd_count.unwrap_or(0) as f64,
                AlertMetric::ThreadCount => info.threads as f64,
            };

            let fired = match rule.comparison {
                Comparison::GreaterThan => value > rule.threshold,
                Comparison::LessThan => value < rule.threshold,
            };

            if !fired {
                continue;
            }

            rule.last_triggered = Some(now);
            let event = AlertEvent {
                rule_id: rule.id.clone(),
                pid: rule.pid,
                metric: rule.metric.to_string(),
                value,
                threshold: rule.threshold,
                ts: now,
            };

            // Deliver notification
            deliver(&rule.notify, &event);
            events.push(event);
        }

        events
    }
}

fn deliver(method: &NotifyMethod, event: &AlertEvent) {
    match method {
        NotifyMethod::Log => {
            tracing::warn!(
                rule_id = %event.rule_id,
                pid = event.pid,
                metric = %event.metric,
                value = event.value,
                threshold = event.threshold,
                "alert fired"
            );
        }
        NotifyMethod::Stderr => {
            eprintln!(
                "[peekd alert] rule:{} pid:{} {}={:.2} (threshold {:.2})",
                event.rule_id, event.pid, event.metric, event.value, event.threshold
            );
        }
        NotifyMethod::Script { command } => {
            let cmd = command
                .replace("{pid}", &event.pid.to_string())
                .replace("{metric}", &event.metric)
                .replace("{value}", &format!("{:.2}", event.value));
            let _ = std::process::Command::new("sh").args(["-c", &cmd]).spawn();
        }
    }
}

// ─── JSON wire format for adding rules ───────────────────────────────────────

/// Parsed from `{"action":"alert_add","pid":123,"metric":"cpu_percent",
///               "comparison":"greater_than","threshold":80,"notify":{"type":"log"}}`
#[derive(Debug, Deserialize)]
pub struct AlertAddRequest {
    pub pid: i32,
    pub metric: AlertMetric,
    pub comparison: Comparison,
    pub threshold: f64,
    pub notify: NotifyMethod,
    pub cooldown_secs: Option<u64>,
}

impl AlertAddRequest {
    pub fn into_rule(self, id: String) -> AlertRule {
        AlertRule {
            id,
            pid: self.pid,
            metric: self.metric,
            comparison: self.comparison,
            threshold: self.threshold,
            notify: self.notify,
            cooldown_secs: self.cooldown_secs.unwrap_or_else(default_cooldown),
            last_triggered: None,
        }
    }
}

// ─── Static config (alerts.toml) ─────────────────────────────────────────────

/// Rule as defined in a static config file.
///
/// Example `alerts.toml`:
///
/// ```toml
/// [[rules]]
/// pid = 1234
/// metric = "cpu_percent"
/// comparison = "greater_than"
/// threshold = 80.0
/// cooldown_secs = 60
/// notify = { type = "log" }
/// ```
#[derive(Debug, Deserialize)]
pub struct AlertConfig {
    pub rules: Vec<AlertConfigRule>,
}

#[derive(Debug, Deserialize)]
pub struct AlertConfigRule {
    pub pid: i32,
    pub metric: AlertMetric,
    pub comparison: Comparison,
    pub threshold: f64,
    pub cooldown_secs: Option<u64>,
    pub notify: Option<NotifyMethod>,
}

/// Load alert rules from alerts.toml into the engine and seed watched PIDs.
#[cfg(unix)]
pub fn load_config_into(
    engine: &mut AlertEngine,
    watched_pids: &mut Vec<i32>,
) -> anyhow::Result<()> {
    use std::fs;
    use std::path::PathBuf;

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        candidates.push(PathBuf::from(xdg).join("peek").join("alerts.toml"));
    } else if let Ok(home) = std::env::var("HOME") {
        candidates.push(
            PathBuf::from(home)
                .join(".config")
                .join("peek")
                .join("alerts.toml"),
        );
    }
    candidates.push(PathBuf::from("/etc/peekd/alerts.toml"));
    candidates.push(PathBuf::from("/etc/peek/alerts.toml"));

    let path = match candidates.into_iter().find(|p| p.exists()) {
        Some(p) => p,
        None => return Ok(()), // no config is fine
    };

    let raw = fs::read_to_string(&path)?;
    let cfg: AlertConfig = toml::from_str(&raw)?;

    for (idx, rule) in cfg.rules.into_iter().enumerate() {
        let id = format!("config-{}", idx + 1);
        let alert_rule = AlertRule {
            id: id.clone(),
            pid: rule.pid,
            metric: rule.metric,
            comparison: rule.comparison,
            threshold: rule.threshold,
            notify: rule.notify.unwrap_or(NotifyMethod::Log),
            cooldown_secs: rule.cooldown_secs.unwrap_or_else(default_cooldown),
            last_triggered: None,
        };
        if !watched_pids.contains(&rule.pid) {
            watched_pids.push(rule.pid);
        }
        engine.add_rule(alert_rule);
    }

    tracing::info!("loaded alert rules from {}", path.display());
    Ok(())
}
