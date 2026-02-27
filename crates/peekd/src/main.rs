mod alerts;

#[cfg(unix)]
pub const SOCKET_PATH: &str = "/run/peekd/peekd.sock";

#[cfg(unix)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Sample {
    pub ts: chrono::DateTime<chrono::Local>,
    pub rss_kb: u64,
    pub vm_size_kb: u64,
    pub threads: i32,
    pub cpu_percent: Option<f64>,
    pub fd_count: Option<usize>,
}

#[cfg(unix)]
type History     = std::sync::Arc<std::sync::Mutex<std::collections::HashMap<i32, Vec<Sample>>>>;
#[cfg(unix)]
type WatchedPids = std::sync::Arc<std::sync::Mutex<Vec<i32>>>;
#[cfg(unix)]
type AlertEng    = std::sync::Arc<std::sync::Mutex<alerts::AlertEngine>>;

// ─── Entry point ─────────────────────────────────────────────────────────────

#[cfg(not(unix))]
fn main() {
    eprintln!("peekd is only supported on Linux/Unix.");
    std::process::exit(1);
}

#[cfg(unix)]
fn main() {
    if let Err(e) = run() {
        eprintln!("peekd: {:#}", e);
        std::process::exit(1);
    }
}

#[cfg(unix)]
fn run() -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(daemon_main())
}

#[cfg(unix)]
async fn daemon_main() -> anyhow::Result<()> {
    use alerts::AlertEngine;
    use chrono::Local;
    use peek_core::collect;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tokio::net::UnixListener;
    use tokio::time::{interval, Duration};
    use tracing_subscriber::EnvFilter;

    const RING_SIZE: usize = 300; // 5 min at 1 s

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("peekd starting (socket: {})", SOCKET_PATH);

    if let Some(dir) = std::path::Path::new(SOCKET_PATH).parent() {
        std::fs::create_dir_all(dir)?;
    }
    let _ = std::fs::remove_file(SOCKET_PATH);

    let history:  History     = Arc::new(Mutex::new(HashMap::new()));
    let watched:  WatchedPids = Arc::new(Mutex::new(Vec::new()));
    let alerts:   AlertEng    = Arc::new(Mutex::new(AlertEngine::new()));

    // ── Sampling task ──────────────────────────────────────────────────────
    {
        let history = Arc::clone(&history);
        let watched = Arc::clone(&watched);
        let alerts  = Arc::clone(&alerts);
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));
            loop {
                ticker.tick().await;
                let pids: Vec<i32> = watched.lock().unwrap().clone();
                for pid in pids {
                    match collect(pid) {
                        Ok(info) => {
                            let sample = Sample {
                                ts:          Local::now(),
                                rss_kb:      info.rss_kb,
                                vm_size_kb:  info.vm_size_kb,
                                threads:     info.threads,
                                cpu_percent: info.cpu_percent,
                                fd_count:    info.fd_count,
                            };
                            {
                                let mut h = history.lock().unwrap();
                                let ring = h.entry(pid).or_insert_with(Vec::new);
                                ring.push(sample);
                                if ring.len() > RING_SIZE { ring.remove(0); }
                            }
                            // Evaluate alert rules
                            let fired = alerts.lock().unwrap().evaluate(&info);
                            for event in fired {
                                tracing::warn!(
                                    "alert fired: rule={} pid={} {}={:.2} threshold={:.2}",
                                    event.rule_id, event.pid, event.metric, event.value, event.threshold
                                );
                            }
                        }
                        Err(_) => {
                            tracing::info!("pid {} gone, removing", pid);
                            watched.lock().unwrap().retain(|&p| p != pid);
                            history.lock().unwrap().remove(&pid);
                            alerts.lock().unwrap().remove_rules_for_pid(pid);
                        }
                    }
                }
            }
        });
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    tracing::info!("peekd ready");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let history = Arc::clone(&history);
                let watched = Arc::clone(&watched);
                let alerts  = Arc::clone(&alerts);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, history, watched, alerts).await {
                        tracing::warn!("client error: {}", e);
                    }
                });
            }
            Err(e) => tracing::error!("accept error: {}", e),
        }
    }
}

// ─── Client handler ───────────────────────────────────────────────────────────

