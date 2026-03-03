// NVIDIA/AMD GPU sampling.
//
// Logic moved from `peek-core::proc::gpu`. This crate owns the low-level GPU
// metrics and struct; peek-core re-exports `GpuInfo` so callers keep using
// `peek_core::GpuInfo` as before.

use serde::{Deserialize, Serialize};

/// GPU utilisation snapshot for a single device.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuInfo {
    pub index: usize,
    pub name: String,
    pub utilization_percent: Option<f64>,
    pub memory_used_mb: Option<f64>,
    pub memory_total_mb: Option<f64>,
    /// Memory used by the inspected PID on this GPU (NVIDIA, when available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_used_mb: Option<f64>,
    /// "nvml", "sysfs", or "nvidia-smi"
    pub source: String,
}

/// Attempt to collect GPU utilisation for this process.
///
/// Strategy (in order):
/// 1. Run `nvidia-smi` (NVIDIA) and parse CSV output.
/// 2. Walk `/sys/class/drm/card*/device/` for AMD via ROCm sysfs.
/// 3. Return empty Vec if nothing is found.
///
/// Per-process memory (process_used_mb) is filled when nvidia-smi supports
/// `--query-compute-apps` and this PID has compute processes on a GPU.
pub fn collect_gpu(pid: i32) -> Vec<GpuInfo> {
    // Try nvidia-smi first (includes per-PID attribution when available)
    let mut gpus = try_nvidia_smi(pid);
    if !gpus.is_empty() {
        return gpus;
    }

    // Try AMD sysfs
    gpus = try_amd_sysfs();
    gpus
}

// ─── NVIDIA via nvidia-smi ────────────────────────────────────────────────────

/// Map from GPU uuid (from nvidia-smi) to memory used by the given PID (MiB).
fn query_nvidia_compute_apps(pid: i32) -> std::collections::HashMap<String, f64> {
    let output = match std::process::Command::new("nvidia-smi")
        .args([
            "--query-compute-apps=pid,gpu_uuid,used_memory",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return std::collections::HashMap::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pid_str = pid.to_string();
    let mut map = std::collections::HashMap::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').map(str::trim).collect();
        if parts.len() < 3 {
            continue;
        }
        if parts[0] != pid_str {
            continue;
        }
        let uuid = parts[1].to_string();
        if let Ok(used) = parts[2].parse::<f64>() {
            *map.entry(uuid).or_insert(0.0) += used;
        }
    }
    map
}

fn try_nvidia_smi(pid: i32) -> Vec<GpuInfo> {
    // Query GPUs with uuid so we can match compute-apps to GPU index
    let output = match std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=index,uuid,name,utilization.gpu,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let process_mem = query_nvidia_compute_apps(pid);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').map(str::trim).collect();
        // index,uuid,name,util,mem_used,mem_total (6 columns; uuid may contain dashes only)
        if parts.len() < 6 {
            continue;
        }
        let index: usize = parts[0].parse().unwrap_or(0);
        let uuid = parts[1].to_string();
        let name = parts[2].to_string();
        let util: Option<f64> = parts[3].parse().ok();
        let mem_used: Option<f64> = parts[4].parse().ok();
        let mem_total: Option<f64> = parts[5].parse().ok();
        let process_used_mb = process_mem.get(&uuid).copied();

        gpus.push(GpuInfo {
            index,
            name,
            utilization_percent: util,
            memory_used_mb: mem_used,
            memory_total_mb: mem_total,
            process_used_mb,
            source: "nvidia-smi".to_string(),
        });
    }

    gpus
}

// ─── AMD via sysfs ───────────────────────────────────────────────────────────

fn try_amd_sysfs() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    let drm = match std::fs::read_dir("/sys/class/drm") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let mut index = 0usize;
    let mut entries: Vec<_> = drm.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name();
        let s = name.to_string_lossy();
        // Only top-level card entries, not renderD
        if !s.starts_with("card") || s.contains('-') {
            continue;
        }

        let base = entry.path().join("device");
        if !base.exists() {
            continue;
        }

        // GPU busy percent (AMD)
        let util: Option<f64> = std::fs::read_to_string(base.join("gpu_busy_percent"))
            .ok()
            .and_then(|s| s.trim().parse().ok());

        if util.is_none() {
            // Not an AMD GPU with known sysfs interface
            continue;
        }

        // VRAM used / total (bytes)
        let vram_used_mb: Option<f64> = std::fs::read_to_string(base.join("mem_info_vram_used"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|b| b as f64 / 1_048_576.0);

        let vram_total_mb: Option<f64> = std::fs::read_to_string(base.join("mem_info_vram_total"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|b| b as f64 / 1_048_576.0);

        // Try to get a friendly name from uevent
        let gpu_name = std::fs::read_to_string(base.join("uevent"))
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("DRIVER="))
                    .map(|l| l.to_string())
            })
            .unwrap_or_else(|| format!("AMD GPU ({})", s));

        gpus.push(GpuInfo {
            index,
            name: gpu_name,
            utilization_percent: util,
            memory_used_mb: vram_used_mb,
            memory_total_mb: vram_total_mb,
            process_used_mb: None,
            source: "sysfs/amdgpu".to_string(),
        });
        index += 1;
    }

    gpus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_gpu_returns_vec() {
        // Should not panic regardless of whether a GPU is present.
        let result = collect_gpu(1);
        // Either empty or populated — both are valid.
        let _ = result.len();
    }
}
