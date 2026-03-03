// /proc/PID/status — name, state, PID, PPID, UID/GID, VmRSS, VmSize, threads, FDSize.
// Logic moved from peek-core::proc::linux (and procfs status()).

#[cfg(target_os = "linux")]
pub fn read_status(_pid: i32) -> Option<()> {
    Some(())
}

#[cfg(not(target_os = "linux"))]
pub fn read_status(_pid: i32) -> Option<()> {
    None
}
