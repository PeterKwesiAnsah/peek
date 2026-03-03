// Systemd unit detection from cgroup.
//
// Both cgroups v1 and v2 encode the unit name in the cgroup path.

/// Try to infer the systemd unit managing `pid`, if any, by inspecting its
/// cgroup membership.
pub fn detect_systemd_unit(pid: i32) -> Option<String> {
    let cgroup = std::fs::read_to_string(format!("/proc/{}/cgroup", pid)).ok()?;
    for line in cgroup.lines() {
        // cgroup lines are "hier-id:controllers:path"
        let path = line.splitn(3, ':').nth(2)?;
        // Extract the leaf component
        let leaf = path.split('/').next_back()?;
        if leaf.ends_with(".service") || leaf.ends_with(".scope") {
            return Some(leaf.to_string());
        }
    }
    None
}
