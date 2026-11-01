//! # NSButton Drawing Implementation
//!
//! Rendering delegate for button controls.

use std::sync::{Arc, Mutex};
use crate::{NSView, NSPoint, NSColor, NSViewDelegate};
use crate::theme::{self, keys as theme_keys};
use super::types::{NSBezelStyle, NSButtonType, NSButtonState};

pub(crate) struct NSButtonViewDelegate {
    pub(crate) state: Arc<Mutex<NSButtonState>>,
    pub(crate) action: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
}

impl NSViewDelegate for NSButtonViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Determine background color based on state
        let bg_token = if state.is_pressed {
            theme_keys::CLR_BG_BUTTON_PRESSED
        } else if state.is_hovered {
            theme_keys::CLR_BG_BUTTON_HOVER
        } else {
            theme_keys::CLR_BG_BUTTON
        };
        
        let mut bg_color = theme::color_or(bg_token, NSColor::new(0.2, 0.2, 0.2, 1.0));
        
        // Override with accent color if set
        if let Some(accent) = state.accent_color {
            bg_color = accent;
        }
        
        let text_color = theme::color_or(theme_keys::CLR_TEXT_LABEL, NSColor::white());
        
        // Determine corner radius based on bezel style
        let corner_radius = match state.bezel_style {
            NSBezelStyle::Rounded | NSBezelStyle::RoundRect => 6.0,
            NSBezelStyle::Circular => frame.width().min(frame.height()) / 2.0,
            NSBezelStyle::Inline => 4.0,
            _ => 0.0,
        };
        
        // Draw background
        context.set_color(bg_color);
        if corner_radius > 0.0 {
            context.fill_rounded_rect(frame, corner_radius);
        } else {
            context.fill_rect(frame);
        }
        
        // Draw title if present
        if !state.title.is_empty() {
            context.set_color(text_color);
            let text_x = frame.min_x() + 8.0;
            let text_y = frame.mid_y() - 5.5;
            context.draw_text(&state.title, NSPoint::new(text_x, text_y), 13.0);
        }
        
        // TODO: Draw image if present
        // if let Some(ref image) = state.image {
        //     // Render image based on image_position
        // }
    }

    fn mouse_down(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_pressed = true;
        
        // Fire action immediately for momentary buttons
        if state.button_type == NSButtonType::MomentaryPushIn {
            drop(state); // Release lock before calling action
            if let Some(ref action) = *self.action.lock().unwrap() {
                action();
            }
        }
    }

    fn mouse_up(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_pressed = false;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
