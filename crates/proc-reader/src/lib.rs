//! Low-level parsing of `/proc/<PID>/*` and sysfs.
//!
//! Reads status, stat, fd, environ, cgroup, limits, and related files into raw structs.
//! Does not depend on peek-core; used by peek-core and peekd.

pub mod cgroup;
pub mod current;
pub mod environ;
pub mod fd;
pub mod limits;
pub mod maps;
pub mod net;
pub mod security;
pub mod stat;
pub mod status;
