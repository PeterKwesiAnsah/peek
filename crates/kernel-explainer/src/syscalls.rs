// Syscall name → plain English; x86_64 number → name for /proc/<pid>/syscall.
//
// This is a small curated set focused on the most common syscalls users are
// likely to see in `/proc/<pid>/syscall` or tracing output.

/// Returns the syscall name for x86_64 syscall number, or `None` if unknown.
#[cfg(target_arch = "x86_64")]
pub fn syscall_name_x86_64(num: u64) -> Option<&'static str> {
    Some(match num {
        0 => "read",
        1 => "write",
        2 => "open",
        3 => "close",
        9 => "mmap",
        11 => "munmap",
        23 => "nanosleep",
        59 => "execve",
        57 => "fork",
        58 => "vfork",
        61 => "wait4",
        202 => "futex",
        228 => "clone",
        232 => "epoll_wait",
        270 => "poll",
        293 => "accept4",
        288 => "accept",
        42 => "connect",
        44 => "sendto",
        45 => "recvfrom",
        46 => "sendmsg",
        47 => "recvmsg",
        257 => "openat",
        326 => "execveat",
        494 => "pselect6",
        525 => "io_uring_enter",
        40 => "sendfile",
        10 => "mprotect",
        12 => "brk",
        274 => "ppoll",
        _ => return None,
    })
}

#[cfg(not(target_arch = "x86_64"))]
pub fn syscall_name_x86_64(_num: u64) -> Option<&'static str> {
    None
}

pub fn syscall_description(name: &str) -> &'static str {
    match name {
        "epoll_wait" | "epoll_pwait" => "Waiting for I/O events (network or file activity)",
        "select" | "pselect6" => "Waiting for readiness on multiple file descriptors",
        "poll" | "ppoll" => "Polling file descriptors for events",
        "read" | "pread64" => "Reading data from a file or socket",
        "write" | "pwrite64" => "Writing data to a file or socket",
        "recvfrom" | "recvmsg" => "Receiving data from a socket",
        "sendto" | "sendmsg" => "Sending data to a socket",
        "accept" | "accept4" => "Accepting a new incoming network connection",
        "connect" => "Establishing a new outgoing network connection",
        "open" | "openat" => "Opening a file or device",
        "close" => "Closing a file descriptor",
        "futex" => "Synchronising threads (futex wait/wake)",
        "nanosleep" | "clock_nanosleep" => "Sleeping for a specified duration",
        "mmap" | "mmap2" => "Mapping files or anonymous memory into the process",
        "munmap" => "Unmapping a memory region",
        "clone" | "clone3" => "Creating a new thread or process",
        "fork" | "vfork" => "Creating a new process",
        "execve" | "execveat" => "Executing a new program image",
        "wait4" | "waitid" => "Waiting for child process status changes",
        "sendfile" => "Transferring file data directly between descriptors",
        "io_uring_enter" => "Submitting or waiting for async I/O via io_uring",
        "brk" | "mprotect" => "Managing the process heap or memory protections",
        _ => "System call",
    }
}
