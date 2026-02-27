use crate::OpenFile;

pub fn collect_files(pid: i32) -> anyhow::Result<Vec<OpenFile>> {
    let fd_dir = format!("/proc/{}/fd", pid);
    let mut files = Vec::new();

    let entries = std::fs::read_dir(&fd_dir)?;
    let mut pairs: Vec<(u32, std::path::PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let fd: u32 = e.file_name().to_string_lossy().parse().ok()?;
            Some((fd, e.path()))
        })
        .collect();
    pairs.sort_by_key(|(fd, _)| *fd);

    for (fd, path) in pairs {
        let (fd_type, description) = resolve_fd(pid, fd, &path);
        files.push(OpenFile { fd, fd_type, description });
    }

    Ok(files)
}

pub fn count_fds(pid: i32) -> anyhow::Result<usize> {
    let fd_dir = format!("/proc/{}/fd", pid);
    let count = std::fs::read_dir(fd_dir)?.count();
    Ok(count)
}

fn resolve_fd(pid: i32, fd: u32, fd_path: &std::path::Path) -> (String, String) {
    let target = match std::fs::read_link(fd_path) {
        Ok(t) => t.to_string_lossy().into_owned(),
        Err(_) => return ("unknown".to_string(), "?".to_string()),
    };

    if target.starts_with('/') {
        // Regular file or device
        let fd_type = if target.starts_with("/dev/") {
            "device"
        } else {
            "file"
        };
        // Try to get access mode from fdinfo
        let mode = read_fdinfo_mode(pid, fd);
        return (fd_type.to_string(), format!("{}{}", target, mode));
    }

    if let Some(inode) = target.strip_prefix("socket:[").and_then(|s| s.strip_suffix(']')) {
        return ("socket".to_string(), format!("socket inode {}", inode));
    }

    if let Some(inode) = target.strip_prefix("pipe:[").and_then(|s| s.strip_suffix(']')) {
        return ("pipe".to_string(), format!("pipe:[{}]", inode));
    }

    if target.starts_with("anon_inode:") {
        let kind = target.trim_start_matches("anon_inode:");
        let desc = match kind {
            "eventfd" => "eventfd (async event notification)".to_string(),
            "eventpoll" => "epoll instance".to_string(),
            "timerfd" => "timerfd (timer)".to_string(),
            "signalfd" => "signalfd (signal handler)".to_string(),
            _ => format!("anon_inode:{}", kind),
        };
        return ("anon_inode".to_string(), desc);
    }

    ("other".to_string(), target)
}

fn read_fdinfo_mode(pid: i32, fd: u32) -> String {
    let path = format!("/proc/{}/fdinfo/{}", pid, fd);
    if let Ok(raw) = std::fs::read_to_string(path) {
        for line in raw.lines() {
            if let Some(val) = line.strip_prefix("flags:\t") {
                let flags = u32::from_str_radix(val.trim(), 8).unwrap_or(0);
                let mode = flags & 0b11;
                return match mode {
                    0 => " (read-only)".to_string(),
                    1 => " (write-only)".to_string(),
                    2 => " (read-write)".to_string(),
                    _ => "".to_string(),
                };
            }
        }
    }
    String::new()
}

