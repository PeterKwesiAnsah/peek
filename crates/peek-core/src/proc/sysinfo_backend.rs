// Cross-platform (macOS, Windows, etc.) implementation using sysinfo.

use crate::{PeekError, ProcessInfo, Result};
use chrono::{Local, TimeZone};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

pub fn collect_process_impl(pid: i32, sample_cpu: bool) -> Result<ProcessInfo> {
    let pid_u = pid as u32;
    let sys_pid = Pid::from_u32(pid_u);

    let mut sys = System::new_all();
    if sample_cpu {
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            ProcessRefreshKind::new().with_cpu(),
        );
    } else {
        sys.refresh_processes(ProcessesToUpdate::All);
    }

    let process = sys.process(sys_pid).ok_or(PeekError::NotFound(pid))?;

    let name = process.name().to_string_lossy().into_owned();
    let cmdline = process
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ");
    let cmdline = if cmdline.is_empty() {
        name.clone()
    } else {
        cmdline
    };
    let exe = process.exe().and_then(|p| p.to_str().map(String::from));

    let state = process.status().to_string();
    let ppid = process.parent().map(|p| p.as_u32() as i32).unwrap_or(0);

    // On non-Linux targets we don't rely on real uid/gid values.
    // The full /proc-based implementation is only compiled on Linux,
    // so it's safe to use placeholder values here for portability.
    let uid: u32 = 0;
    let gid: u32 = 0;

    let started_at = (process.start_time() > 0)
        .then(|| Local.timestamp_opt(process.start_time() as i64, 0).single())
        .flatten();

    let threads = 1; // sysinfo doesn't expose thread count on all platforms
    let rss_kb = process.memory() / 1024;
    let vm_size_kb = process.virtual_memory() / 1024;

    let cpu_percent = if sample_cpu {
        Some(process.cpu_usage() as f64)
    } else {
        None
    };

    let (io_read_bytes, io_write_bytes) = {
        let d = process.disk_usage();
        (Some(d.total_read_bytes), Some(d.total_written_bytes))
    };

    Ok(ProcessInfo {
        pid,
        name,
        cmdline,
        exe,
        state,
        ppid,
        uid,
        gid,
        started_at,
        threads,
        vm_size_kb,
        rss_kb,
        pss_kb: None,
        swap_kb: None,
        cpu_percent,
        io_read_bytes,
        io_write_bytes,
        fd_count: None,
        kernel: None,
        network: None,
        open_files: None,
        env_vars: None,
        process_tree: None,
        gpu: None,
    })
}
