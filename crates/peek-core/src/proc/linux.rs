// Linux-only implementation using /proc and procfs.

use crate::{PeekError, ProcessInfo, Result};
use chrono::{DateTime, Local};
use procfs::process::Process;

pub fn collect_process_impl(pid: i32, sample_cpu: bool) -> Result<ProcessInfo> {
    let process = Process::new(pid).map_err(|err| match err {
        procfs::ProcError::NotFound(_) => PeekError::NotFound(pid),
        procfs::ProcError::Io(e, _) => PeekError::ProcIo { pid, source: e },
        other => PeekError::ProcParse {
            pid,
            msg: other.to_string(),
        },
    })?;

    let stat = process
        .stat()
        .map_err(|e| PeekError::ProcParse { pid, msg: e.to_string() })?;
    let status = process
        .status()
        .map_err(|e| PeekError::ProcParse { pid, msg: e.to_string() })?;
    let statm = process
        .statm()
        .map_err(|e| PeekError::ProcParse { pid, msg: e.to_string() })?;

    let name = stat.comm.clone();
    let cmdline = process
        .cmdline()
        .ok()
        .filter(|v| !v.is_empty())
        .map(|v| v.join(" "))
        .unwrap_or_else(|| name.clone());

    let exe = process
        .exe()
        .ok()
        .and_then(|p| p.into_os_string().into_string().ok());

    let state = format_state(stat.state);
    let ppid = stat.ppid;
    let uid = status.ruid;
    let gid = status.rgid;

    let started_at = boot_time().map(|boot| {
        let ticks = procfs::ticks_per_second() as u64;
        let secs = stat.starttime / ticks;
        let nanos = (stat.starttime % ticks) * 1_000_000_000 / ticks;
        boot + chrono::Duration::seconds(secs as i64)
            + chrono::Duration::nanoseconds(nanos as i64)
    });

    let threads = stat.num_threads as i32;
    let vm_size_kb = statm.size * 4;
    let rss_kb = statm.resident * 4;

    let cpu_percent = if sample_cpu {
        super::resources::sample_cpu(pid)
    } else {
        None
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
        cpu_percent,
        io_read_bytes: None,
        io_write_bytes: None,
        fd_count: None,
        kernel: None,
        network: None,
        open_files: None,
        env_vars: None,
        process_tree: None,
        gpu: None,
    })
}

fn format_state(c: char) -> String {
    match c {
        'R' => "Running".to_string(),
        'S' => "Sleeping (interruptible)".to_string(),
        'D' => "Uninterruptible sleep (disk/NFS wait)".to_string(),
        'Z' => "Zombie".to_string(),
        'T' => "Stopped (signal)".to_string(),
        't' => "Tracing stop".to_string(),
        'W' => "Paging".to_string(),
        'X' | 'x' => "Dead".to_string(),
        'I' => "Idle".to_string(),
        other => other.to_string(),
    }
}

fn boot_time() -> Option<DateTime<Local>> {
    procfs::boot_time().ok()
}
