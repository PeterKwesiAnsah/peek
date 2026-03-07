use network_inspector::{resolver, tcp, unix};

#[test]
fn inodes_using_port_returns_map() {
    let map = tcp::inodes_using_port(0);
    // Port 0 is invalid for lookup; map may be empty or contain entries from parsing
    for (local, remote, state) in map.values() {
        assert!(!local.is_empty() || !remote.is_empty());
        assert!(!state.is_empty());
    }
}

#[test]
fn collect_network_for_current_process_is_resilient() {
    let pid = std::process::id() as i32;
    let result = tcp::collect_network(pid);
    if let Ok(info) = &result {
        let _ = (&info.listening_tcp, &info.listening_udp, &info.connections);
    }
}

#[test]
fn unix_sockets_list_is_resilient() {
    let pid = std::process::id() as i32;
    let list = unix::list_unix_sockets(pid);
    for entry in &list {
        let _ = entry.path.as_str();
    }
}

#[test]
fn resolver_is_resilient() {
    let result = resolver::resolve("127.0.0.1:80");
    // May be Some(hostname) or None depending on DNS; must not panic
    if let Some(s) = &result {
        assert!(!s.is_empty());
    }
}
