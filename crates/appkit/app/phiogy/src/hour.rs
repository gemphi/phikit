//! The hour on the person's own clock, for what they write, and the
//! moment, which makes the id of a message of theirs. In a browser
//! there is no clock to ask here yet: the hour is left out, and the
//! moment is a count.

/// The hour now, written as `format` says (`%H:%M`).
#[cfg(not(target_family = "wasm"))]
pub fn now(format: &str) -> String {
    chrono::Local::now().format(format).to_string()
}

#[cfg(target_family = "wasm")]
pub fn now(_: &str) -> String {
    String::new()
}

/// The moment now: thousandths of a second since 1970.
#[cfg(not(target_family = "wasm"))]
pub fn moment() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_millis())
}

#[cfg(target_family = "wasm")]
pub fn moment() -> u128 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNT: AtomicU64 = AtomicU64::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed) as u128
}
