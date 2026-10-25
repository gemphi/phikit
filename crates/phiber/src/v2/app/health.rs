//! What a slow session should say about itself, in the Info log where the
//! user looks: whether this is a debug build, which device phiray draws on
//! and whether that is a real GPU, and where the session journal is.
//!
//! Three questions decide where the time goes on a large file, and none of
//! them was answered anywhere on screen: a debug build lays the window out
//! several times slower than a release one; a software adapter (lavapipe,
//! WARP, SwiftShader) traces at a fraction of a card's speed and says
//! "Vulkan" like a card does; and the journal (`journal.rs`) that would
//! show the rest was at a path nothing announced. The lines go in once,
//! from the running app (`app::run` arms them), never from a test's window.

use crate::v2::state::log;
use std::sync::atomic::{AtomicBool, Ordering};

/// Set by the running app; a test's window says nothing.
static ARMED: AtomicBool = AtomicBool::new(false);
/// The device line has been written.
static DEVICE_SAID: AtomicBool = AtomicBool::new(false);

pub struct BHealth;

impl BHealth {
    /// The running app: say the startup lines and let the device line follow
    /// once phiray has opened its device.
    pub fn arm() {
        ARMED.store(true, Ordering::Release);
        for line in Self::at_start() {
            log::push(line);
        }
    }

    /// The lines for the moment the window opens.
    pub fn at_start() -> Vec<String> {
        let mut out = Vec::new();
        if cfg!(debug_assertions) {
            out.push("Debug build: the interface runs several times slower than `cargo run --release -p phiber`".to_string());
        }
        out.push(format!(
            "Session log: {} — every action with its time, every frame over {:.0} ms, the frame summaries and the tracer's timings; send it when the app feels slow",
            super::journal::BJournal::path().display(),
            super::trace::SLOW_MS
        ));
        out
    }

    /// The line for phiray's device, once it is open: its name, backend and
    /// kind, and a warning when it is a software adapter.
    pub fn device_line() -> Option<String> {
        let a = phiray::Gpu::in_use()?;
        let line = format!("Render device: {} ({}, {})", a.name, a.backend_label(), a.kind_label().to_lowercase());
        Some(if a.is_software() {
            format!("{line} — a software adapter: no GPU phiray could use was found, so Rendered shading and F12 run on the CPU; check the driver and Preferences ▸ System ▸ Render Devices")
        } else {
            line
        })
    }

    /// Each frame of the running app: the device line, the first time the
    /// device is open. A try-lock and a flag otherwise.
    pub fn each_frame() {
        if !ARMED.load(Ordering::Acquire) || DEVICE_SAID.load(Ordering::Acquire) {
            return;
        }
        if let Some(line) = Self::device_line() {
            DEVICE_SAID.store(true, Ordering::Release);
            log::push(line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_startup_lines_name_the_build_and_the_frame_log() {
        // The debug warning leads in a debug build and is absent in a release
        // one — `cargo test --release` runs these too.
        let lines = BHealth::at_start();
        assert_eq!(lines[0].starts_with("Debug build"), cfg!(debug_assertions), "{lines:?}");
        assert!(lines.last().unwrap().contains("phiber-session.log"));
    }

    #[test]
    fn the_device_line_names_the_adapter_and_flags_a_software_one() {
        // Both halves are absent on a machine with no usable adapter, and they
        // are absent independently: `shared()` can hand back a GPU whose chosen
        // adapter `in_use()` has not recorded. Unwrapping the second after
        // guarding only the first is what panicked here.
        let Some(_) = phiray::Gpu::shared() else { return };
        let Some(a) = phiray::Gpu::in_use() else { return };
        let line = BHealth::device_line().expect("the device is open");
        assert!(line.starts_with(&format!("Render device: {} ({}", a.name, a.backend_label())), "{line}");
        assert_eq!(line.contains("software adapter"), a.is_software(), "{line}");
    }

    #[test]
    fn a_window_that_is_not_the_running_app_s_says_nothing() {
        log::clear();
        BHealth::each_frame();
        assert!(log::entries().is_empty(), "not armed: no device line");
    }
}
