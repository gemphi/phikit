//! # NSScrollView — Scrollable Container View
//!
//! A view that provides scrolling functionality for content larger than the visible area.
//!
//! ## Apple Documentation
//! - [NSScrollView](https://developer.apple.com/documentation/appkit/nsscrollview)
//! - [NSClipView](https://developer.apple.com/documentation/appkit/nsclipview)
//! - [NSScroller](https://developer.apple.com/documentation/appkit/nsscroller)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut scroll_view = NSScrollView::new(frame);
//! scroll_view.set_document_view(content_view);
//! scroll_view.set_border_type(NSBorderType::BezelBorder);
//! ```
//!
//! ## See Also
//! - [NSTableView](https://developer.apple.com/documentation/appkit/nstableview)
//! - [NSTextView](https://developer.apple.com/documentation/appkit/nstextview)

use crate::{NSPoint, NSRect, NSView};

/// Border type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSBorderType {
    NoBorder,
    LineBorder,
    BezelBorder,
    GrooveBorder,
}

/// Scroller style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSScrollerStyle {
    Legacy,
    Overlay,
}

/// Clip view - content view of scroll view
#[derive(Debug, Clone)]
pub struct NSClipView {
    view: NSView,
    document_view: Option<Box<NSView>>,
    copies_on_scroll: bool,
}

/// Scroller - scrollbar control
#[derive(Debug, Clone)]
pub struct NSScroller {
    view: NSView,
    knob_proportion: f64,
    double_value: f64,
}

/// Scroll view
#[derive(Debug, Clone)]
pub struct NSScrollView {
    view: NSView,
    content_view: NSClipView,
    document_view: Option<Box<NSView>>,
    border_type: NSBorderType,
}

struct ScrollViewDelegate;

impl crate::view::NSViewDelegate for ScrollViewDelegate {
    fn draw(&self, _view: &NSView, _ctx: &mut crate::render::RenderContext) {
        // Draw is handled by subviews
    }

    fn scroll_wheel(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let mut bounds = view.bounds();
        let frame = view.frame();

        // Find document (content) size from the first subview
        let doc_height = view
            .subviews()
            .first()
            .map(|sub| sub.frame().height())
            .unwrap_or(0.0);
        let doc_width = view
            .subviews()
            .first()
            .map(|sub| sub.frame().width())
            .unwrap_or(0.0);

        // Apply scroll delta directly — deltas arrive in pixel units from the
        // run loop (LineDelta is pre-converted at 20px/line, PixelDelta is
        // already in logical pixels).
        bounds.origin.y -= event.delta_y;
        bounds.origin.x -= event.delta_x;

        // Clamp scroll_offset to [0, content_size - visible_size]
        let max_y = (doc_height - frame.height()).max(0.0);
        let max_x = (doc_width - frame.width()).max(0.0);

        bounds.origin.y = bounds.origin.y.clamp(0.0, max_y);
        bounds.origin.x = bounds.origin.x.clamp(0.0, max_x);

        view.set_bounds(bounds);
        view.set_needs_display();
    }

