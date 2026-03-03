// RSS, PSS, swap from smaps_rollup and status. Plan: optional PSS/swap for ProcessInfo.

/// Memory snapshot: RSS (from smaps_rollup), PSS, and swap in KB.
/// Returns None if files are missing or unreadable (e.g. no permission).
#[cfg(target_os = "linux")]
pub fn sample_memory(pid: i32) -> Option<(u64, u64, u64)> {
    let rss_kb = read_smaps_rollup_field(pid, "Rss")?;
    let pss_kb = read_smaps_rollup_field(pid, "Pss")?;
    let swap_kb = read_vmswap_kb(pid).unwrap_or(0);
    Some((rss_kb, pss_kb, swap_kb))
}

#[cfg(not(target_os = "linux"))]
pub fn sample_memory(_pid: i32) -> Option<(u64, u64, u64)> {
    None
}

#[cfg(target_os = "linux")]
fn read_smaps_rollup_field(pid: i32, name: &str) -> Option<u64> {
    let path = format!("/proc/{}/smaps_rollup", pid);
    let raw = std::fs::read_to_string(path).ok()?;
    for line in raw.lines() {
        let line = line.trim_start();
        if line.starts_with(name) {
            // "Pss:  1234 kB" or "Rss:  5678 kB"
            let rest = line.strip_prefix(name)?.trim_start();
            let rest = rest.strip_prefix(':')?.trim_start();
            let num: u64 = rest.split_whitespace().next()?.parse().ok()?;
            return Some(num);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn read_vmswap_kb(pid: i32) -> Option<u64> {
    let path = format!("/proc/{}/status", pid);
    let raw = std::fs::read_to_string(path).ok()?;
    for line in raw.lines() {
        if line.starts_with("VmSwap:") {
            let rest = line.strip_prefix("VmSwap:")?.trim_start();
            let num: u64 = rest
                .split_whitespace()
                .next()?
                .replace(',', "")
                .parse()
                .ok()?;
            return Some(num);
        }
    }
    Some(0)
}
