//! # NSBox — Container and Grouping View
//!
//! A container view for grouping subviews with an optional title and border.
//!
//! ## Apple Documentation
//! - [NSBox](https://developer.apple.com/documentation/appkit/nsbox)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut box_view = NSBox::new(frame);
//! box_view.set_title("Settings");
//! box_view.set_box_type(NSBoxType::Primary);
//! box_view.set_corner_radius(8.0);
//! ```
//!
//! ## See Also
//! - [NSStackView](https://developer.apple.com/documentation/appkit/nsstackview)
//! - [NSSplitView](https://developer.apple.com/documentation/appkit/nssplitview)

use crate::{NSColor, NSPoint, NSRect, NSSize, NSView, NSViewDelegate};
use crate::theme;
use std::sync::{Arc, Mutex};

/// **NSBoxType** — The visual style of the box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSBoxType {
    /// Group subviews with a standard look.
    Primary = 0,
    /// Vertical or horizontal separator line.
    Separator = 2,
    /// Custom boxes draw based entirely on user parameters.
    Custom = 4,
}

/// **NSTitlePosition** — Where the title is placed relative to the box border
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSTitlePosition {
    NoTitle = 0,
    AboveTop = 1,
    AtTop = 2,
    BelowTop = 3,
    AboveBottom = 4,
    AtBottom = 5,
    BelowBottom = 6,
}

/// **NSBox** — A container view with optional title and border
#[derive(Clone)]
pub struct NSBox {
    view: NSView,
    inner: Arc<Mutex<NSBoxState>>,
}

struct NSBoxState {
    box_type: NSBoxType,
    title: String,
    title_position: NSTitlePosition,
    border_width: f32,
    corner_radius: f32,
    border_color: NSColor,
    fill_color: NSColor,
}

impl NSBox {
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(NSBoxState {
            box_type: NSBoxType::Primary,
            title: String::new(),
            title_position: NSTitlePosition::AtTop,
            border_width: crate::NSLayoutConstants::GRID_LINE_WIDTH,
            corner_radius: crate::NSLayoutConstants::MARGIN_XSMALL,
            border_color: theme::color(theme::CLR_SEPARATOR),
            fill_color: theme::color(theme::CLR_BG_PANEL),
        }));

        let mut view = NSView::new(frame);
        view.set_delegate(NSBoxDelegate {
            state: state.clone(),
        });

        Self { view, inner: state }
    }

    /// **separator** — Create a horizontal or vertical separator line
    pub fn separator(frame: NSRect) -> Self {
        let mut b = Self::new(frame);
        b.set_box_type(NSBoxType::Separator);
        b.set_title_position(NSTitlePosition::NoTitle);
        b
    }

    pub fn set_box_type(&mut self, bt: NSBoxType) {
        self.inner.lock().unwrap().box_type = bt;
    }
    pub fn set_title(&mut self, title: &str) {
        self.inner.lock().unwrap().title = title.to_string();
    }
    pub fn set_title_position(&mut self, pos: NSTitlePosition) {
        self.inner.lock().unwrap().title_position = pos;
    }
    pub fn set_border_width(&mut self, width: f32) {
        self.inner.lock().unwrap().border_width = width;
    }
    pub fn set_corner_radius(&mut self, radius: f32) {
        self.inner.lock().unwrap().corner_radius = radius;
    }
    pub fn set_border_color(&mut self, color: NSColor) {
        self.inner.lock().unwrap().border_color = color;
    }
    pub fn set_fill_color(&mut self, color: NSColor) {
        self.inner.lock().unwrap().fill_color = color;
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }
    pub fn view_clone(&self) -> NSView {
        self.view.clone()
    }
}

struct NSBoxDelegate {
    state: Arc<Mutex<NSBoxState>>,
}

impl NSViewDelegate for NSBoxDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let bounds = view.bounds();

        match state.box_type {
            NSBoxType::Separator => {
                context.set_color(state.border_color);
                if bounds.width() > bounds.height() {
                    // Horizontal
                    context.fill_rect(NSRect::new(
                        bounds.min_x(),
                        bounds.mid_y() - 0.5,
                        bounds.width(),
                        1.0,
                    ));
                } else {
                    // Vertical
                    context.fill_rect(NSRect::new(
                        bounds.mid_x() - 0.5,
                        bounds.min_y(),
                        1.0,
                        bounds.height(),
                    ));
                }
            }
            NSBoxType::Primary | NSBoxType::Custom => {
                if state.border_width > 0.0 {
                    // Border (Outer)
                    context.set_color(state.border_color);
                    context.fill_rounded_rect(bounds, state.corner_radius);

                    // Fill (Inner)
                    let inner_bounds = NSRect::new(
                        bounds.min_x() + state.border_width,
                        bounds.min_y() + state.border_width,
                        bounds.width() - state.border_width * 2.0,
                        bounds.height() - state.border_width * 2.0,
                    );
                    context.set_color(state.fill_color);
                    context.fill_rounded_rect(
                        inner_bounds,
                        (state.corner_radius - state.border_width).max(0.0),
                    );
                } else {
                    // Fill only
                    context.set_color(state.fill_color);
                    context.fill_rounded_rect(bounds, state.corner_radius);
                }

                // Title (simplified implementation)
                if state.title_position != NSTitlePosition::NoTitle && !state.title.is_empty() {
                    context.set_color(NSColor::label());
                    let font_size = crate::NSLayoutConstants::FONT_SIZE_HEADER;
                    let text_pos = match state.title_position {
                        NSTitlePosition::AtTop
                        | NSTitlePosition::AboveTop
                        | NSTitlePosition::BelowTop => {
                            NSPoint::new(bounds.min_x() + crate::NSLayoutConstants::MARGIN, bounds.min_y() + 2.0)
                        }
                        _ => NSPoint::new(bounds.min_x() + crate::NSLayoutConstants::MARGIN, bounds.max_y() - font_size - 2.0),
                    };
                    context.draw_text(&state.title, text_pos, font_size);
                }
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