    fn wants_scroll_wheel(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl NSScrollView {
    /// Create a new scroll view
    pub fn new(frame: NSRect) -> Self {
        let content_frame = NSRect::sized(frame.width(), frame.height());
        let content_view = NSClipView::new(content_frame);

        let mut view = NSView::new(frame).with_name("NSScrollView");
        view.set_delegate(ScrollViewDelegate);
        view.set_clips_to_bounds(true);

        Self {
            view,
            content_view,
            document_view: None,
            border_type: NSBorderType::BezelBorder,
        }
    }

    /// Set document view
    pub fn set_document_view(&mut self, mut view: NSView) {
        self.document_view = Some(Box::new(view.clone()));
        self.content_view.document_view =
            self.document_view.as_ref().map(|v| Box::new((**v).clone()));

        // Remove old document view if any
        self.view.subviews_mut().clear();

        // Ensure the document view sizes itself at least as big as its intrinsic content,
        // but for now we just add it as a subview.
        self.view.add_subview(view);
    }

    /// Get document view
    pub fn document_view(&self) -> Option<&NSView> {
        self.document_view.as_ref().map(|v| v.as_ref())
    }

    /// Set border type
    pub fn set_border_type(&mut self, border_type: NSBorderType) {
        self.border_type = border_type;
    }

    /// Scroll to point
    pub fn scroll_to_point(&mut self, point: NSPoint) {
        let mut bounds = self.view.bounds();
        bounds.origin = point;
        self.view.set_bounds(bounds);
    }

    /// Scroll to visible
    pub fn scroll_to_visible(&mut self, rect: NSRect) {
        // Mock visibility adjustment
        let _ = rect;
    }

    /// Flash scrollers
    pub fn flash_scrollers(&self) {
        // Mock scroller feedback
    }

    /// Content view
    pub fn content_view(&self) -> &NSClipView {
        &self.content_view
    }

    /// Access underlying view
    pub fn view(&self) -> &NSView {
        &self.view
    }

    /// Access underlying view mutably
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }

    /// Backing view (clone)
    pub fn view_clone(&self) -> NSView {
        self.view.clone()
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Builder Pattern Methods
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Set document view using builder pattern
    pub fn with_document_view(mut self, view: NSView) -> Self {
        self.set_document_view(view);
        self
    }
    
    /// Set border type using builder pattern
    pub fn with_border_type(mut self, border_type: NSBorderType) -> Self {
        self.set_border_type(border_type);
        self
    }
    
    /// Set the frame using builder pattern
    pub fn with_frame(mut self, frame: NSRect) -> Self {
        self.view.set_frame(frame);
        self
    }
    
    /// Set the name using builder pattern
    pub fn with_name(mut self, name: &str) -> Self {
        self.view.set_name(name);
        self
    }
    
    /// Set the background color using builder pattern
    pub fn with_background_color(mut self, color: Option<crate::graphics::NSColor>) -> Self {
        self.view.set_background_color(color);
        self
    }
    
    /// Set whether to clip to bounds using builder pattern
    pub fn with_clips_to_bounds(mut self, clips: bool) -> Self {
        self.view.set_clips_to_bounds(clips);
        self
    }
}

impl NSClipView {
    /// Create a new clip view
    pub fn new(frame: NSRect) -> Self {
        Self {
            view: NSView::new(frame),
            document_view: None,
            copies_on_scroll: true,
        }
    }

    /// Set document view
    pub fn set_document_view(&mut self, view: Option<Box<NSView>>) {
        self.document_view = view;
    }

    /// Get document view
    pub fn document_view(&self) -> Option<&NSView> {
        self.document_view.as_ref().map(|v| v.as_ref())
    }

    /// Set copies on scroll
    pub fn set_copies_on_scroll(&mut self, copies: bool) {
        self.copies_on_scroll = copies;
    }

    /// Set bounds origin (for scrolling)
    pub fn set_bounds_origin(&mut self, point: NSPoint) {
        let mut bounds = self.view.bounds();
        bounds.origin = point;
        self.view.set_bounds(bounds);
    }

    /// Access underlying view
    pub fn view(&self) -> &NSView {
        &self.view
    }
}

impl NSScroller {
    /// Create horizontal scroller
    pub fn horizontal() -> Self {
        Self {
            view: NSView::new(NSRect::sized(100.0, 15.0)),
            knob_proportion: 1.0,
            double_value: 0.0,
        }
    }

    /// Create vertical scroller
    pub fn vertical() -> Self {
        Self {
            view: NSView::new(NSRect::sized(15.0, 100.0)),
            knob_proportion: 1.0,
            double_value: 0.0,
        }
    }

    /// Set knob proportion
    pub fn set_knob_proportion(&mut self, proportion: f64) {
        self.knob_proportion = proportion.clamp(0.0, 1.0);
    }

    /// Set double value (scroll position)
    pub fn set_double_value(&mut self, value: f64) {
        self.double_value = value.clamp(0.0, 1.0 - self.knob_proportion);
    }

    /// Access underlying view
    pub fn view(&self) -> &NSView {
        &self.view
    }
}
