//! One read owns the file at a time (`load.rs`).
//!
//! Opening a file takes the file over: a read still under way is told to
//! stop, and the generation it belongs to is left behind, so whatever it
//! returns is not adopted. Blender has one `WM_file_read` at a time; two
//! here used to race, and the second to finish replaced the scene.

use super::BPhiberApp;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

impl BPhiberApp {
    /// Take the file over for a new read: cancel the one under way, if any,
    /// and return the new read's generation and cancel flag.
    pub(super) fn take_load(&mut self) -> (u64, Arc<AtomicBool>) {
        if let Some(previous) = self.load_owner.replace(Arc::new(AtomicBool::new(false))) {
            previous.store(true, Ordering::Relaxed);
            let was = self.state.get(&crate::v2::scene::selected_key()).unwrap_or("").to_string();
            super::journal::BJournal::note(format!("Open: a read of {was} was under way and is cancelled — this one takes over"));
        }
        self.load_gen = self.load_gen.wrapping_add(1);
        (self.load_gen, self.load_owner.clone().expect("set just above"))
    }
}
