//! # NSWindow Actions — lifecycle mutations and cursor control.
//!
//! Window lifecycle actions that mutate state: visibility, miniaturize,
//! fullscreen, zoom, and cursor management.

use super::window::NSWindow;

impl NSWindow {
    pub fn make_key_and_order_front(&mut self) {
        self.is_visible = true;
        self.is_key = true;
        if let Some(ref window) = self.winit_window {
            window.set_visible(true);
            window.focus_window();
        }
    }

    pub fn close(&mut self) {
        // In a real app, this would notify the delegate and close the native window.
        println!("[NSWindow] Closing window: {}", self.title);
    }

    pub fn miniaturize(&mut self) {
        self.is_miniaturized = true;
        if let Some(ref window) = self.winit_window {
            window.set_minimized(true);
        }
    }

    pub fn deminiaturize(&mut self) {
        self.is_miniaturized = false;
        if let Some(ref window) = self.winit_window {
            window.set_minimized(false);
        }
    }

    pub fn is_miniaturized(&self) -> bool {
        self.is_miniaturized
    }

    pub fn toggle_fullscreen(&mut self) {
        let _ = self.winit_window.as_ref().map(|window| {
            let next = match window.fullscreen().is_some() {
                true => None,
                false => Some(winit::window::Fullscreen::Borderless(None)),
            };
            window.set_fullscreen(next);
        });
    }

    pub fn zoom(&mut self) {
        if let Some(ref window) = self.winit_window {
            let is_maximized = window.is_maximized();
            window.set_maximized(!is_maximized);
        }
    }

    /// Set the mouse cursor icon for this window.
    /// Wraps winit's `set_cursor_icon` for use by split view divider hover.
    pub fn set_cursor_icon(&self, cursor: winit::window::CursorIcon) {
        if let Some(ref w) = self.winit_window {
            w.set_cursor_icon(cursor);
        }
    }
}

/// Set the cursor icon on the key window. Convenience for delegates that
/// don't have direct access to the window (e.g., NSSplitViewDelegate).
pub fn set_cursor(cursor: winit::window::CursorIcon) {
    if let Ok(app) = crate::application::NSApplication::shared().try_lock() {
        if let Some(win) = app.window_at(0) {
            win.set_cursor_icon(cursor);
        }
    }
}
