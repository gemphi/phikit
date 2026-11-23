//! # NSSegmentedControl — Segmented Button Control
//!
//! A control that displays a horizontal sequence of segments, each of which can behave like a button.
//!
//! ## Apple Documentation
//! - [NSSegmentedControl](https://developer.apple.com/documentation/appkit/nssegmentedcontrol)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut control = NSSegmentedControl::new(frame);
//! control.set_segment_count(3);
//! control.set_label(0, "Left");
//! control.set_label(1, "Center");
//! control.set_label(2, "Right");
//! control.set_action(|index| println!("Selected: {}", index));
//! ```
//!
//! ## See Also
//! - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)
//! - [NSToolbar](https://developer.apple.com/documentation/appkit/nstoolbar)

mod delegate;

pub use delegate::NSSegmentedControlDelegate;

use crate::control::NSControl;
use crate::event::NSEvent;
use crate::graphics::{NSColor, NSPoint, NSRect};
use crate::render::RenderContext;
use crate::view::{NSView, NSViewDelegate};
use delegate::{NSSegmentedControlViewDelegate, SegmentedControlState};
use std::sync::{Arc, Mutex};

/// Segement state
#[derive(Debug, Clone)]
struct Segment {
    label: String,
    width: f32,
    enabled: bool,
    selected: bool,
}

struct NSSegmentedControlStateInternal {
    segments: Vec<Segment>,
    selected_segment: isize,
    action: Option<Arc<dyn Fn(isize) + Send + Sync>>,
}

#[derive(Clone)]
pub struct NSSegmentedControl {
    control: NSControl,
    inner: Arc<Mutex<NSSegmentedControlStateInternal>>,
    state: Arc<Mutex<SegmentedControlState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSSegmentedControlDelegate>>>>,
}

struct NSSegmentedControlInternalDelegate {
    state: Arc<Mutex<NSSegmentedControlStateInternal>>,
}

impl NSViewDelegate for NSSegmentedControlInternalDelegate {
    fn draw(&self, view: &NSView, context: &mut RenderContext) {
        let state = self.state.lock().unwrap();
        let bounds = view.bounds();
        let radius = crate::theme::f32_or(crate::theme::keys::RADIUS_BUTTON, 6.0);
        let inset = 3.0;

        // Background pill
        let bg_color = crate::theme::color(crate::theme::keys::CLR_BG_CONTROL);
        context.set_color(bg_color);
        context.fill_rounded_rect(bounds, radius + 2.0);

        // Draw segments
        if state.segments.is_empty() {
            return;
        }
        let segment_width = bounds.width() / state.segments.len() as f32;

        for (i, segment) in state.segments.iter().enumerate() {
            let seg_x = i as f32 * segment_width;

            // Selection highlight
            if state.selected_segment == i as isize {
                let sel_color = crate::theme::color(crate::theme::keys::CLR_ACCENT);
                context.set_color(sel_color);
                let sel_rect = NSRect::new(
                    seg_x + inset,
                    inset,
                    segment_width - inset * 2.0,
                    bounds.height() - inset * 2.0,
                );
                context.fill_rounded_rect(sel_rect, radius);
            }

            // Label — vertically centered
            let text_color = if state.selected_segment == i as isize {
                NSColor::white()
            } else {
                crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL)
            };

            let font_sz = 12.0;
            let char_w = 7.0;
            context.set_color(text_color);
            let text_w = segment.label.len() as f32 * char_w;
            let text_x = seg_x + (segment_width - text_w) / 2.0;
            let text_y = (bounds.height() - font_sz) / 2.0;
            context.draw_text(&segment.label, NSPoint::new(text_x, text_y), font_sz);
        }
    }

    fn mouse_down(&mut self, view: &mut NSView, event: &NSEvent) {
        let frame = view.frame();
        let mouse_x = event.location_in_window.x;

        let mut state = self.state.lock().unwrap();
        if state.segments.is_empty() {
            return;
        }

        let segment_width = frame.width() / state.segments.len() as f32;
        let clicked_index = (mouse_x / segment_width).floor() as isize;

        if clicked_index >= 0 && clicked_index < state.segments.len() as isize {
            state.selected_segment = clicked_index;

            let action_clone = state.action.clone();
            drop(state); // drop lock before callback

            if let Some(action) = action_clone {
                action(clicked_index);
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

impl NSSegmentedControl {
    pub fn new(frame: NSRect) -> Self {
        let inner = Arc::new(Mutex::new(NSSegmentedControlStateInternal {
            segments: Vec::new(),
            selected_segment: -1,
            action: None,
        }));
        
        // Initialize delegate infrastructure following the delegate pattern
        let state = Arc::new(Mutex::new(SegmentedControlState::default()));
        let delegate = Arc::new(Mutex::new(None));

        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(NSSegmentedControlViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });

        Self { 
            control, 
            inner, 
            state, 
            delegate 
        }
    }

    pub fn set_segment_count(&mut self, count: isize) {
        let mut state = self.inner.lock().unwrap();
        state.segments.clear();
        for _ in 0..count {
            state.segments.push(Segment {
                label: String::new(),
                width: 0.0,
                enabled: true,
                selected: false,
            });
        }
    }

    pub fn set_label(&mut self, label: &str, for_segment: isize) {
        let mut state = self.inner.lock().unwrap();
        if for_segment >= 0 && for_segment < state.segments.len() as isize {
            state.segments[for_segment as usize].label = label.to_string();
        }
    }

    pub fn set_selected_segment(&mut self, segment: isize) {
        self.inner.lock().unwrap().selected_segment = segment;
    }

    pub fn selected_segment(&self) -> isize {
        self.inner.lock().unwrap().selected_segment
    }

    pub fn set_action(&mut self, action: impl Fn(isize) + Send + Sync + 'static) {
        self.inner.lock().unwrap().action = Some(Arc::new(action));
    }

    /// Set the delegate for segmented control events.
    ///
    /// Apple: `setTarget:action:` (we use delegate pattern)
    pub fn set_delegate(&mut self, delegate: Box<dyn NSSegmentedControlDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    pub fn view(&self) -> &NSView {
        self.control.view()
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        self.control.view_mut()
    }

    pub fn view_clone(&self) -> NSView {
        self.control.view().clone()
    }
}
