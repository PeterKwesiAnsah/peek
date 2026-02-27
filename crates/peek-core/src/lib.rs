pub mod proc;
pub mod ringbuf;

use serde::{Deserialize, Serialize};

// ─── Core process snapshot ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub cmdline: String,
    /// Resolved executable path from `/proc/<pid>/exe`, when available.
    pub exe: Option<String>,
    pub state: String,
    pub ppid: i32,
    pub uid: u32,
    pub gid: u32,
    pub started_at: Option<chrono::DateTime<chrono::Local>>,
    pub threads: i32,
    pub vm_size_kb: u64,
    pub rss_kb: u64,
    // Extended resource fields
    pub cpu_percent: Option<f64>,
    pub io_read_bytes: Option<u64>,
    pub io_write_bytes: Option<u64>,
    pub fd_count: Option<usize>,
    // Optional rich sections
    pub kernel: Option<KernelInfo>,
    pub network: Option<NetworkInfo>,
    pub open_files: Option<Vec<OpenFile>>,
    pub env_vars: Option<Vec<EnvVar>>,
    pub process_tree: Option<ProcessNode>,
    pub gpu: Option<Vec<GpuInfo>>,
}

// ─── Kernel context ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KernelInfo {
    pub sched_policy: String,
    pub nice: i32,
    pub priority: i32,
    pub oom_score: i32,
    pub oom_score_adj: i32,
    pub cgroup: String,
    pub namespaces: Vec<NamespaceEntry>,
    pub cap_permitted: String,
    pub cap_effective: String,
    pub seccomp: u32,
    pub voluntary_ctxt_switches: Option<u64>,
    pub nonvoluntary_ctxt_switches: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamespaceEntry {
    pub ns_type: String,
    pub inode: String,
}

// ─── Network ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInfo {
    pub listening: Vec<SocketEntry>,
    pub connections: Vec<ConnectionEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SocketEntry {
    pub protocol: String,
    pub local_addr: String,
    pub local_port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionEntry {
    pub protocol: String,
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub state: String,
}

// ─── Files ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenFile {
    pub fd: u32,
    pub fd_type: String,
    pub description: String,
}

// ─── Environment ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub redacted: bool,
}

// ─── Process tree ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessNode {
    pub pid: i32,
    pub name: String,
    pub uid: u32,
    pub rss_kb: u64,
    pub children: Vec<ProcessNode>,
}

// ─── GPU ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuInfo {
    pub index: usize,
    pub name: String,
    pub utilization_percent: Option<f64>,
    pub memory_used_mb: Option<f64>,
    pub memory_total_mb: Option<f64>,
    /// "nvml", "sysfs", or "nvidia-smi"
    pub source: String,
}

// ─── Signal impact pre-flight ─────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignalImpact {
    /// Number of active TCP connections this process has.
    pub active_tcp_connections: usize,
    /// Number of direct child processes.
    pub child_process_count: usize,
    /// Whether the process holds any exclusive file locks.
    pub has_file_locks: bool,
    /// Detected systemd unit name (e.g. "nginx.service"), if any.
    pub systemd_unit: Option<String>,
    /// Human-readable recommendation.
    pub recommendation: String,
    /// Whether a graceful stop is preferred over a hard kill.
    pub prefer_graceful: bool,
}

// ─── FD leak detection ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FdLeakWarning {
    /// FD count at the start of the observation window.
    pub start_count: usize,
    /// FD count at the end.
    pub end_count: usize,
    /// How many consecutive samples showed an increase.
    pub consecutive_increases: usize,
}

// ─── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum PeekError {
    #[error("process {0} not found")]
    NotFound(i32),

    #[error("failed to read /proc for pid {pid}: {source}")]
    ProcIo {
        pid: i32,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse /proc for pid {pid}: {msg}")]
    ProcParse { pid: i32, msg: String },
}

pub type Result<T> = std::result::Result<T, PeekError>;

// ─── Collect options ─────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct CollectOptions {
    pub resources: bool,
    pub kernel: bool,
    pub network: bool,
    pub files: bool,
    pub env: bool,
    pub tree: bool,
    pub gpu: bool,
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Fast baseline snapshot (no CPU sampling, no extended sections).
pub fn collect(pid: i32) -> Result<ProcessInfo> {
    proc::collect_process(pid, false)
}

/// Full snapshot gated by `opts`. On Linux includes kernel, network, files, env, tree, GPU; elsewhere baseline only.
pub fn collect_extended(pid: i32, opts: &CollectOptions) -> Result<ProcessInfo> {
    let mut info = proc::collect_process(pid, opts.resources)?;

    #[cfg(target_os = "linux")]
    {
        if opts.resources {
            info.io_read_bytes  = proc::resources::read_io(pid).map(|io| io.0).ok();
            info.io_write_bytes = proc::resources::read_io(pid).map(|io| io.1).ok();
            info.fd_count       = proc::files::count_fds(pid).ok();
        }
        if opts.kernel   { info.kernel       = proc::kernel::collect_kernel(pid).ok(); }
        if opts.network  { info.network      = proc::network::collect_network(pid).ok(); }
        if opts.files    { info.open_files   = proc::files::collect_files(pid).ok(); }
        if opts.env      { info.env_vars     = proc::env::collect_env(pid).ok(); }
        if opts.tree     { info.process_tree = proc::tree::build_tree(pid).ok(); }
        if opts.gpu      { info.gpu          = Some(proc::gpu::collect_gpu(pid)); }
    }

    Ok(info)
}

/// Pre-flight signal impact analysis. Linux only.
pub fn signal_impact(pid: i32) -> anyhow::Result<SignalImpact> {
    #[cfg(target_os = "linux")]
    return proc::signal::analyze(pid);
    #[cfg(not(target_os = "linux"))]
    anyhow::bail!("Signal impact analysis is only available on Linux")
}

