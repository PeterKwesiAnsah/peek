use crate::{ConnectionEntry, NetworkInfo, SocketEntry, UnixSocketEntry};
use network_inspector::tcp;
use network_inspector::unix;

pub fn collect_network(pid: i32) -> anyhow::Result<NetworkInfo> {
    let raw = tcp::collect_network(pid)?;
    let unix_list = unix::list_unix_sockets(pid);

    let listening_tcp = raw
        .listening_tcp
        .into_iter()
        .map(|s| SocketEntry {
            protocol: s.protocol,
            local_addr: s.local_addr,
            local_port: s.local_port,
        })
        .collect();

    let listening_udp = raw
        .listening_udp
        .into_iter()
        .map(|s| SocketEntry {
            protocol: s.protocol,
            local_addr: s.local_addr,
            local_port: s.local_port,
        })
        .collect();

    let connections = raw
        .connections
        .into_iter()
        .map(|c| ConnectionEntry {
            protocol: c.protocol,
            local_addr: c.local_addr,
            local_port: c.local_port,
            remote_addr: c.remote_addr,
            remote_port: c.remote_port,
            state: c.state,
        })
        .collect();

    let unix_sockets = if unix_list.is_empty() {
        None
    } else {
        Some(
            unix_list
                .into_iter()
                .map(|u| UnixSocketEntry { path: u.path })
                .collect(),
        )
    };

    let (traffic_rx_bytes_per_sec, traffic_tx_bytes_per_sec) =
        if let Some((rx, tx)) = resource_sampler::net::sample_network_rate(pid) {
            (Some(rx), Some(tx))
        } else {
            (None, None)
        };

    Ok(NetworkInfo {
        listening_tcp,
        listening_udp,
        connections,
        unix_sockets,
        traffic_rx_bytes_per_sec,
        traffic_tx_bytes_per_sec,
    })
}
