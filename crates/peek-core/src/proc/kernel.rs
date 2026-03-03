use crate::{KernelInfo, NamespaceEntry};
use kernel_explainer::capabilities::format_caps;
use proc_reader::cgroup::read_cgroup;
use proc_reader::security::read_label;

pub fn collect_kernel(pid: i32) -> anyhow::Result<KernelInfo> {
    let stat_raw = std::fs::read_to_string(format!("/proc/{}/stat", pid))?;
    let (nice, priority, sched_policy) = parse_sched(&stat_raw);

    let oom_score = std::fs::read_to_string(format!("/proc/{}/oom_score", pid))
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(0);

    let oom_score_adj = std::fs::read_to_string(format!("/proc/{}/oom_score_adj", pid))
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(0);

    let cgroup = read_cgroup(pid).unwrap_or_else(|| "unknown".to_string());
    let namespaces = read_namespaces(pid);

    let (cap_permitted, cap_effective, seccomp, vol_ctx, nonvol_ctx) = parse_status(pid);
    let security_label = read_label(pid);

    Ok(KernelInfo {
        sched_policy,
        nice,
        priority,
        oom_score,
        oom_score_adj,
        cgroup,
        namespaces,
        cap_permitted,
        cap_effective,
        seccomp,
        voluntary_ctxt_switches: vol_ctx,
        nonvoluntary_ctxt_switches: nonvol_ctx,
        security_label,
    })
}

fn parse_sched(stat_raw: &str) -> (i32, i32, String) {
    // After comm (find last ')') fields are 0-indexed from ')':
    // [0]=state [1]=ppid ... [15]=priority [16]=nice ... [38]=policy
    let after = match stat_raw.rfind(')') {
        Some(i) => &stat_raw[i + 2..],
        None => return (0, 0, "SCHED_OTHER".to_string()),
    };
    let fields: Vec<&str> = after.split_whitespace().collect();

    let priority = fields
        .get(15)
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    let nice = fields
        .get(16)
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    let policy_num = fields
        .get(38)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let policy = match policy_num {
        0 => "SCHED_OTHER",
        1 => "SCHED_FIFO",
        2 => "SCHED_RR",
        3 => "SCHED_BATCH",
        5 => "SCHED_IDLE",
        6 => "SCHED_DEADLINE",
        _ => "UNKNOWN",
    };

    (nice, priority, policy.to_string())
}

fn read_namespaces(pid: i32) -> Vec<NamespaceEntry> {
    let ns_types = ["cgroup", "ipc", "mnt", "net", "pid", "time", "user", "uts"];
    let mut entries = Vec::new();
    for ns_type in &ns_types {
        let path = format!("/proc/{}/ns/{}", pid, ns_type);
        if let Ok(target) = std::fs::read_link(&path) {
            let inode = target
                .to_string_lossy()
                .split('[')
                .nth(1)
                .and_then(|s| s.split(']').next())
                .unwrap_or("?")
                .to_string();
            entries.push(NamespaceEntry {
                ns_type: ns_type.to_string(),
                inode,
            });
        }
    }
    entries
}

fn parse_status(pid: i32) -> (String, String, u32, Option<u64>, Option<u64>) {
    let raw = match std::fs::read_to_string(format!("/proc/{}/status", pid)) {
        Ok(r) => r,
        Err(_) => return ("0".to_string(), "0".to_string(), 0, None, None),
    };

    let mut cap_prm_bits = 0u64;
    let mut cap_eff_bits = 0u64;
    let mut seccomp = 0u32;
    let mut vol_ctx = None;
    let mut nonvol_ctx = None;

    for line in raw.lines() {
        if let Some(v) = line.strip_prefix("CapPrm:\t") {
            cap_prm_bits = u64::from_str_radix(v.trim(), 16).unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("CapEff:\t") {
            cap_eff_bits = u64::from_str_radix(v.trim(), 16).unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("Seccomp:\t") {
            seccomp = v.trim().parse().unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("voluntary_ctxt_switches:\t") {
            vol_ctx = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("nonvoluntary_ctxt_switches:\t") {
            nonvol_ctx = v.trim().parse().ok();
        }
    }

    let (cap_prm, cap_eff) = format_caps(cap_prm_bits, cap_eff_bits);

    (cap_prm, cap_eff, seccomp, vol_ctx, nonvol_ctx)
}
