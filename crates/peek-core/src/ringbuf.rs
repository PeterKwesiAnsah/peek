use std::collections::VecDeque;

/// Fixed-capacity ring buffer. Oldest entry is dropped when full.
#[derive(Debug, Clone)]
pub struct RingBuf<T: Clone> {
    data: VecDeque<T>,
    capacity: usize,
}

impl<T: Clone> RingBuf<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    pub fn push(&mut self, value: T) {
        if self.data.len() == self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn capacity(&self) -> usize { self.capacity }

    /// Returns a slice-like iterator over all values in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &T> { self.data.iter() }

    /// Collect into a Vec for use with ratatui Sparkline (which needs &[u64]).
    pub fn to_vec(&self) -> Vec<T> {
        self.data.iter().cloned().collect()
    }

    /// Returns the most recent N entries as a Vec.
    pub fn last_n(&self, n: usize) -> Vec<T> {
        self.data.iter().rev().take(n).rev().cloned().collect()
    }
}

// ─── Specialised helpers for resource metrics ─────────────────────────────────

#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceSample {
    pub cpu_pct_x10: u64,   // cpu_percent * 10 → stored as u64 for sparkline
    pub rss_kb: u64,
    pub fd_count: u64,
    pub thread_count: u64,
}

/// Detect potential FD leaks from a series of FD count samples.
///
/// Returns `Some((start, end, consecutive_increases))` if the last
/// `window` samples all show a monotonic increase.
pub fn detect_fd_leak(history: &RingBuf<ResourceSample>, window: usize) -> Option<(usize, usize, usize)> {
    let window = window.min(history.len());
    if window < 3 {
        return None;
    }
    // Oldest → newest for the last `window` samples
    let samples: Vec<u64> = history
        .last_n(window)
        .iter()
        .map(|s| s.fd_count)
        .collect();
    if samples.len() < 3 {
        return None;
    }

    let mut consecutive = 0usize;
    for i in 0..samples.len() - 1 {
        if samples[i + 1] > samples[i] {
            consecutive += 1;
        } else {
            break;
        }
    }
    if consecutive >= window - 1 {
        let start = samples.first().copied().unwrap_or(0) as usize;
        let end = samples.last().copied().unwrap_or(0) as usize;
        Some((start, end, consecutive))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buf_capacity() {
        let mut rb: RingBuf<u32> = RingBuf::new(3);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        rb.push(4);
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.to_vec(), vec![2, 3, 4]);
    }

    #[test]
    fn ring_buf_last_n() {
        let mut rb: RingBuf<u32> = RingBuf::new(10);
        for i in 0..7u32 { rb.push(i); }
        assert_eq!(rb.last_n(3), vec![4, 5, 6]);
    }

    #[test]
    fn fd_leak_detector_fires() {
        let mut rb: RingBuf<ResourceSample> = RingBuf::new(20);
        // Monotonically increasing FD counts
        for i in 10..20u64 {
            rb.push(ResourceSample { fd_count: i, ..Default::default() });
        }
        let result = detect_fd_leak(&rb, 8);
        assert!(result.is_some());
        let (start, end, n) = result.unwrap();
        assert!(end > start);
        assert!(n >= 7);
    }

    #[test]
    fn fd_leak_detector_silent_when_stable() {
        let mut rb: RingBuf<ResourceSample> = RingBuf::new(20);
        for _ in 0..10 {
            rb.push(ResourceSample { fd_count: 42, ..Default::default() });
        }
        assert!(detect_fd_leak(&rb, 8).is_none());
    }
}

