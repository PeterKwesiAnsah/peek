use crate::ProcessNode;
use procfs::process::all_processes;
use std::collections::HashMap;

pub fn build_tree(root_pid: i32) -> anyhow::Result<ProcessNode> {
    // Collect all processes into a flat map
    let mut nodes: HashMap<i32, ProcessNode> = HashMap::new();
    let mut children_map: HashMap<i32, Vec<i32>> = HashMap::new();

    for proc in all_processes()?.flatten() {
        let pid = proc.pid;
        let stat = match proc.stat() {
            Ok(s) => s,
            Err(_) => continue,
        };
        let status = proc.status().ok();
        let uid = status.as_ref().map(|s| s.ruid).unwrap_or(0);

        let statm = proc.statm().ok();
        let rss_kb = statm.map(|m| m.resident * 4).unwrap_or(0);

        nodes.insert(
            pid,
            ProcessNode {
                pid,
                name: stat.comm.clone(),
                uid,
                rss_kb,
                children: Vec::new(),
            },
        );
        children_map.entry(stat.ppid).or_default().push(pid);
    }

    // Build tree recursively starting from root_pid
    fn attach_children(
        pid: i32,
        nodes: &mut HashMap<i32, ProcessNode>,
        children_map: &HashMap<i32, Vec<i32>>,
        depth: usize,
    ) -> ProcessNode {
        let mut node = nodes.remove(&pid).unwrap_or(ProcessNode {
            pid,
            name: "?".to_string(),
            uid: 0,
            rss_kb: 0,
            children: Vec::new(),
        });
        if depth < 10 {
            if let Some(child_pids) = children_map.get(&pid) {
                let mut child_pids = child_pids.clone();
                child_pids.sort();
                for cpid in child_pids {
                    if cpid != pid {
                        let child = attach_children(cpid, nodes, children_map, depth + 1);
                        node.children.push(child);
                    }
                }
            }
        }
        node
    }

    Ok(attach_children(root_pid, &mut nodes, &children_map, 0))
}
