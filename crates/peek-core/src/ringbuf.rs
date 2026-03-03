// Thin re-exports of the generic ring buffer and helpers that now live in the
// `resource-sampler` crate.

pub use resource_sampler::ring_buffer::{detect_fd_leak, ResourceSample, RingBuf};
