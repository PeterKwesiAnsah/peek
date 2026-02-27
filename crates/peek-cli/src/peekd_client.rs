use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

pub const SOCKET_PATH: &str = "/run/peekd/peekd.sock";

#[derive(Debug, Serialize)]
struct Request<'a> {
    action: &'a str,
    pid: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub ok: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct HistorySample {
    pub ts: String,
    pub rss_kb: u64,
    pub vm_size_kb: u64,
    pub threads: i32,
    pub cpu_percent: Option<f64>,
}

fn send_request(action: &str, pid: Option<i32>) -> Result<Response> {
    let stream = UnixStream::connect(SOCKET_PATH)
        .with_context(|| format!("cannot connect to peekd at {SOCKET_PATH}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    let req = Request { action, pid };
    let mut json = serde_json::to_string(&req)?;
    json.push('\n');

    let mut writer = stream.try_clone()?;
    writer.write_all(json.as_bytes())?;

    let mut reader = BufReader::new(stream);
    let mut resp_line = String::new();
    reader.read_line(&mut resp_line)?;
    let resp: Response = serde_json::from_str(resp_line.trim())?;
    Ok(resp)
}

pub fn fetch_history(pid: i32) -> Result<Vec<HistorySample>> {
    let resp = send_request("history", Some(pid))?;
    if !resp.ok {
        anyhow::bail!("{}", resp.message.unwrap_or_else(|| "unknown peekd error".to_string()));
    }
    let data = resp.data.context("no data in response")?;
    let samples: Vec<HistorySample> = serde_json::from_value(data)?;
    Ok(samples)
}

pub fn ping() -> bool {
    send_request("ping", None).map(|r| r.ok).unwrap_or(false)
}

pub fn register_watch(pid: i32) -> Result<()> {
    let resp = send_request("watch", Some(pid))?;
    if !resp.ok {
        anyhow::bail!("{}", resp.message.unwrap_or_default());
    }
    Ok(())
}

