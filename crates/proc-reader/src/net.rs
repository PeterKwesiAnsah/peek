// /proc/PID/net/* and socket inodes. Logic moved from peek-core::proc::network.
#[cfg(target_os = "linux")]
pub fn read_net(_pid: i32) -> Option<()> {
    Some(())
}

#[cfg(not(target_os = "linux"))]
pub fn read_net(_pid: i32) -> Option<()> {
    None
}
