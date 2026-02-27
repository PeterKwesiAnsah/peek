use crate::EnvVar;

const SECRET_PATTERNS: &[&str] = &[
    "PASSWORD",
    "PASSWD",
    "SECRET",
    "TOKEN",
    "API_KEY",
    "APIKEY",
    "AUTH",
    "CREDENTIAL",
    "PRIVATE_KEY",
    "ACCESS_KEY",
    "AWS_SECRET",
    "DATABASE_URL",
    "DB_URL",
    "REDIS_URL",
    "MONGO_URL",
    "DSN",
];

pub fn collect_env(pid: i32) -> anyhow::Result<Vec<EnvVar>> {
    let raw = std::fs::read(format!("/proc/{}/environ", pid))?;
    let mut vars = Vec::new();

    for entry in raw.split(|&b| b == 0) {
        if entry.is_empty() {
            continue;
        }
        let s = String::from_utf8_lossy(entry);
        if let Some(eq) = s.find('=') {
            let key = s[..eq].to_string();
            let value = s[eq + 1..].to_string();
            let redacted = is_secret(&key);
            vars.push(EnvVar {
                key,
                value: if redacted { "***".to_string() } else { value },
                redacted,
            });
        }
    }

    vars.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(vars)
}

fn is_secret(key: &str) -> bool {
    let upper = key.to_uppercase();
    SECRET_PATTERNS.iter().any(|p| upper.contains(p))
}

