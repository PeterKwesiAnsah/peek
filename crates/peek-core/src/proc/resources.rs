use resource_sampler::cpu;
use resource_sampler::disk_io;

/// Sample CPU usage over ~200ms and return a percentage (0–100*ncpus).
///
/// Thin wrapper around `resource-sampler` so existing callers keep their
/// imports while the implementation lives in the sampler crate.
pub fn sample_cpu(pid: i32) -> Option<f64> {
    cpu::sample_cpu(pid)
}

/// Return (read_bytes, write_bytes) from `/proc/<pid>/io`.
pub fn read_io(pid: i32) -> anyhow::Result<(u64, u64)> {
    disk_io::read_io(pid)
}
