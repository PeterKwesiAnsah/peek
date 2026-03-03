// Full struct as JSON. Consumes a type that implements Serialize (peek_core::ProcessInfo).
pub fn to_json<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}
