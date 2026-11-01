//! View properties: delegate trait, core types, struct definition, and constructors.

use crate::graphics::{NSAutoresizingMask, NSColor, NSPoint, NSRect, NSSize};
use std::sync::{Arc, Mutex};
use super::constraints::NSLayoutConstraint;

/// View delegate trait
pub trait NSViewDelegate: Send + Sync {
    fn view_did_move_to_window(&mut self, _view: &NSView) {}
    fn draw(&self, _view: &NSView, _context: &mut crate::render::RenderContext) {}
    fn hit_test(&self, _view: &NSView, _point: NSPoint) -> bool { true }
    /// Return true if this delegate wants to intercept a mouse_down at this
    /// point BEFORE the event is dispatched to children.
    fn wants_mouse_down(&self, _point: NSPoint) -> bool { false }
    fn mouse_down(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {}
    fn mouse_up(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {}
    fn right_mouse_down(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {}
    fn mouse_dragged(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {}
    fn mouse_moved(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {}
    fn wants_scroll_wheel(&self) -> bool { false }
    fn scroll_wheel(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {}
    fn key_down(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {}
    fn key_up(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {}
    fn dropped_file(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {}
    fn dragging_entered(
        &mut self, _view: &mut NSView, _info: &crate::event::NSDraggingInfo,
    ) -> crate::event::NSDragOperation { crate::event::NSDragOperation::None }
    fn dragging_updated(
        &mut self, _view: &mut NSView, _info: &crate::event::NSDraggingInfo,
    ) -> crate::event::NSDragOperation { crate::event::NSDragOperation::None }
    fn dragging_exited(&mut self, _view: &mut NSView, _info: &crate::event::NSDraggingInfo) {}
    fn perform_drag_operation(
        &mut self, _view: &mut NSView, _info: &crate::event::NSDraggingInfo,
    ) -> bool { false }
    fn layout(&mut self, _view: &mut NSView) {}
    fn intrinsic_content_size(&self, _view: &NSView) -> NSSize { NSSize::new(100.0, 100.0) }
    /// Called for GPU-accelerated rendering.
    fn gpu_draw(
        &self, _view: &NSView, _renderer: &crate::gpu::GpuRenderer,
        _encoder: &mut wgpu::CommandEncoder, _target_view: &wgpu::TextureView,
    ) {}
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSLayoutMode {
    FrameBased,
    AutoLayout,
    StackView,
}

/// User interface layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSUserInterfaceLayoutDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Clone)]
pub struct NSView {
    pub id: u64,
    pub name: String,
    pub(crate) frame: NSRect,
    pub(crate) bounds: NSRect,
    pub(crate) subviews: Vec<NSView>,
    pub(crate) is_hidden: bool,
    pub(crate) is_enabled: bool,
    pub(crate) is_flipped: bool,
    pub(crate) is_opaque: bool,
    pub(crate) allows_vibrancy: bool,
    pub(crate) user_interface_layout_direction: NSUserInterfaceLayoutDirection,
    pub(crate) autoresizing_mask: NSAutoresizingMask,
    pub(crate) background_color: Option<NSColor>,
    pub(crate) gradient_top: Option<NSColor>,
    pub(crate) gradient_bottom: Option<NSColor>,
    pub(crate) alpha_value: f32,
    pub(crate) delegate: Option<Arc<Mutex<dyn NSViewDelegate>>>,
    pub(crate) constraints: Vec<NSLayoutConstraint>,
    pub(crate) intrinsic_size: NSSize,
    pub(crate) min_size: NSSize,
    pub(crate) max_size: NSSize,
    pub(crate) layout_mode: NSLayoutMode,
    pub(crate) theme_token: Option<String>,
    pub(crate) radius_token: Option<String>,
    pub(crate) glass_token: Option<String>,
    pub(crate) border_color_token: Option<String>,
    pub(crate) border_width: f32,
    pub(crate) border_color: Option<NSColor>,
    pub(crate) corner_radius: f32,
    pub(crate) is_glass: bool,
    pub(crate) glass_style: crate::glass::GlassStyle,
    pub(crate) glass_tint: Option<NSColor>,
    /// With a value, this view renders its subtree's glass as one layer.
    pub(crate) glass_container: Option<f32>,
    pub(crate) clips_to_bounds: bool,
    pub(crate) wants_layer: bool,
    pub(crate) tool_tip: Option<String>,
    pub(crate) identifier: Option<String>,
    /// Color to use when the view is hovered.
    pub(crate) hover_color: Option<NSColor>,
    /// Whether the cursor is currently over this view.
    pub(crate) is_hovered: bool,
    /// Per-view appearance override. If `None`, inherits from parent (or system).
    pub(crate) appearance: Option<crate::theme::NSAppearance>,
    /// Tracks whether this view's delegate is currently handling a drag sequence.
    pub(crate) delegate_is_dragging: bool,
}

impl std::fmt::Debug for NSView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NSView")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("frame", &self.frame)
            .field("subviews", &self.subviews.len())
            .finish()
    }
}

impl NSView {
    pub fn new(frame: NSRect) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            name: String::new(),
            frame,
            bounds: NSRect::sized(frame.width(), frame.height()),
            subviews: Vec::new(),
            is_hidden: false,
            is_enabled: true,
            is_flipped: false,
            is_opaque: false,
            allows_vibrancy: false,
            user_interface_layout_direction: NSUserInterfaceLayoutDirection::LeftToRight,
            autoresizing_mask: NSAutoresizingMask::default(),
            background_color: None,
            gradient_top: None,
            gradient_bottom: None,
            alpha_value: 1.0,
            delegate: None,
            constraints: Vec::new(),
            intrinsic_size: NSSize::new(100.0, 100.0),
            min_size: NSSize::new(0.0, 0.0),
            max_size: NSSize::new(f32::INFINITY, f32::INFINITY),
            layout_mode: NSLayoutMode::FrameBased,
            theme_token: None,
            radius_token: None,
            glass_token: None,
            border_color_token: None,
            border_width: 0.0,
            border_color: None,
            corner_radius: 0.0,
            is_glass: false,
            glass_style: crate::glass::GlassStyle::default(),
            glass_tint: None,
            glass_container: None,
            clips_to_bounds: false,
            wants_layer: false,
            tool_tip: None,
            identifier: None,
            hover_color: None,
            is_hovered: false,
            appearance: None,
            delegate_is_dragging: false,
        }
    }

    /// Creates a view at the origin (0, 0) with the given width and height.
    pub fn sized(width: f32, height: f32) -> Self {
        Self::new(NSRect::sized(width, height))
    }

    /// Creates a view at (x, y) with the given width and height.
    pub fn at(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self::new(NSRect::new(x, y, w, h))
    }
}
