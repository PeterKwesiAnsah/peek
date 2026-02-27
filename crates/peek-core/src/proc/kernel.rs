use crate::{KernelInfo, NamespaceEntry};

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

    let cgroup = read_cgroup(pid);
    let namespaces = read_namespaces(pid);

    let (cap_permitted, cap_effective, seccomp, vol_ctx, nonvol_ctx) = parse_status(pid);

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

    let priority = fields.get(15).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
    let nice = fields.get(16).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
    let policy_num = fields.get(38).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

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

fn read_cgroup(pid: i32) -> String {
    std::fs::read_to_string(format!("/proc/{}/cgroup", pid))
        .map(|s| {
            s.lines()
                .find(|l| l.starts_with("0:"))
                .or_else(|| s.lines().next())
                .map(|l| {
                    let parts: Vec<&str> = l.splitn(3, ':').collect();
                    parts.get(2).unwrap_or(&"unknown").to_string()
                })
                .unwrap_or_else(|| "unknown".to_string())
        })
        .unwrap_or_else(|_| "unknown".to_string())
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

fn parse_status(
    pid: i32,
) -> (
    String,
    String,
    u32,
    Option<u64>,
    Option<u64>,
) {
    let raw = match std::fs::read_to_string(format!("/proc/{}/status", pid)) {
        Ok(r) => r,
        Err(_) => return ("0".to_string(), "0".to_string(), 0, None, None),
    };

    let mut cap_prm = "0".to_string();
    let mut cap_eff = "0".to_string();
    let mut seccomp = 0u32;
    let mut vol_ctx = None;
    let mut nonvol_ctx = None;

    for line in raw.lines() {
        if let Some(v) = line.strip_prefix("CapPrm:\t") {
            cap_prm = decode_caps(v.trim());
        } else if let Some(v) = line.strip_prefix("CapEff:\t") {
            cap_eff = decode_caps(v.trim());
        } else if let Some(v) = line.strip_prefix("Seccomp:\t") {
            seccomp = v.trim().parse().unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("voluntary_ctxt_switches:\t") {
            vol_ctx = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("nonvoluntary_ctxt_switches:\t") {
            nonvol_ctx = v.trim().parse().ok();
        }
    }

    (cap_prm, cap_eff, seccomp, vol_ctx, nonvol_ctx)
}

fn decode_caps(hex: &str) -> String {
    let val = u64::from_str_radix(hex, 16).unwrap_or(0);
    if val == 0 {
        return "none".to_string();
    }

    const CAP_NAMES: &[&str] = &[
        "CHOWN",
        "DAC_OVERRIDE",
        "DAC_READ_SEARCH",
        "FOWNER",
        "FSETID",
        "KILL",
        "SETGID",
        "SETUID",
        "SETPCAP",
        "LINUX_IMMUTABLE",
        "NET_BIND_SERVICE",
        "NET_BROADCAST",
        "NET_ADMIN",
        "NET_RAW",
        "IPC_LOCK",
        "IPC_OWNER",
        "SYS_MODULE",
        "SYS_RAWIO",
        "SYS_CHROOT",
        "SYS_PTRACE",
        "SYS_PACCT",
        "SYS_ADMIN",
        "SYS_BOOT",
        "SYS_NICE",
        "SYS_RESOURCE",
        "SYS_TIME",
        "SYS_TTY_CONFIG",
        "MKNOD",
        "LEASE",
        "AUDIT_WRITE",
        "AUDIT_CONTROL",
        "SETFCAP",
        "MAC_OVERRIDE",
        "MAC_ADMIN",
        "SYSLOG",
        "WAKE_ALARM",
        "BLOCK_SUSPEND",
        "AUDIT_READ",
        "PERFMON",
        "BPF",
        "CHECKPOINT_RESTORE",
    ];

    let caps: Vec<&str> = CAP_NAMES
        .iter()
        .enumerate()
        .filter(|(i, _)| val & (1u64 << i) != 0)
        .map(|(_, name)| *name)
        .collect();

    if caps.is_empty() {
        hex.to_string()
    } else {
        caps.join(", ")
    }
}

