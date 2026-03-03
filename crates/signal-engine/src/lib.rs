//! Signal impact analysis and systemd unit detection.
//!
//! Counts connections, children, and file locks; suggests graceful vs forceful signals
//! and detects the owning systemd unit for a PID.

pub mod impact;
pub mod signals;
pub mod systemd;
