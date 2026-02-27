/// Integration and fixture-based tests for peek-core.
///
/// Fixture files live in tests/fixtures/ and mirror the format of real
/// /proc entries. Tests that require a real process use std::process::id()
/// to inspect the current process — this always works in CI.
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

// ─── Env parsing from fixture ─────────────────────────────────────────────────

#[test]
fn parse_environ_fixture() {
    let fixture = std::fs::read(fixtures_dir().join("environ")).unwrap();
    let mut vars = Vec::new();

    const SECRET_PATTERNS: &[&str] = &[
        "PASSWORD", "PASSWD", "SECRET", "TOKEN", "API_KEY", "APIKEY",
        "AUTH", "CREDENTIAL", "PRIVATE_KEY", "ACCESS_KEY", "AWS_SECRET",
        "DATABASE_URL", "DB_URL", "REDIS_URL", "MONGO_URL", "DSN",
    ];

    fn is_secret(key: &str) -> bool {
        let upper = key.to_uppercase();
        SECRET_PATTERNS.iter().any(|p| upper.contains(p))
    }

    // /proc/<pid>/environ uses null-separated entries; fixture may use null or newline
    let sep = if fixture.contains(&0) { 0u8 } else { b'\n' };
    for entry in fixture.split(move |&b| b == sep) {
        if entry.is_empty() { continue; }
        let s = String::from_utf8_lossy(entry);
        if let Some(eq) = s.find('=') {
            let key = s[..eq].to_string();
            let value = s[eq + 1..].to_string();
            let redacted = is_secret(&key);
            vars.push((key, value, redacted));
        }
    }

    assert!(!vars.is_empty(), "should parse at least one env var");

    let db = vars.iter().find(|(k, _, _)| k == "DATABASE_URL");
    assert!(db.is_some(), "DATABASE_URL should be present");
    assert!(db.unwrap().2, "DATABASE_URL should be redacted");

    let aws = vars.iter().find(|(k, _, _)| k == "AWS_ACCESS_KEY_ID");
    assert!(aws.is_some(), "AWS_ACCESS_KEY_ID should be present");
    assert!(aws.unwrap().2, "AWS_ACCESS_KEY_ID should be redacted");

    let port = vars.iter().find(|(k, _, _)| k == "PORT");
    assert!(port.is_some(), "PORT should be present");
    assert!(!port.unwrap().2, "PORT should NOT be redacted");
}

// ─── Status fixture parsing ───────────────────────────────────────────────────

