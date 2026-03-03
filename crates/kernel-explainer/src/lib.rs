//! Converts raw kernel values into human-readable explanations.
//!
//! State characters, scheduler policy/priority, capability bitmasks, OOM score, namespaces,
//! and well-known binary names.

pub mod capabilities;
pub mod namespaces;
pub mod oom;
pub mod scheduler;
pub mod signals;
pub mod states;
pub mod syscalls;
pub mod well_known;
