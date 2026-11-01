use super::window::NSWindow;

/// Window delegate trait
pub trait NSWindowDelegate: Send + Sync {
    fn window_should_close(&self, _window: &NSWindow) -> bool {
        true
    }
    fn window_will_close(&self, _window: &NSWindow) {}
    fn window_did_resize(&self, _window: &NSWindow) {}
    fn window_did_move(&self, _window: &NSWindow) {}
    fn window_did_become_key(&self, _window: &NSWindow) {}
    fn window_did_resign_key(&self, _window: &NSWindow) {}
}