#[test]
fn parse_status_fixture() {
    let raw = std::fs::read_to_string(fixtures_dir().join("status")).unwrap();

    let uid: u32 = raw.lines()
        .find(|l| l.starts_with("Uid:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(uid, 33, "ruid should be 33 (www-data)");

    let threads: i32 = raw.lines()
        .find(|l| l.starts_with("Threads:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(threads, 4);

    let seccomp: u32 = raw.lines()
        .find(|l| l.starts_with("Seccomp:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(seccomp, 2, "seccomp filter mode");

    let vol: u64 = raw.lines()
        .find(|l| l.starts_with("voluntary_ctxt_switches:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(vol, 1523);
}

// ─── stat fixture parsing ─────────────────────────────────────────────────────

#[test]
fn parse_stat_fixture() {
    let raw = std::fs::read_to_string(fixtures_dir().join("stat")).unwrap();

    // PID and comm
    let pid: i32 = raw.split_whitespace().next().unwrap().parse().unwrap();
    assert_eq!(pid, 1234);

    let comm_start = raw.find('(').unwrap() + 1;
    let comm_end = raw.rfind(')').unwrap();
    let comm = &raw[comm_start..comm_end];
    assert_eq!(comm, "nginx");

    // Fields after ')'
    let after = &raw[comm_end + 2..];
    let fields: Vec<&str> = after.split_whitespace().collect();

    let state_char = fields[0].chars().next().unwrap();
    assert_eq!(state_char, 'S');

    let ppid: i32 = fields[1].parse().unwrap();
    assert_eq!(ppid, 1);

    let num_threads: i32 = fields[17].parse().unwrap();
    assert_eq!(num_threads, 4);
}

// ─── statm fixture parsing ────────────────────────────────────────────────────

#[test]
fn parse_statm_fixture() {
    let raw = std::fs::read_to_string(fixtures_dir().join("statm")).unwrap();
    let fields: Vec<u64> = raw.split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    assert!(fields.len() >= 2);
    let size_pages = fields[0];     // total VM pages
    let resident_pages = fields[1]; // resident pages

    let vm_size_kb = size_pages * 4;
    let rss_kb = resident_pages * 4;

    assert_eq!(vm_size_kb, 102400);
    assert_eq!(rss_kb, 18432);
}

// ─── cgroup fixture ───────────────────────────────────────────────────────────

#[test]
fn parse_cgroup_fixture() {
    let raw = std::fs::read_to_string(fixtures_dir().join("cgroup")).unwrap();
    let unit: Option<String> = raw.lines().find_map(|line| {
        let path = line.splitn(3, ':').nth(2)?;
        let leaf = path.split('/').last()?;
        if leaf.ends_with(".service") || leaf.ends_with(".scope") {
            Some(leaf.to_string())
        } else {
            None
        }
    });
    assert_eq!(unit, Some("nginx.service".to_string()));
}

// ─── Live process test (self) ─────────────────────────────────────────────────

#[test]
fn collect_self_process() {
    let pid = std::process::id() as i32;
    let info = peek_core::collect(pid).expect("should collect own process info");

    assert_eq!(info.pid, pid);
    assert!(!info.name.is_empty(), "process name should not be empty");
    assert!(info.rss_kb > 0, "RSS should be non-zero");
    assert!(info.vm_size_kb >= info.rss_kb, "VSZ >= RSS");
    assert!(info.threads >= 1);
    assert!(info.ppid > 0);
}

#[test]
fn collect_extended_resources_self() {
    let pid = std::process::id() as i32;
    let opts = peek_core::CollectOptions {
        resources: true,
        ..Default::default()
    };
    let info = peek_core::collect_extended(pid, &opts)
        .expect("collect_extended should succeed");

    assert!(info.fd_count.unwrap_or(0) > 0, "should have at least some FDs");
}

#[test]
fn collect_extended_kernel_self() {
    let pid = std::process::id() as i32;
    let opts = peek_core::CollectOptions {
        kernel: true,
        ..Default::default()
    };
    let info = peek_core::collect_extended(pid, &opts).unwrap();
    let k = info.kernel.expect("kernel info should be present");
    assert!(!k.sched_policy.is_empty());
    // OOM score in valid range
    assert!(k.oom_score >= 0 && k.oom_score <= 1000);
}

#[test]
fn collect_extended_env_self() {
    let pid = std::process::id() as i32;
    let opts = peek_core::CollectOptions {
        env: true,
        ..Default::default()
    };
    let info = peek_core::collect_extended(pid, &opts).unwrap();
    let env = info.env_vars.expect("env vars should be collected");
    // PATH is set in virtually every process
    assert!(env.iter().any(|v| v.key == "PATH"), "PATH should be in environment");
}

#[test]
fn collect_nonexistent_pid_returns_not_found() {
    // PID 999999999 should not exist
    let result = peek_core::collect(999_999_999);
    assert!(
        matches!(result, Err(peek_core::PeekError::NotFound(_))),
        "expected NotFound error"
    );
}

// ─── Ring buffer tests ────────────────────────────────────────────────────────

#[test]
fn ring_buf_wraps_correctly() {
    use peek_core::ringbuf::RingBuf;
    let mut rb: RingBuf<u32> = RingBuf::new(4);
    for i in 0..8u32 { rb.push(i); }
    assert_eq!(rb.len(), 4);
    assert_eq!(rb.to_vec(), vec![4, 5, 6, 7]);
}

#[test]
fn fd_leak_detector_integration() {
    use peek_core::ringbuf::{detect_fd_leak, RingBuf, ResourceSample};

    let mut rb: RingBuf<ResourceSample> = RingBuf::new(20);

    // Stable FD count — no leak
    for _ in 0..12 {
        rb.push(ResourceSample { fd_count: 30, ..Default::default() });
    }
    assert!(detect_fd_leak(&rb, 8).is_none(), "stable FDs should not trigger");

    // Suddenly growing FD count
    for i in 0..10u64 {
        rb.push(ResourceSample { fd_count: 30 + i, ..Default::default() });
    }
    assert!(detect_fd_leak(&rb, 8).is_some(), "growing FDs should trigger leak warning");
}

