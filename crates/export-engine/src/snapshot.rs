// Wrapper for a process snapshot with capture metadata (plan.md § ProcessSnapshot).

use chrono::{DateTime, Utc};
use peek_core::ProcessInfo;
use serde::Serialize;

/// A process snapshot with capture time and peek version for exports.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessSnapshot {
    /// When the snapshot was captured (UTC).
    pub captured_at: DateTime<Utc>,
    /// peek version that produced this snapshot (e.g. from CARGO_PKG_VERSION).
    pub peek_version: String,
    /// The process data.
    pub process: ProcessInfo,
}
