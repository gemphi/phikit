//! # NSWindow — the window object.
//!
//! Contains the `NSWindow` struct definition, builder API, setters, getters,
//! and chrome layout/drawing methods.

use crate::responder::NSResponder;
use crate::{NSColor, NSPoint, NSRect, NSSize, NSView};
use softbuffer::Surface;
use std::sync::{Arc, Mutex};
use winit::window::Window;

use super::controller;
use super::delegate::NSWindowDelegate;
use super::NSWindowStyle;

/// Window struct
///
/// DEVIATION JUSTIFICATION:
/// We use Arc<Mutex<Surface>> for the software renderer because softbuffer's
/// Surface must be thread-safe for our Arc<Mutex<NSApplication>> model.
pub struct NSWindow {
    pub frame: NSRect,
    pub title: String,
    pub content_view: Option<Box<NSView>>,
    pub toolbar_view: Option<NSView>,
    pub toolbar_height: f32,
    pub toolbar: Option<crate::toolbar::NSToolbar>,
    pub delegate: Option<Arc<Mutex<dyn NSWindowDelegate>>>,
    pub is_key: bool,
    pub is_visible: bool,
    pub is_miniaturized: bool,
    pub background_color: NSColor,
    pub min_size: NSSize,
    pub max_size: NSSize,
    pub winit_window: Option<Arc<Window>>,
    pub surface: Option<Arc<Mutex<Surface<Arc<Window>, Arc<Window>>>>>,
    pub last_mouse_pos: NSPoint,
    pub first_responder: Option<Box<dyn NSResponder>>,
    pub style_mask: Vec<NSWindowStyle>,
    pub shortcut_handler: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
    pub last_hovered_view_id: Option<u64>,
}

impl NSWindow {
    pub fn new(frame: NSRect, style: &[NSWindowStyle]) -> Self {
        Self {
            frame,
            title: String::new(),
            content_view: None,
            toolbar_view: None,
            toolbar_height: 0.0,
            toolbar: None,
            delegate: None,
            is_key: false,
            is_visible: false,
            is_miniaturized: false,
            background_color: crate::theme::color(crate::theme::keys::CLR_BG_WINDOW),
            min_size: NSSize::new(100.0, 100.0),
            max_size: NSSize::new(10000.0, 10000.0),
            winit_window: None,
            surface: None,
            last_mouse_pos: NSPoint::new(0.0, 0.0),
            first_responder: None,
            style_mask: style.to_vec(),
            shortcut_handler: None,
            last_hovered_view_id: None,
        }
    }

    // ── Builder API ─────────────────────────────────────────────────────

    /// Create a window from a title string. Returns self for chaining.
    /// This is the entry point for the fluent builder pattern.
    pub fn new_titled(title: &str) -> Self {
        let mut win = Self::new(
            NSRect::new(100.0, 100.0, 800.0, 600.0),
            &[NSWindowStyle::Titled, NSWindowStyle::Closable, NSWindowStyle::Miniaturizable],
        );
        win.set_title(title);
        win
    }

