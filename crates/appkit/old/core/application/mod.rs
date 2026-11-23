//! # NSApplication Modular Implementation
//!
//! Application management for Windows AppKit - integrates with winit.

pub mod delegate;
pub mod event_dispatch;
pub mod layout;
pub mod renderer;
pub mod run_loop;

use super::{NSPoint, NSWindow};
use softbuffer::Context;
use std::sync::{Arc, Mutex};
use winit::window::Window;

pub use delegate::NSAppDelegate;

/// Application struct (singleton pattern)
pub struct NSApplication {
    pub windows: Vec<NSWindow>,
    pub key_window: Option<usize>,
    pub main_window: Option<usize>,
    pub delegate: Option<Arc<Mutex<dyn NSAppDelegate>>>,
    pub is_running: bool,
    pub is_active: bool,
    pub is_left_mouse_down: bool,
    pub is_right_mouse_down: bool,
    pub ctrl_pressed: bool,
    pub shift_pressed: bool,
    pub opt_pressed: bool,
    pub cmd_pressed: bool,
    pub softbuffer_context: Option<Arc<Context<Arc<Window>>>>,
    pub gpu_renderer: Option<Arc<Mutex<crate::gpu::GpuRenderer>>>,
    pub last_mouse_pos: NSPoint,
    /// View that captured mouse on mouse_down — drag events go here until mouse_up
    pub captured_view_id: Option<u64>,
    pub active_drag_session: Option<crate::event::NSDraggingInfo>,
    /// Currently hovered view during a drag session
    pub drag_hovered_view_id: Option<u64>,
    pub pending_actions: Vec<Box<dyn FnOnce(&mut crate::NSView) + Send>>,
    /// Callback to rebuild the editor layout at a given size.
    /// Set by the app crate — called on resize and initial layout.
    pub on_rebuild_layout: Option<Arc<dyn Fn(&mut crate::NSView, f32, f32) + Send + Sync>>,
    /// Application menu bar (File, Edit, View, etc.)
    pub menu_bar: Option<crate::menu::MenuBar>,
    /// Callback for app-level keyboard shortcuts (set by the app crate).
    pub on_key_event: Option<Arc<dyn Fn(&crate::event::NSEvent) + Send + Sync>>,
    /// Callback for app-level scroll/mouse events (viewport camera control).
    pub on_scroll_event: Option<Arc<dyn Fn(&crate::event::NSEvent) + Send + Sync>>,
}

impl NSApplication {
    fn new() -> Self {
        Self {
            windows: Vec::new(),
            key_window: None,
            main_window: None,
            delegate: None,
            is_running: false,
            is_active: false,
            is_left_mouse_down: false,
            is_right_mouse_down: false,
            ctrl_pressed: false,
            shift_pressed: false,
            opt_pressed: false,
            cmd_pressed: false,
            softbuffer_context: None,
            gpu_renderer: None,
            last_mouse_pos: NSPoint::new(0.0, 0.0),
            captured_view_id: None,
            active_drag_session: None,
            drag_hovered_view_id: None,
            pending_actions: Vec::new(),
            on_rebuild_layout: None,
            menu_bar: None,
            on_key_event: None,
            on_scroll_event: None,
        }
    }

    /// Get shared application instance
    pub fn shared() -> Arc<Mutex<Self>> {
        static INSTANCE: std::sync::OnceLock<Arc<Mutex<NSApplication>>> =
            std::sync::OnceLock::new();
        INSTANCE
            .get_or_init(|| Arc::new(Mutex::new(NSApplication::new())))
            .clone()
    }

    pub fn begin_dragging_session(&mut self, payload: String, location: NSPoint) {
        self.active_drag_session = Some(crate::event::NSDraggingInfo {
            dragging_location: location,
            dragging_pasteboard_string: payload,
            source_operation_mask: crate::event::NSDragOperation::Copy,
        });
    }

    // ── Window Management ───────────────────────────────────────────────────

    pub fn add_window(&mut self, window: NSWindow) -> usize {
        let id = self.windows.len();
        self.windows.push(window);
        if self.main_window.is_none() {
            self.main_window = Some(id);
        }
        id
    }

    pub fn window_at(&self, index: usize) -> Option<&NSWindow> {
        self.windows.get(index)
    }

    pub fn window_at_mut(&mut self, index: usize) -> Option<&mut NSWindow> {
        self.windows.get_mut(index)
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn close_window(&mut self, index: usize) {
        if index < self.windows.len() {
            if self.key_window == Some(index) {
                self.key_window = None;
            }
            if self.main_window == Some(index) {
                self.main_window = None;
            }
            self.windows.remove(index);
        }
    }

    pub fn set_delegate(&mut self, delegate: impl NSAppDelegate + 'static) {
        self.delegate = Some(Arc::new(Mutex::new(delegate)));
    }

    pub fn set_delegate_arc(&mut self, delegate: Arc<Mutex<dyn NSAppDelegate>>) {
        self.delegate = Some(delegate);
    }

    pub fn terminate(&mut self) {
        println!("[NSApplication] Terminating...");
        self.is_running = false;
        // In a real app, this would notify the delegate and perform cleanup
        std::process::exit(0);
    }
}
