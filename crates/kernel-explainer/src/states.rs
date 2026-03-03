// Process state char → human description (R, S, D, Z, T, t, I, ...).
//
// Mirrors the mapping previously implemented in `peek-core::proc::linux`.
pub fn state_description(c: char) -> String {
    match c {
        'R' => "Running".to_string(),
        'S' => "Sleeping (interruptible)".to_string(),
        'D' => "Uninterruptible sleep (disk/NFS wait)".to_string(),
        'Z' => "Zombie".to_string(),
        'T' => "Stopped (signal)".to_string(),
        't' => "Tracing stop".to_string(),
        'W' => "Paging".to_string(),
        'X' | 'x' => "Dead".to_string(),
        'I' => "Idle".to_string(),
        other => other.to_string(),
    }
}
