/// Return (read_bytes, write_bytes) from `/proc/<pid>/io`.
///
/// This mirrors the behaviour previously implemented in
/// `peek-core::proc::resources::read_io`.
pub fn read_io(pid: i32) -> anyhow::Result<(u64, u64)> {
    let raw = std::fs::read_to_string(format!("/proc/{}/io", pid))?;
    let mut read_bytes = 0u64;
    let mut write_bytes = 0u64;
    for line in raw.lines() {
        if let Some(val) = line.strip_prefix("read_bytes: ") {
            read_bytes = val.trim().parse().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("write_bytes: ") {
            write_bytes = val.trim().parse().unwrap_or(0);
        }
    }
    Ok((read_bytes, write_bytes))
}
