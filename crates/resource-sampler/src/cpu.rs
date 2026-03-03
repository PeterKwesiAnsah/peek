use std::thread;
use std::time::Duration;

/// Sample CPU usage over ~200ms and return a percentage (0–100*ncpus).
///
/// This mirrors the behaviour previously implemented in
/// `peek-core::proc::resources::sample_cpu`.
pub fn sample_cpu(pid: i32) -> Option<f64> {
    let (pid_t1, sys_t1) = read_cpu_ticks(pid)?;
    thread::sleep(Duration::from_millis(200));
    let (pid_t2, sys_t2) = read_cpu_ticks(pid)?;

    let d_pid = pid_t2.saturating_sub(pid_t1) as f64;
    let d_sys = sys_t2.saturating_sub(sys_t1) as f64;

    if d_sys == 0.0 {
        return None;
    }
    // Number of logical CPUs from /proc/stat non-aggregate lines
    let ncpus = cpu_count().max(1) as f64;
    Some((d_pid / d_sys) * 100.0 * ncpus)
}

fn read_cpu_ticks(pid: i32) -> Option<(u64, u64)> {
    // Process ticks from /proc/<pid>/stat fields 14 (utime) + 15 (stime)
    let stat_raw = std::fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
    // Field 14 and 15 are utime and stime (0-indexed: 13 and 14)
    // The comm field may contain spaces, find the closing ')'
    let after_comm = stat_raw.rfind(')').map(|i| &stat_raw[i + 2..])?;
    let fields: Vec<&str> = after_comm.split_whitespace().collect();
    // After ')' and state char: field index 11=utime, 12=stime (0-indexed)
    let utime: u64 = fields.get(11)?.parse().ok()?;
    let stime: u64 = fields.get(12)?.parse().ok()?;
    let process_ticks = utime + stime;

    // Total system ticks from /proc/stat first line
    let kstat_raw = std::fs::read_to_string("/proc/stat").ok()?;
    let first_line = kstat_raw.lines().next()?;
    let cpu_fields: Vec<u64> = first_line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let total_ticks: u64 = cpu_fields.iter().sum();

    Some((process_ticks, total_ticks))
}

fn cpu_count() -> usize {
    std::fs::read_to_string("/proc/stat")
        .map(|s| {
            s.lines()
                .filter(|l| l.starts_with("cpu") && l.len() > 3)
                .count()
        })
        .unwrap_or(1)
}
