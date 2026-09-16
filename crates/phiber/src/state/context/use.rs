//! Context state set methods - pure state mutations.
//!
//! Convenience helpers for object selection, frame control, and playback.

use super::BContext;

impl BContext {
    pub fn select_object(&mut self, name: &str) {
        self.active_object = Some(name.to_string());
        if !self.selected_objects.iter().any(|o| o == name) {
            self.selected_objects.push(name.to_string());
        }
    }

    pub fn deselect_all(&mut self) {
        self.selected_objects.clear();
        self.active_object = None;
    }

    pub fn set_frame(&mut self, frame: i32) {
        self.frame_current = frame;
    }

    pub fn toggle_playback(&mut self) {
        self.playing = !self.playing;
    }
}
