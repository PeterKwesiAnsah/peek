use crate::{ConnectionEntry, NetworkInfo, SocketEntry};
use std::collections::HashSet;
use std::net::{Ipv4Addr, Ipv6Addr};

pub fn collect_network(pid: i32) -> anyhow::Result<NetworkInfo> {
    // 1. Find socket inodes belonging to this process
    let socket_inodes = process_socket_inodes(pid);

    // 2. Parse kernel network tables
    let mut listening = Vec::new();
    let mut connections = Vec::new();

    for &is_v6 in &[false, true] {
        for &udp in &[false, true] {
            let proto = match (is_v6, udp) {
                (false, false) => "TCP",
                (false, true) => "UDP",
                (true, false) => "TCP6",
                (true, true) => "UDP6",
            };
            let path = match (is_v6, udp) {
                (false, false) => "/proc/net/tcp",
                (false, true) => "/proc/net/udp",
                (true, false) => "/proc/net/tcp6",
                (true, true) => "/proc/net/udp6",
            };

            if let Ok(raw) = std::fs::read_to_string(path) {
                for line in raw.lines().skip(1) {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() < 10 {
                        continue;
                    }
                    let inode: u64 = fields[9].parse().unwrap_or(0);
                    if !socket_inodes.contains(&inode) {
                        continue;
                    }

                    let local = parse_addr(fields[1], is_v6);
                    let remote = parse_addr(fields[2], is_v6);
                    let state_hex = u8::from_str_radix(fields[3], 16).unwrap_or(0);
                    let state = tcp_state(state_hex);

                    if state == "LISTEN" {
                        listening.push(SocketEntry {
                            protocol: proto.to_string(),
                            local_addr: local.0,
                            local_port: local.1,
                        });
                    } else if !udp || remote.1 != 0 {
                        connections.push(ConnectionEntry {
                            protocol: proto.to_string(),
                            local_addr: local.0,
                            local_port: local.1,
                            remote_addr: remote.0,
                            remote_port: remote.1,
                            state: state.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(NetworkInfo { listening, connections })
}

fn process_socket_inodes(pid: i32) -> HashSet<u64> {
    let mut inodes = HashSet::new();
    let fd_dir = format!("/proc/{}/fd", pid);
    if let Ok(entries) = std::fs::read_dir(&fd_dir) {
        for entry in entries.flatten() {
            if let Ok(target) = std::fs::read_link(entry.path()) {
                let s = target.to_string_lossy();
                if let Some(inode_str) = s.strip_prefix("socket:[").and_then(|s| s.strip_suffix(']')) {
                    if let Ok(inode) = inode_str.parse::<u64>() {
                        inodes.insert(inode);
                    }
                }
            }
        }
    }
    inodes
}

/// Parse hex address:port like "0100007F:1F40" into ("127.0.0.1", 8000).
fn parse_addr(field: &str, is_v6: bool) -> (String, u16) {
    let parts: Vec<&str> = field.splitn(2, ':').collect();
    if parts.len() != 2 {
        return ("?".to_string(), 0);
    }
    let port = u16::from_str_radix(parts[1], 16).unwrap_or(0);
    let addr_hex = parts[0];

    let addr = if is_v6 {
        // Four 32-bit little-endian words
        let bytes: Vec<u32> = addr_hex
            .as_bytes()
            .chunks(8)
            .filter_map(|c| {
                let s = std::str::from_utf8(c).ok()?;
                u32::from_str_radix(s, 16).ok()
            })
            .collect();
        if bytes.len() == 4 {
            let b: Vec<u8> = bytes
                .iter()
                .flat_map(|w| w.to_le_bytes())
                .collect();
            let arr: [u8; 16] = b.try_into().unwrap_or([0; 16]);
            Ipv6Addr::from(arr).to_string()
        } else {
            addr_hex.to_string()
        }
    } else {
        if let Ok(n) = u32::from_str_radix(addr_hex, 16) {
            let ip = Ipv4Addr::from(n.to_le_bytes());
            ip.to_string()
        } else {
            addr_hex.to_string()
        }
    };

    (addr, port)
}

fn tcp_state(state: u8) -> &'static str {
    match state {
        0x01 => "ESTABLISHED",
        0x02 => "SYN_SENT",
        0x03 => "SYN_RECV",
        0x04 => "FIN_WAIT1",
        0x05 => "FIN_WAIT2",
        0x06 => "TIME_WAIT",
        0x07 => "CLOSE",
        0x08 => "CLOSE_WAIT",
        0x09 => "LAST_ACK",
        0x0A => "LISTEN",
        0x0B => "CLOSING",
        _ => "UNKNOWN",
    }
}

