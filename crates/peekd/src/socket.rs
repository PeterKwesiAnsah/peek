// Unix socket accept loop and request dispatch. Plan: socket.

#[cfg(unix)]
use crate::alert;
#[cfg(unix)]
use crate::ring_store;
#[cfg(unix)]
use crate::watcher::{AlertEng, WatchedPids};

#[cfg(unix)]
pub async fn run_listener(
    path: &str,
    history: ring_store::History,
    watched: WatchedPids,
    alerts: AlertEng,
) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    use tokio::net::UnixListener;

    let listener = UnixListener::bind(path)?;
    // Allow unprivileged users (e.g. peek run without sudo) to connect.
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o666);
        let _ = std::fs::set_permissions(path, perms);
    }
    tracing::info!("peekd ready");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let history = std::sync::Arc::clone(&history);
                let watched = std::sync::Arc::clone(&watched);
                let alerts = std::sync::Arc::clone(&alerts);
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

#[cfg(unix)]
async fn handle_client(
    stream: tokio::net::UnixStream,
    history: ring_store::History,
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

#[cfg(unix)]
fn dispatch(
    req: serde_json::Value,
    history: &ring_store::History,
    watched: &WatchedPids,
    alerts: &AlertEng,
) -> serde_json::Value {
    let action = req["action"].as_str().unwrap_or("").to_string();

    match action.as_str() {
        "watch" => {
            let pid = match req["pid"].as_i64() {
                Some(p) => p as i32,
                None => return json_err("'watch' requires pid"),
            };
            let mut w = watched.lock().unwrap();
            if !w.contains(&pid) {
                w.push(pid);
            }
            json_ok(serde_json::json!({ "watching": pid }))
        }
        "unwatch" => {
            let pid = match req["pid"].as_i64() {
                Some(p) => p as i32,
                None => return json_err("'unwatch' requires pid"),
            };
            watched.lock().unwrap().retain(|&p| p != pid);
            ring_store::remove_pid(history, pid);
            alerts.lock().unwrap().remove_rules_for_pid(pid);
            json_ok(serde_json::json!({ "unwatched": pid }))
        }
        "list" => {
            let w = watched.lock().unwrap().clone();
            json_ok(serde_json::json!({ "watching": w }))
        }
        "history" => {
            let pid = match req["pid"].as_i64() {
                Some(p) => p as i32,
                None => return json_err("'history' requires pid"),
            };
            // Lazy-load from disk if not present in memory.
            {
                let h = history.lock().unwrap();
                if !h.contains_key(&pid) {
                    drop(h);
                    crate::ring_store::load_from_disk(history, pid);
                }
            }
            let h = history.lock().unwrap();
            match h.get(&pid) {
                Some(samples) => {
                    let j = serde_json::to_value(samples).unwrap_or(serde_json::Value::Null);
                    json_ok(j)
                }
                None => json_err(format!(
                    "no history for pid {} — add it with 'watch' first",
                    pid
                )),
            }
        }
        "alert_add" => {
            let add_req: alert::AlertAddRequest = match serde_json::from_value(req.clone()) {
                Ok(r) => r,
                Err(e) => return json_err(format!("invalid alert rule: {}", e)),
            };
            let id = format!("rule-{}", chrono::Local::now().timestamp_millis());
            let rule = add_req.into_rule(id.clone());
            alerts.lock().unwrap().add_rule(rule);
            let pid_to_watch = req["pid"].as_i64().unwrap_or(0) as i32;
            if pid_to_watch > 0 {
                let mut w = watched.lock().unwrap();
                if !w.contains(&pid_to_watch) {
                    w.push(pid_to_watch);
                }
            }
            json_ok(serde_json::json!({ "rule_id": id }))
        }
        "alert_list" => {
            let eng = alerts.lock().unwrap();
            let rules: Vec<serde_json::Value> = eng
                .rules
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "pid": r.pid,
                        "metric": r.metric.to_string(),
                        "threshold": r.threshold,
                        "cooldown_secs": r.cooldown_secs,
                    })
                })
                .collect();
            json_ok(serde_json::json!({ "rules": rules }))
        }
        "alert_remove" => {
            let rule_id = match req["rule_id"].as_str() {
                Some(s) => s.to_string(),
                None => return json_err("'alert_remove' requires rule_id"),
            };
            let removed = alerts.lock().unwrap().remove_rule(&rule_id);
            json_ok(serde_json::json!({ "removed": removed, "rule_id": rule_id }))
        }
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
