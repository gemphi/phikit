//! View hierarchy: naming, subview management, frame/bounds, view lookup, and delegate.

use crate::graphics::{NSAutoresizingMask, NSRect};
use std::sync::{Arc, Mutex};
use super::properties::{NSView, NSViewDelegate};

impl NSView {
    pub fn set_identifier(&mut self, identifier: &str) {
        self.identifier = Some(identifier.to_string());
    }
    pub fn identifier(&self) -> Option<String> { self.identifier.clone() }

    /// **with_identifier** — Chaining setter
    pub fn with_identifier(mut self, identifier: &str) -> Self {
        self.set_identifier(identifier);
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    pub fn set_name(&mut self, name: &str) { self.name = name.to_string(); }

    // ── Builder Methods ──────────────────────────────────────────────

    /// Set the frame (builder pattern).
    pub fn with_frame(mut self, frame: NSRect) -> Self {
        self.set_frame(frame);
        self
    }

    /// Set the width (builder pattern).
    pub fn with_width(mut self, width: f32) -> Self {
        let current = self.frame;
        self.set_frame(NSRect::new(current.x(), current.y(), width, current.height()));
        self
    }

    /// Set the height (builder pattern).
    pub fn with_height(mut self, height: f32) -> Self {
        let current = self.frame;
        self.set_frame(NSRect::new(current.x(), current.y(), current.width(), height));
        self
    }

    /// Set the background color (builder pattern).
    pub fn with_background_color(mut self, color: Option<crate::graphics::NSColor>) -> Self {
        self.background_color = color;
        self
    }

    /// Set the corner radius (builder pattern).
    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set the border color (builder pattern).
    pub fn with_border_color(mut self, color: Option<crate::graphics::NSColor>) -> Self {
        self.border_color = color;
        self
    }

    /// Set the border width (builder pattern).
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Set the alpha value (builder pattern).
    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha_value = alpha;
        self
    }

    /// Set the hidden state (builder pattern).
    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.is_hidden = hidden;
        self
    }

    /// Set the enabled state (builder pattern).
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.is_enabled = enabled;
        self
    }

    /// Set the hover color (builder pattern).
    pub fn with_hover_color(mut self, color: Option<crate::graphics::NSColor>) -> Self {
        self.hover_color = color;
        self
    }

    /// Set the gradient (builder pattern).
    pub fn with_gradient(mut self, top: crate::graphics::NSColor, bottom: crate::graphics::NSColor) -> Self {
        self.gradient_top = Some(top);
        self.gradient_bottom = Some(bottom);
        self
    }

    /// Add a subview (builder pattern).
    pub fn with_subview(mut self, view: NSView) -> Self {
        self.add_subview(view);
        self
    }

    /// Set the glass effect (builder pattern).
    pub fn with_glass(mut self, enabled: bool) -> Self {
        self.is_glass = enabled;
        self
    }

    /// Set the clips to bounds (builder pattern).
    pub fn with_clips_to_bounds(mut self, clips: bool) -> Self {
        self.clips_to_bounds = clips;
        self
    }

    /// Set the autoresizing mask (builder pattern).
    pub fn with_autoresizing_mask(mut self, mask: NSAutoresizingMask) -> Self {
        self.autoresizing_mask = mask;
        self
    }

    /// Set the tool tip (builder pattern).
    pub fn with_tool_tip(mut self, tip: &str) -> Self {
        self.tool_tip = Some(tip.to_string());
        self
    }

    pub fn add_subview(&mut self, mut view: NSView) {
        view.bounds = NSRect::sized(view.frame.width(), view.frame.height());
        self.subviews.push(view);
    }

    pub fn frame(&self) -> NSRect { self.frame }
    pub fn set_frame(&mut self, frame: NSRect) {
        let old_size = self.frame.size;
        self.frame = frame;
        self.bounds.size = frame.size;
        if old_size.width != frame.width() || old_size.height != frame.height() {
            self.autoresize_subviews(old_size);
            self.layout_subviews();
        }
    }

    /// Resizes the view to the given dimensions, resetting origin to (0, 0).
    pub fn resize(&mut self, width: f32, height: f32) {
        self.set_frame(NSRect::sized(width, height));
    }

    pub fn bounds(&self) -> NSRect { self.bounds }
    pub fn set_bounds(&mut self, bounds: NSRect) { self.bounds = bounds; }

    pub fn subviews(&self) -> &[NSView] { &self.subviews }
    pub fn subviews_mut(&mut self) -> &mut Vec<NSView> { &mut self.subviews }

    pub fn set_autoresizing_mask(&mut self, mask: NSAutoresizingMask) {
        self.autoresizing_mask = mask;
    }
    pub fn autoresizing_mask(&self) -> NSAutoresizingMask { self.autoresizing_mask }

    pub fn find_view_by_id(&mut self, target_id: u64) -> Option<&mut NSView> {
        if self.id == target_id { return Some(self); }
        for subview in &mut self.subviews {
            if let Some(found) = subview.find_view_by_id(target_id) { return Some(found); }
        }
        None
    }

    pub fn remove_all_subviews(&mut self) { self.subviews.clear(); }

    /// Remove and return all subviews, transferring ownership to the caller.
    pub fn take_subviews(&mut self) -> Vec<NSView> { std::mem::take(&mut self.subviews) }

    pub fn layout_subviews(&mut self) {
        if let Some(delegate_arc) = self.delegate.clone() {
            if let Ok(mut d) = delegate_arc.lock() { d.layout(self); }
        }
        for subview in &mut self.subviews { subview.layout_subviews(); }
    }

    pub fn is_hidden(&self) -> bool { self.is_hidden }
    pub fn set_hidden(&mut self, hidden: bool) { self.is_hidden = hidden; }

    /// Request a redraw of the window containing this view.
    pub fn set_needs_display(&self) {
        if let Ok(mut app) = crate::application::NSApplication::shared().try_lock() {
            if let Some(win) = app.window_at_mut(0) { win.draw(); }
        }
    }

    pub fn find_view(&self, id: u64) -> Option<&NSView> {
        if self.id == id { return Some(self); }
        for subview in &self.subviews {
            if let Some(v) = subview.find_view(id) { return Some(v); }
        }
        None
    }

    pub fn find_view_mut(&mut self, id: u64) -> Option<&mut NSView> {
        if self.id == id { return Some(self); }
        for subview in &mut self.subviews {
            if let Some(v) = subview.find_view_mut(id) { return Some(v); }
        }
        None
    }

    pub fn find_view_by_name(&self, name: &str) -> Option<&NSView> {
        if self.name == name { return Some(self); }
        for subview in &self.subviews {
            if let Some(v) = subview.find_view_by_name(name) { return Some(v); }
        }
        None
    }

    pub fn find_view_mut_by_name(&mut self, name: &str) -> Option<&mut NSView> {
        if self.name == name { return Some(self); }
        for subview in &mut self.subviews {
            if let Some(v) = subview.find_view_mut_by_name(name) { return Some(v); }
        }
        None
    }

    pub fn set_delegate(&mut self, delegate: impl NSViewDelegate + 'static) {
        self.delegate = Some(Arc::new(Mutex::new(delegate)));
    }

    pub fn set_delegate_arc(&mut self, delegate: Arc<Mutex<dyn NSViewDelegate>>) {
        self.delegate = Some(delegate);
    }

    pub fn delegate(&self) -> Option<Arc<Mutex<dyn NSViewDelegate>>> {
        self.delegate.clone()
    }
}
