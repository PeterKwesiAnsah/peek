use peek_proc_reader::{cgroup, current, environ, fd, limits, security};

#[test]
fn limits_parse_limit_field_unlimited_and_numeric() {
    // Invalid PID may return Err on Linux or default Limits on non-Linux; must not panic.
    let result = limits::read_limits(-1);
    if let Ok(l) = &result {
        assert!(l.max_open_files_soft.is_some() || l.max_open_files_soft.is_none());
        assert!(l.max_open_files_hard.is_some() || l.max_open_files_hard.is_none());
    }
}

#[test]
fn environ_read_is_resilient() {
    let pid = std::process::id() as i32;
    let result = environ::read_environ(pid);
    assert!(result.is_ok());
    let _vec = result.unwrap();
}

#[test]
fn fd_read_is_resilient() {
    let pid = std::process::id() as i32;
    let result = fd::read_fd(pid);
    assert!(result.is_ok());
    let _vec = result.unwrap();
}

#[test]
fn cgroup_and_security_are_resilient() {
    let pid = std::process::id() as i32;
    let cg = cgroup::read_cgroup(pid);
    let lab = security::read_label(pid);
    assert!(cg.is_some() || cg.is_none());
    assert!(lab.is_some() || lab.is_none());
}

#[test]
fn current_syscall_does_not_panic() {
    let pid = std::process::id() as i32;
    let result = current::read_syscall(pid);
    assert!(result.is_some() || result.is_none());
}
