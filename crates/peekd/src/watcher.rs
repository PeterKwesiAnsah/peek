// Sampling loop: poll watched PIDs every second, store samples, evaluate alerts. Plan: watcher.

#[cfg(unix)]
pub type WatchedPids = std::sync::Arc<std::sync::Mutex<Vec<i32>>>;

#[cfg(unix)]
pub type AlertEng = std::sync::Arc<std::sync::Mutex<crate::alert::AlertEngine>>;

#[cfg(unix)]
pub fn run(history: crate::ring_store::History, watched: WatchedPids, alerts: AlertEng) {
    use chrono::Local;
    use peek_core::collect;
    use tokio::time::{interval, Duration};

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(1));
        loop {
            ticker.tick().await;
            let pids: Vec<i32> = watched.lock().unwrap().clone();
            for pid in pids {
                match collect(pid) {
                    Ok(info) => {
                        let sample = crate::ring_store::Sample {
                            ts: Local::now(),
                            rss_kb: info.rss_kb,
                            vm_size_kb: info.vm_size_kb,
                            threads: info.threads,
                            cpu_percent: info.cpu_percent,
                            fd_count: info.fd_count,
                        };
                        crate::ring_store::push_sample(&history, pid, sample);
                        let fired = alerts.lock().unwrap().evaluate(&info);
                        for event in fired {
                            tracing::warn!(
                                "alert fired: rule={} pid={} {}={:.2} threshold={:.2}",
                                event.rule_id,
                                event.pid,
                                event.metric,
                                event.value,
                                event.threshold
                            );
                        }
                    }
                    Err(_) => {
                        tracing::info!("pid {} gone, removing", pid);
                        watched.lock().unwrap().retain(|&p| p != pid);
                        crate::ring_store::remove_pid(&history, pid);
                        alerts.lock().unwrap().remove_rules_for_pid(pid);
                    }
                }
            }
        }
    });
}
