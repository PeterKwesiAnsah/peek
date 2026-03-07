// Pre-flight impact: TCP count, children, file locks.
//
// Logic moved from `peek-core::proc::signal` so it can be reused by multiple
// frontends (CLI, daemon, future UI) without depending on peek-core.

use serde::{Deserialize, Serialize};

use crate::systemd::detect_systemd_unit;
use network_inspector::tcp;

/// Structured description of the impact of sending a signal to a process.
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

/// Analyse the potential impact of sending a signal to `pid`.
pub fn analyze_impact(pid: i32) -> anyhow::Result<SignalImpact> {
    let active_tcp_connections = count_tcp_connections(pid);
    let child_process_count = count_children(pid);
    let has_file_locks = check_file_locks(pid);
    let systemd_unit = detect_systemd_unit(pid);

    let (recommendation, prefer_graceful) = build_recommendation(
        active_tcp_connections,
        child_process_count,
        has_file_locks,
        &systemd_unit,
    );

    Ok(SignalImpact {
        active_tcp_connections,
        child_process_count,
        has_file_locks,
        systemd_unit,
        recommendation,
        prefer_graceful,
    })
}

// ─── TCP connection counting ─────────────────────────────────────────────────

fn count_tcp_connections(pid: i32) -> usize {
    let inodes = tcp::process_socket_inodes(pid);
    if inodes.is_empty() {
        return 0;
    }
    let mut count = 0usize;
    for path in &["/proc/net/tcp", "/proc/net/tcp6"] {
        if let Ok(raw) = std::fs::read_to_string(path) {
            for line in raw.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() < 10 {
                    continue;
                }
                let inode: u64 = fields[9].parse().unwrap_or(0);
                if inodes.contains(&inode) {
                    // State 01 = ESTABLISHED, 0A = LISTEN (skip listen)
                    let state = u8::from_str_radix(fields[3], 16).unwrap_or(0);
                    if state == 0x01 {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

// ─── Child counting ──────────────────────────────────────────────────────────

fn count_children(pid: i32) -> usize {
    let raw = match std::fs::read_to_string(format!("/proc/{}/task/{}/children", pid, pid)) {
        Ok(r) => r,
        Err(_) => {
            // Fallback: scan all /proc/*/stat
            return count_children_fallback(pid);
        }
    };
    raw.split_whitespace().count()
}

fn count_children_fallback(parent_pid: i32) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if let Ok(p) = s.parse::<i32>() {
                if let Ok(stat) = std::fs::read_to_string(format!("/proc/{}/stat", p)) {
                    let after = stat.rfind(')').map(|i| &stat[i + 2..]).unwrap_or("");
                    let fields: Vec<&str> = after.split_whitespace().collect();
                    let ppid: i32 = fields.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    if ppid == parent_pid {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

// ─── File lock detection ─────────────────────────────────────────────────────

fn check_file_locks(pid: i32) -> bool {
    // /proc/locks lists all kernel file locks; check if pid appears
    if let Ok(raw) = std::fs::read_to_string("/proc/locks") {
        let pid_str = format!("{}", pid);
        for line in raw.lines() {
            // Format: "N: TYPE MAND PERM PID ..."
            let fields: Vec<&str> = line.split_whitespace().collect();
            // Field index 4 is the PID for POSIX locks; 5 for OFD locks
            for &idx in &[4usize, 5usize] {
                if fields.get(idx).copied() == Some(pid_str.as_str()) {
                    // Only flag EXCLUSIVE (WRITE) locks
                    if fields.get(3).map(|s| s.contains("WRITE")).unwrap_or(false) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// ─── Recommendation builder ───────────────────────────────────────────────────

fn build_recommendation(
    tcp: usize,
    children: usize,
    locks: bool,
    unit: &Option<String>,
) -> (String, bool) {
    let mut points = Vec::new();
    let mut prefer_graceful = false;

    if tcp > 0 {
        points.push(format!(
            "{} active TCP connection(s) will be abruptly terminated by a hard kill",
            tcp
        ));
        prefer_graceful = true;
    }
    if children > 0 {
        points.push(format!(
            "{} child process(es) will be orphaned or killed (depending on signal)",
            children
        ));
    }
    if locks {
        points.push(
            "process holds exclusive file lock(s) — hard kill may leave stale locks".to_string(),
        );
        prefer_graceful = true;
    }
    if let Some(unit) = unit {
        points.push(format!(
            "process is managed by systemd unit '{}' — consider 'systemctl stop/restart' instead",
            unit
        ));
        prefer_graceful = true;
    }

    if points.is_empty() {
        (
            "Process appears safe to terminate with SIGKILL.".to_string(),
            false,
        )
    } else {
        let rec = format!(
            "{}{}",
            points.join(". "),
            if prefer_graceful {
                ". Graceful stop (SIGTERM) is recommended."
            } else {
                "."
            }
        );
        (rec, prefer_graceful)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommendation_no_risks() {
        let (msg, graceful) = build_recommendation(0, 0, false, &None);
        assert!(!graceful);
        assert!(msg.contains("safe"));
    }

    #[test]
    fn recommendation_with_tcp() {
        let (msg, graceful) = build_recommendation(5, 0, false, &None);
        assert!(graceful);
        assert!(msg.contains("TCP"));
    }

    #[test]
    fn recommendation_with_systemd() {
        let (msg, graceful) = build_recommendation(0, 0, false, &Some("nginx.service".to_string()));
        assert!(graceful);
        assert!(msg.contains("systemd"));
    }
}
