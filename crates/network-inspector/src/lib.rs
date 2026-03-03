//! Per-process network view: TCP/UDP/Unix sockets, listening state, connections, reverse DNS.
//!
//! Reads `/proc/net/*` and `/proc/<PID>/fd` to list sockets and optional traffic rates.

pub mod resolver;
pub mod tcp;
pub mod udp;
pub mod unix;
