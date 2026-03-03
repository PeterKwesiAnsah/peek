use crate::OpenFile;
use proc_reader::fd::{count_fds as reader_count_fds, read_fd, FdEntry};

pub fn collect_files(pid: i32) -> anyhow::Result<Vec<OpenFile>> {
    let entries: Vec<FdEntry> = read_fd(pid)?;
    let mut files = Vec::with_capacity(entries.len());

    for entry in entries {
        files.push(OpenFile {
            fd: entry.fd,
            fd_type: entry.fd_type,
            description: entry.description,
        });
    }

    Ok(files)
}

pub fn count_fds(pid: i32) -> anyhow::Result<usize> {
    reader_count_fds(pid)
}
