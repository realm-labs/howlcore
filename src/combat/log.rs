//! Helpers for consistent work log formatting.

pub fn round_line(round: u32, message: impl AsRef<str>) -> String {
    format!("[Round {round}] {}", message.as_ref())
}
