use clap::Parser;

/// Process intelligence tool for Linux.
///
/// Inspect and troubleshoot a single process by PID or name, view live
/// resource usage, network connections, open files, kernel context, GPU,
/// and more. You can also search for processes by TCP/UDP port using
/// `--port <PORT>` and then open an interactive kill/control panel.
#[derive(Debug, Parser)]
#[command(name = "peek", version)]
pub struct Cli {
    /// PID or process name to inspect
    pub target: Option<String>,

    /// Show resource usage dashboard
    #[arg(short = 'r', long)]
    pub resources: bool,

    /// Show kernel context (scheduler, OOM, namespaces, seccomp)
    #[arg(short = 'k', long)]
    pub kernel: bool,

    /// Show network connections and ports
    #[arg(short = 'n', long)]
    pub network: bool,

    /// Resolve remote addresses to hostnames (best-effort, time-bounded)
    #[arg(long)]
    pub resolve: bool,

    /// List open file descriptors
    #[arg(short = 'f', long)]
    pub files: bool,

    /// Show environment variables (secrets redacted)
    #[arg(short = 'e', long)]
    pub env: bool,

    /// Show full process tree
    #[arg(short = 't', long)]
    pub tree: bool,

    /// Live-updating mode (default: 2000ms refresh). Optionally pass interval in ms.
    #[arg(short = 'w', long, value_name = "INTERVAL_MS")]
    pub watch: Option<Option<u64>>,

    /// Interactive kill/control panel
    #[arg(long)]
    pub kill: bool,

    /// Show everything
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Export report format: json | html | md
    #[arg(long, value_name = "FORMAT")]
    pub export: Option<String>,

    /// Raw JSON output (suppress interactive UI)
    #[arg(short = 'j', long)]
    pub json: bool,

    /// JSON snapshot (captured_at, peek_version, process)
    #[arg(long)]
    pub json_snapshot: bool,

    /// Disable colour output
    #[arg(long)]
    pub no_color: bool,

    /// Compare with another process
    #[arg(long, value_name = "PID2")]
    pub diff: Option<i32>,

    /// Show resource history (requires peekd daemon)
    #[arg(long)]
    pub history: bool,

    /// List alert rules (requires peekd)
    #[arg(long)]
    pub alert_list: bool,

    /// Add alert rule: METRIC GT|LT THRESHOLD (e.g. cpu_percent gt 80). Requires target PID.
    #[arg(long, value_name = "METRIC OP THRESHOLD", num_args = 3)]
    pub alert_add: Option<Vec<String>>,

    /// Remove alert rule by rule_id (from --alert-list)
    #[arg(long, value_name = "RULE_ID")]
    pub alert_remove: Option<String>,

    /// Request elevated privileges via sudo
    #[arg(long)]
    pub sudo: bool,

    /// Search for processes listening on or connected to a TCP/UDP PORT
    #[arg(long, value_name = "PORT")]
    pub port: Option<u16>,
}
