// /proc/PID/maps — memory regions.
//
// We keep this parser deliberately simple and focused on the core fields that
// are most useful for diagnostics (address range, permissions, offset, device,
// inode, and optional pathname).

#[derive(Debug, Clone)]
pub struct MapRegion {
    pub address: String,
    pub perms: String,
    pub offset: u64,
    pub dev: String,
    pub inode: u64,
    pub pathname: Option<String>,
}

pub fn read_maps(pid: i32) -> anyhow::Result<Vec<MapRegion>> {
    let path = format!("/proc/{}/maps", pid);
    let raw = std::fs::read_to_string(path)?;
    let mut regions = Vec::new();

    for line in raw.lines() {
        // Format: address perms offset dev inode pathname?
        // Example:
        // 00400000-00452000 r-xp 00000000 fd:01 123456 /usr/bin/bash
        let mut parts = line.split_whitespace();
        let address = match parts.next() {
            Some(a) => a.to_string(),
            None => continue,
        };
        let perms = parts.next().unwrap_or("").to_string();
        let offset_str = parts.next().unwrap_or("0");
        let dev = parts.next().unwrap_or("").to_string();
        let inode_str = parts.next().unwrap_or("0");

        let offset = u64::from_str_radix(offset_str, 16).unwrap_or(0);
        let inode = inode_str.parse::<u64>().unwrap_or(0);

        let pathname = parts.next().map(|s| {
            // The remaining portion of the line (including spaces) belongs to the path.
            let mut rest = s.to_string();
            for p in parts {
                rest.push(' ');
                rest.push_str(p);
            }
            rest
        });

        regions.push(MapRegion {
            address,
            perms,
            offset,
            dev,
            inode,
            pathname,
        });
    }

    Ok(regions)
}