    /// Set the window size. Builder method — returns self.
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.frame = NSRect::new(self.frame.origin.x, self.frame.origin.y, width, height);
        self
    }

    /// Set the window style mask. Builder method — returns self.
    pub fn with_style(mut self, styles: &[NSWindowStyle]) -> Self {
        self.style_mask = styles.to_vec();
        self
    }

    /// Set whether the window is resizable. Builder method — returns self.
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        if resizable && !self.style_mask.contains(&NSWindowStyle::Resizable) {
            self.style_mask.push(NSWindowStyle::Resizable);
        } else if !resizable {
            self.style_mask.retain(|s| *s != NSWindowStyle::Resizable);
        }
        self
    }

    /// Set the content view. Builder method — returns self.
    pub fn with_content(mut self, view: NSView) -> Self {
        self.content_view = Some(Box::new(view));
        self
    }

    /// Set the toolbar. Builder method — returns self.
    pub fn with_toolbar_view(mut self, view: NSView) -> Self {
        let h = view.frame().size.height;
        self.toolbar_height = h;
        self.toolbar_view = Some(view);
        self
    }

    /// Set the delegate. Builder method — returns self.
    pub fn with_delegate(mut self, delegate: Arc<Mutex<dyn NSWindowDelegate>>) -> Self {
        self.delegate = Some(delegate);
        self
    }

    /// Set the minimum size. Builder method — returns self.
    pub fn with_min_size(mut self, width: f32, height: f32) -> Self {
        self.min_size = NSSize::new(width, height);
        self
    }

    /// Set the window position. Builder method — returns self.
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.frame.origin.x = x;
        self.frame.origin.y = y;
        self
    }

    /// Create a window with a custom title bar (full-size content style).
    pub fn with_custom_title_bar(title: &str, width: f32, height: f32) -> Self {
        let mut win = Self::new(
            NSRect::new(100.0, 100.0, width, height),
            &[
                NSWindowStyle::Titled,
                NSWindowStyle::Closable,
                NSWindowStyle::Miniaturizable,
                NSWindowStyle::Resizable,
                NSWindowStyle::FullSizeContent,
            ],
        );
        win.set_title(title);
        win
    }

    // ── Factory ─────────────────────────────────────────────────────────

    /// Creates an NSWindow configured by a WindowController implementor.
    /// The controller declares the window's identity (title, size, resizable)
    /// and the window applies those as its style and defaults.
    pub fn from_controller(controller: &dyn controller::WindowController) -> Self {
        let (w, h) = controller.window_size();
        let mut style = vec![
            NSWindowStyle::Titled,
            NSWindowStyle::Closable,
            NSWindowStyle::Miniaturizable,
            NSWindowStyle::FullSizeContent,
        ];
        if controller.is_resizable() {
            style.push(NSWindowStyle::Resizable);
        }
        let mut win = Self::new(NSRect::new(100.0, 100.0, w, h), &style);
        win.set_title(controller.window_title());
        win
    }

    // ── Setters ─────────────────────────────────────────────────────────

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        if let Some(ref window) = self.winit_window {
            window.set_title(title);
        }
    }

    pub fn set_content_view(&mut self, view: NSView) {
        self.content_view = Some(Box::new(view));
    }

    pub fn set_content_view_controller(
        &mut self,
        mut controller: crate::split_view::NSSplitViewController,
    ) {
        self.set_content_view(controller.view().clone());
    }

    /// Set the toolbar. The window will automatically compile its views
    /// and position it at the top, offsetting the content_view below it.
    pub fn set_toolbar(&mut self, toolbar: crate::toolbar::NSToolbar) {
        let view = toolbar.build_view(self.frame.width());
        self.toolbar_height = view.frame().size.height;
        self.toolbar_view = Some(view);
        self.toolbar = Some(toolbar);
        self.layout_chrome();
    }

    /// Set the toolbar view directly.
    pub fn set_toolbar_view(&mut self, view: NSView) {
        let h = view.frame().size.height;
        self.toolbar_height = h;
        self.toolbar_view = Some(view);
        self.layout_chrome();
    }

    pub fn set_background_color(&mut self, color: NSColor) {
        self.background_color = color;
    }

    pub fn set_min_size(&mut self, size: NSSize) {
        self.min_size = size;
    }

    /// Set the window position (x, y coordinates)
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.frame.origin.x = x;
        self.frame.origin.y = y;
        // If winit window exists, also update its position
        if let Some(ref window) = self.winit_window {
            window.set_outer_position(winit::dpi::PhysicalPosition::new(x as i32, y as i32));
        }
    }

    pub fn set_shortcut_handler(&mut self, handler: Arc<dyn Fn(&str) -> bool + Send + Sync>) {
        self.shortcut_handler = Some(handler);
    }

    // ── Getters ─────────────────────────────────────────────────────────

    pub fn content_view(&self) -> Option<&NSView> {
        self.content_view.as_ref().map(|v| v.as_ref())
    }

    pub fn content_view_mut(&mut self) -> Option<&mut NSView> {
        self.content_view.as_mut().map(|v| v.as_mut())
    }

    pub fn toolbar_view(&self) -> Option<&NSView> {
        self.toolbar_view.as_ref()
    }

    pub fn toolbar_view_mut(&mut self) -> Option<&mut NSView> {
        self.toolbar_view.as_mut()
    }

    /// Returns the window's logical size. Before winit initialises the physical
    /// surface this matches the size passed to `with_custom_title_bar`, which is
    /// sufficient for computing the initial panel layout.
    pub fn inner_size_logical(&self) -> (f32, f32) {
        match self.winit_window {
            Some(ref w) => {
                let sz = w.inner_size();
                let scale = w.scale_factor() as f32;
                (sz.width as f32 / scale, sz.height as f32 / scale)
            }
            None => (self.frame.width(), self.frame.height()),
        }
    }

    // ── Chrome Layout & Drawing ─────────────────────────────────────────

    /// Layout window chrome: positions toolbar at top, content flush below it.
    /// Called automatically when toolbar or frame changes.
    pub fn layout_chrome(&mut self) {
        let win_w = self.frame.width();
        let win_h = self.frame.height();

        // Responsively re-compile the toolbar view with the updated window width
        match self.toolbar {
            Some(ref toolbar) => {
                let view = toolbar.build_view(win_w);
                self.toolbar_view = Some(view);
            }
            None => {
                let _ = self.toolbar_view.as_mut().map(|v| {
                    v.set_frame(NSRect::sized(win_w, self.toolbar_height));
                });
            }
        }

        let _ = self.content_view.as_mut().map(|content| {
            let content_y = self.toolbar_height;
            let content_h = (win_h - content_y).max(0.0);
            content.set_frame(NSRect::new(0.0, content_y, win_w, content_h));
        });
    }

    pub fn draw(&mut self) {
        // Layout chrome before drawing
        self.layout_chrome();
        if let Some(ref mut _toolbar) = self.toolbar_view {
            // toolbar.draw(...);
        }
        if let Some(ref mut _content) = self.content_view {
            // content.draw(...);
        }
    }

    pub fn center(&mut self) {
        if let Some(ref window) = self.winit_window {
            if let Some(monitor) = window.current_monitor() {
                let monitor_size = monitor.size();
                let monitor_pos = monitor.position();
                let window_size = window.outer_size();
                let x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
                let y = monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2;
                window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
            }
        }
    }
}