#[cfg(unix)]
async fn handle_client(
    stream: tokio::net::UnixStream,
    history: History,
    watched: WatchedPids,
    alerts: AlertEng,
) -> anyhow::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let (r, mut w) = tokio::io::split(stream);
    let mut reader = BufReader::new(r);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let response = match serde_json::from_str::<serde_json::Value>(line.trim()) {
        Err(e) => json_err(format!("invalid JSON: {}", e)),
        Ok(req) => dispatch(req, &history, &watched, &alerts),
    };

    let mut out = serde_json::to_string(&response)?;
    out.push('\n');
    w.write_all(out.as_bytes()).await?;
    Ok(())
}

// ─── Dispatch ─────────────────────────────────────────────────────────────────

#[cfg(unix)]
fn dispatch(
    req: serde_json::Value,
    history: &History,
    watched: &WatchedPids,
    alerts: &AlertEng,
) -> serde_json::Value {
    let action = req["action"].as_str().unwrap_or("").to_string();

    match action.as_str() {
        // ── Process watching ──────────────────────────────────────────────
        "watch" => {
            let pid = match req["pid"].as_i64() {
                Some(p) => p as i32,
                None => return json_err("'watch' requires pid"),
            };
            let mut w = watched.lock().unwrap();
            if !w.contains(&pid) { w.push(pid); }
            json_ok(serde_json::json!({ "watching": pid }))
        }
        "unwatch" => {
            let pid = match req["pid"].as_i64() {
                Some(p) => p as i32,
                None => return json_err("'unwatch' requires pid"),
            };
            watched.lock().unwrap().retain(|&p| p != pid);
            history.lock().unwrap().remove(&pid);
            alerts.lock().unwrap().remove_rules_for_pid(pid);
            json_ok(serde_json::json!({ "unwatched": pid }))
        }
        "list" => {
            let w = watched.lock().unwrap().clone();
            json_ok(serde_json::json!({ "watching": w }))
        }

        // ── History ───────────────────────────────────────────────────────
        "history" => {
            let pid = match req["pid"].as_i64() {
                Some(p) => p as i32,
                None => return json_err("'history' requires pid"),
            };
            let h = history.lock().unwrap();
            match h.get(&pid) {
                Some(samples) => {
                    let j = serde_json::to_value(samples).unwrap_or(serde_json::Value::Null);
                    json_ok(j)
                }
                None => json_err(format!("no history for pid {} — add it with 'watch' first", pid)),
            }
        }

        // ── Alert management ──────────────────────────────────────────────
        "alert_add" => {
            let add_req: alerts::AlertAddRequest = match serde_json::from_value(req.clone()) {
                Ok(r) => r,
                Err(e) => return json_err(format!("invalid alert rule: {}", e)),
            };
            let id = format!("rule-{}", chrono::Local::now().timestamp_millis());
            let rule = add_req.into_rule(id.clone());
            alerts.lock().unwrap().add_rule(rule);

            // Auto-watch this PID
            let pid_to_watch = req["pid"].as_i64().unwrap_or(0) as i32;
            if pid_to_watch > 0 {
                let mut w = watched.lock().unwrap();
                if !w.contains(&pid_to_watch) { w.push(pid_to_watch); }
            }

            json_ok(serde_json::json!({ "rule_id": id }))
        }
        "alert_list" => {
            let eng = alerts.lock().unwrap();
            let rules: Vec<serde_json::Value> = eng.rules.iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "pid": r.pid,
                    "metric": r.metric.to_string(),
                    "threshold": r.threshold,
                    "cooldown_secs": r.cooldown_secs,
                })
            }).collect();
            json_ok(serde_json::json!({ "rules": rules }))
        }

        // ── Meta ──────────────────────────────────────────────────────────
        "ping" => json_ok(serde_json::json!({
            "pong": true,
            "version": env!("CARGO_PKG_VERSION"),
            "watching": watched.lock().unwrap().len(),
        })),

        other => json_err(format!("unknown action '{}'", other)),
    }
}

#[cfg(unix)]
fn json_ok(data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "ok": true, "data": data })
}

#[cfg(unix)]
fn json_err(msg: impl Into<String>) -> serde_json::Value {
    serde_json::json!({ "ok": false, "message": msg.into() })
}
