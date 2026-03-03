// /proc/PID/stat — scheduler stats, CPU time, start time, priority, nice.
// Logic moved from peek-core::proc::linux (and procfs stat()).

#[cfg(target_os = "linux")]
pub fn read_stat(_pid: i32) -> Option<()> {
    Some(())
}

#[cfg(not(target_os = "linux"))]
pub fn read_stat(_pid: i32) -> Option<()> {
    None
}
