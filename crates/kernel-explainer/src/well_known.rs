// Well-known binary name → description (nginx, postgres, sshd, ...).
//
// This is intentionally small and opinionated; it can be extended over time via
// PRs without affecting the core architecture.

pub fn binary_description(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "nginx" => Some("Web server handling HTTP/HTTPS traffic"),
        "httpd" | "apache2" => Some("Apache HTTP server"),
        "postgres" | "postgresql" => Some("PostgreSQL database server"),
        "mysqld" | "mariadbd" => Some("MySQL/MariaDB database server"),
        "redis-server" | "redis" => Some("Redis in-memory data store"),
        "sshd" => Some("SSH daemon accepting remote shell connections"),
        "systemd" => Some("Init system and service manager (PID 1)"),
        "dockerd" => Some("Docker container daemon"),
        "containerd" => Some("Container runtime daemon (containerd)"),
        "kubelet" => Some("Kubernetes node agent (kubelet)"),
        "node" => Some("Node.js runtime process"),
        "python" | "python3" => Some("Python interpreter process"),
        "java" => Some("JVM process (Java application)"),
        "rsyslogd" => Some("System logging daemon (rsyslog)"),
        "journald" | "systemd-journald" => Some("systemd journal logging service"),
        "sshd-session" => Some("Interactive SSH session"),
        _ => None,
    }
}
