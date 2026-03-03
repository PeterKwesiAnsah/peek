// peekd daemon: main entry, wires watcher + ring_store + alert + socket. Plan: main, watcher, ring_store, alert, socket.

mod alert;
mod ring_store;
mod socket;
mod watcher;

#[cfg(unix)]
pub const SOCKET_PATH: &str = "/run/peekd/peekd.sock";

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
    use alert::{load_config_into, AlertEngine};
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("peekd starting (socket: {})", SOCKET_PATH);

    if let Some(dir) = std::path::Path::new(SOCKET_PATH).parent() {
        std::fs::create_dir_all(dir)?;
    }
    let _ = std::fs::remove_file(SOCKET_PATH);

    let history = ring_store::new_history();
    let mut watched_vec: Vec<i32> = Vec::new();
    let mut engine = AlertEngine::new();
    if let Err(e) = load_config_into(&mut engine, &mut watched_vec) {
        tracing::warn!("failed to load alerts config: {}", e);
    }
    let watched: watcher::WatchedPids = Arc::new(Mutex::new(watched_vec));
    let alerts: watcher::AlertEng = Arc::new(Mutex::new(engine));

    watcher::run(history.clone(), watched.clone(), alerts.clone());

    socket::run_listener(SOCKET_PATH, history, watched, alerts).await
}
