//! Drawing and mouse interaction delegate for NSTextField.

use super::field_types::{NSTextAlignment, NSTextFieldState};
use super::super::{NSPoint, NSView};
use std::sync::{Arc, Mutex};

/// Drawing and interaction delegate
pub(super) struct NSTextFieldDelegate {
    pub(super) state: Arc<Mutex<NSTextFieldState>>,
}

impl super::super::view::NSViewDelegate for NSTextFieldDelegate {
    fn draw(&self, view: &NSView, context: &mut super::super::render::RenderContext) {
        use crate::theme::{color, f32_or, keys};
        let state = self.state.lock().unwrap();
        let bounds = view.bounds();
        let radius = f32_or(keys::RADIUS_CONTROL, 4.0);
        let font_sz = state.font_size;
        let pad_x = f32_or(keys::SPACING_MD, 8.0);

        if state.bordered {
            let bg = color(keys::CLR_BG_CONTROL);
            context.set_color(bg);
            context.fill_rounded_rect(bounds, radius);
            let border = if state.is_scrubbing {
                color(keys::CLR_ACCENT)
            } else {
                color(keys::CLR_BORDER)
            };
            context.set_color(border);
        }

        context.set_color(state.text_color);
        let text_y = bounds.mid_y() - font_sz * 0.5;

        // Calculate text x position based on alignment
        let text_width = crate::render::text::measure_text(&state.text, font_sz);
        let text_x = match state.alignment {
            NSTextAlignment::Center => {
                let container_width = bounds.width();
                bounds.min_x() + (container_width - text_width) / 2.0
            }
            NSTextAlignment::Right => bounds.max_x() - text_width - pad_x,
            _ => {
                // Left, Justified, Natural — default left-aligned
                bounds.min_x() + pad_x
            }
        };

        context.draw_text(&state.text, NSPoint::new(text_x, text_y), font_sz);
    }

    fn mouse_down(&mut self, _view: &mut NSView, event: &super::super::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_scrubbing = true;

        let mouse_x = event.location_in_window.x;
        state.scrub_start_pos = mouse_x;

        // Try parse as CSV/float block
        if let Ok(val) = state
            .text
            .split(',')
            .next()
            .unwrap_or("")
            .trim()
            .parse::<f32>()
        {
            state.scrub_start_value = val;
        } else {
            state.scrub_start_value = 0.0;
        }
    }

    fn mouse_dragged(&mut self, _view: &mut NSView, event: &super::super::event::NSEvent) {
        let is_scrubbing;
        let mut text_snapshot = String::new();
        let mut cb: Option<Arc<dyn Fn(String) + Send + Sync>> = None;

        {
            let mut state = self.state.lock().unwrap();
            is_scrubbing = state.is_scrubbing;
            if is_scrubbing {
                let mouse_x = event.location_in_window.x;
                let delta = (mouse_x - state.scrub_start_pos) * 0.1; // scale drag
                let new_val = state.scrub_start_value + delta;

                let parts: Vec<&str> = state.text.split(',').collect();
                let rest = if parts.len() > 1 {
                    parts[1..].join(",")
                } else {
                    "".to_string()
                };

                state.text = if rest.is_empty() {
                    format!("{:.1}", new_val)
                } else {
                    format!("{:.1},{}", new_val, rest)
                };

                text_snapshot = state.text.clone();
                cb = state.action.clone();
            }
        }

        if is_scrubbing {
            if let Some(act) = cb {
                act(text_snapshot);
            }
        }
    }

    fn mouse_up(&mut self, _view: &mut NSView, _event: &super::super::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_scrubbing = false;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
