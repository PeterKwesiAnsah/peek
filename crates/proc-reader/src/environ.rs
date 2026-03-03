// /proc/PID/environ. Logic moved from peek-core::proc::env.
//
// This crate is responsible for low-level /proc parsing and intentionally
// avoids depending on peek-core types. Callers can turn the raw key/value
// pairs into their own domain structs.

/// Raw environment entry as parsed from `/proc/<pid>/environ`.
#[derive(Debug, Clone)]
pub struct EnvironEntry {
    pub key: String,
    pub value: String,
}

/// Read and parse `/proc/<pid>/environ` into raw key/value pairs.
#[cfg(target_os = "linux")]
pub fn read_environ(pid: i32) -> anyhow::Result<Vec<EnvironEntry>> {
    let raw = std::fs::read(format!("/proc/{}/environ", pid))?;
    let mut vars = Vec::new();

    for entry in raw.split(|&b| b == 0) {
        if entry.is_empty() {
            continue;
        }
        let s = String::from_utf8_lossy(entry);
        if let Some(eq) = s.find('=') {
            let key = s[..eq].to_string();
            let value = s[eq + 1..].to_string();
            vars.push(EnvironEntry { key, value });
        }
    }

    vars.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(vars)
}

/// On non-Linux platforms we don't have /proc; return an empty set.
#[cfg(not(target_os = "linux"))]
pub fn read_environ(_pid: i32) -> anyhow::Result<Vec<EnvironEntry>> {
    Ok(Vec::new())
}
