// Capability set → human descriptions.
//
// Mirrors the decoding logic previously implemented in
// `peek-core::proc::kernel::decode_caps`, but works on the numeric bitset
// directly instead of the raw hex string.

const CAP_NAMES: &[&str] = &[
    "CHOWN",
    "DAC_OVERRIDE",
    "DAC_READ_SEARCH",
    "FOWNER",
    "FSETID",
    "KILL",
    "SETGID",
    "SETUID",
    "SETPCAP",
    "LINUX_IMMUTABLE",
    "NET_BIND_SERVICE",
    "NET_BROADCAST",
    "NET_ADMIN",
    "NET_RAW",
    "IPC_LOCK",
    "IPC_OWNER",
    "SYS_MODULE",
    "SYS_RAWIO",
    "SYS_CHROOT",
    "SYS_PTRACE",
    "SYS_PACCT",
    "SYS_ADMIN",
    "SYS_BOOT",
    "SYS_NICE",
    "SYS_RESOURCE",
    "SYS_TIME",
    "SYS_TTY_CONFIG",
    "MKNOD",
    "LEASE",
    "AUDIT_WRITE",
    "AUDIT_CONTROL",
    "SETFCAP",
    "MAC_OVERRIDE",
    "MAC_ADMIN",
    "SYSLOG",
    "WAKE_ALARM",
    "BLOCK_SUSPEND",
    "AUDIT_READ",
    "PERFMON",
    "BPF",
    "CHECKPOINT_RESTORE",
];

fn caps_to_string(val: u64) -> String {
    if val == 0 {
        return "none".to_string();
    }

    let caps: Vec<&str> = CAP_NAMES
        .iter()
        .enumerate()
        .filter(|(i, _)| val & (1u64 << i) != 0)
        .map(|(_, name)| *name)
        .collect();

    if caps.is_empty() {
        // Fall back to raw value if no names were found.
        format!("{:#x}", val)
    } else {
        caps.join(", ")
    }
}

/// Given the permitted and effective capability bitmasks, return
/// human-readable descriptions for each set.
pub fn format_caps(permitted: u64, effective: u64) -> (String, String) {
    (caps_to_string(permitted), caps_to_string(effective))
}
