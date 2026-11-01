use super::window::NSWindow;
use super::NSWindowStyle;
use crate::graphics::NSRect;
use crate::view::NSView;

// ═══════════════════════════════════════════════════════════════════════
// WindowController trait — implemented by rust-kit-pro windows
// ═══════════════════════════════════════════════════════════════════════

/// Trait for objects that control a window's content and lifecycle.
///
/// Implementors provide BOTH:
///
/// **Window identity** (what the window IS):
/// - `window_title()` — title bar text
/// - `window_size()` — initial dimensions
/// - `is_resizable()` — can the user resize
/// - `build_content()` — create the content view tree
/// - `configure()` — populate an existing content view in-place
///
/// **Controller actions** (what the window DOES):
/// - `on_close()` — cleanup when window closes
/// - `on_show()` — called after window appears
/// - `on_become_key()` — window became active
/// - `on_resign_key()` — window lost focus
///
/// In rust-kit-pro, each window folder splits these into:
/// - `window.rs` — struct + impl for window identity methods
/// - `controller.rs` — impl for controller action methods
pub trait WindowController: Send + Sync {
    // ── Window identity ──────────────────────────────────────────────
    fn window_title(&self) -> &str;
    fn window_size(&self) -> (f32, f32);
    fn is_resizable(&self) -> bool { true }
    fn build_content(&self) -> NSView { NSView::new(crate::graphics::NSRect::zero()) }
    fn configure(&self, content_view: &mut NSView);

    // ── Controller actions ───────────────────────────────────────────
    fn on_close(&mut self) {}
    fn on_show(&mut self) {}
    fn on_become_key(&mut self) {}
    fn on_resign_key(&mut self) {}
}

// ═══════════════════════════════════════════════════════════════════════
// NSWindowController — owns an NSWindow, manages lifecycle
// ═══════════════════════════════════════════════════════════════════════

/// NSWindowController — owns an NSWindow and manages its lifecycle.
/// Created from a `WindowController` implementor.
pub struct NSWindowController {
    pub window: NSWindow,
}

impl NSWindowController {
    pub fn from_controller(controller: &dyn WindowController) -> Self {
        let window = NSWindow::from_controller(controller);
        Self { window }
    }

    pub fn new() -> Self {
        Self {
            window: NSWindow::new(NSRect::new(100.0, 100.0, 800.0, 600.0), &[
                NSWindowStyle::Titled,
                NSWindowStyle::Closable,
                NSWindowStyle::Resizable,
            ]),
        }
    }

    pub fn show_window(&mut self) { self.window.make_key_and_order_front(); }
    pub fn set_content_view(&mut self, view: NSView) { self.window.set_content_view(view); }
    pub fn window(&self) -> &NSWindow { &self.window }
    pub fn window_mut(&mut self) -> &mut NSWindow { &mut self.window }
}

// ═══════════════════════════════════════════════════════════════════════
// NSViewController — manages a single view
// ═══════════════════════════════════════════════════════════════════════

pub struct NSViewController {
    pub view: Option<NSView>,
    pub title: String,
}

impl NSViewController {
    pub fn new() -> Self {
        Self { view: None, title: String::new() }
    }
    pub fn load_view(&mut self) {
        if self.view.is_none() {
            self.view = Some(NSView::new(NSRect::sized(100.0, 100.0)));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// WindowProvider — trait for types that can provide a configured window
// ═══════════════════════════════════════════════════════════════════════

/// Trait for types that can provide a fully configured window.
/// Separates window creation from controller lifecycle.
pub trait WindowProvider: Send + Sync {
    fn provide_window(&self) -> NSWindow;
}

// ═══════════════════════════════════════════════════════════════════════
// WindowControllerActions — trait for controller action handlers
// ═══════════════════════════════════════════════════════════════════════

/// Trait for controller action handlers (show, close, focus events).
pub trait WindowControllerActions: Send + Sync {
    fn on_show(&mut self) {}
    fn on_close(&mut self) {}
    fn on_become_key(&mut self) {}
    fn on_resign_key(&mut self) {}
}
