//! # NSDatePicker Delegate
//!
//! Delegate protocol for date picker value changes.
//!
//! ## Apple Documentation
//! - [NSDatePicker](https://developer.apple.com/documentation/appkit/nsdatepicker)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)

use crate::prelude::*;
use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};

use super::{NSDatePickerElementFlags, NSDatePickerMode, NSDatePickerStyle};

/// Delegate protocol for NSDatePicker events.
///
/// Apple: NSDatePicker uses target-action pattern, but we provide delegate for consistency
pub trait NSDatePickerDelegate: Send + Sync {
    /// Called when the selected date changes.
    ///
    /// Apple: Sent via action method
    fn date_did_change(&mut self, date: DateTime<Local>) {
        let _ = date;
    }
    
    /// Called when the date range changes (for range mode).
    ///
    /// Apple: Sent via action method
    fn date_range_did_change(&mut self, start: DateTime<Local>, end: DateTime<Local>) {
        let _ = (start, end);
    }
}

/// Internal state for date picker
pub(crate) struct DatePickerState {
    pub(crate) date: DateTime<Local>,
    pub(crate) end_date: Option<DateTime<Local>>,
    pub(crate) mode: NSDatePickerMode,
    pub(crate) style: NSDatePickerStyle,
    pub(crate) elements: NSDatePickerElementFlags,
    pub(crate) min_date: Option<DateTime<Local>>,
    pub(crate) max_date: Option<DateTime<Local>>,
    pub(crate) is_editing: bool,
}

impl Default for DatePickerState {
    fn default() -> Self {
        Self {
            date: Local::now(),
            end_date: None,
            mode: NSDatePickerMode::Single,
            style: NSDatePickerStyle::TextFieldAndStepper,
            elements: NSDatePickerElementFlags::default(),
            min_date: None,
            max_date: None,
            is_editing: false,
        }
    }
}

/// Internal view delegate for NSDatePicker rendering and events.
pub(crate) struct NSDatePickerViewDelegate {
    pub(crate) state: Arc<Mutex<DatePickerState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSDatePickerDelegate>>>>,
}

impl NSDatePickerViewDelegate {
    /// Format date according to element flags
    fn format_date(&self, date: &DateTime<Local>, elements: &NSDatePickerElementFlags) -> String {
        let mut parts = Vec::new();
        
        if elements.year_month_day {
            parts.push(date.format("%Y-%m-%d").to_string());
        }
        
        if elements.hour_minute {
            if elements.second {
                parts.push(date.format("%H:%M:%S").to_string());
            } else {
                parts.push(date.format("%H:%M").to_string());
            }
        }
        
        if elements.time_zone {
            parts.push(date.format("%Z").to_string());
        }
        
        if parts.is_empty() {
            date.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            parts.join(" ")
        }
    }
    
    /// Clamp date to min/max bounds
    fn clamp_date(&self, date: DateTime<Local>, state: &DatePickerState) -> DateTime<Local> {
        let mut clamped = date;
        
        if let Some(min) = state.min_date {
            if clamped < min {
                clamped = min;
            }
        }
        
        if let Some(max) = state.max_date {
            if clamped > max {
                clamped = max;
            }
        }
        
        clamped
    }
}

impl NSViewDelegate for NSDatePickerViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Draw background
        let bg_color = if state.is_editing {
            crate::theme::color(crate::theme::keys::CLR_BG_CONTROL)
        } else {
            crate::theme::color(crate::theme::keys::CLR_BG_CONTROL)
        };
        context.set_color(bg_color);
        context.fill_rounded_rect(frame, 4.0);
        
        // Draw border
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BORDER));
        context.stroke_rounded_rect(frame, 4.0, 1.0);
        
        // Format and draw date text
        let date_text = match state.mode {
            NSDatePickerMode::Single => {
                self.format_date(&state.date, &state.elements)
            }
            NSDatePickerMode::Range => {
                if let Some(end) = state.end_date {
                    format!(
                        "{} - {}",
                        self.format_date(&state.date, &state.elements),
                        self.format_date(&end, &state.elements)
                    )
                } else {
                    self.format_date(&state.date, &state.elements)
                }
            }
        };
        
        // Draw text
        let text_color = crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL);
        context.set_color(text_color);
        
        let text_x = frame.min_x() + 8.0;
        let text_y = frame.mid_y() - 6.0; // Approximate vertical centering
        
        context.draw_text(&date_text, NSPoint::new(text_x, text_y), 14.0);
        
        // Draw stepper arrows if style includes stepper
        if matches!(state.style, NSDatePickerStyle::TextFieldAndStepper) {
            let arrow_x = frame.max_x() - 20.0;
            
            // Up arrow
            let up_y = frame.mid_y() - 8.0;
            context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_SECONDARY));
            context.draw_text("▲", NSPoint::new(arrow_x, up_y), 10.0);
            
            // Down arrow
            let down_y = frame.mid_y() + 2.0;
            context.draw_text("▼", NSPoint::new(arrow_x, down_y), 10.0);
        }
    }
    
    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_editing = true;
        
        let frame = view.frame();
        let click_pos = event.location_in_window();
        
        // Check if clicked on stepper arrows
        if matches!(state.style, NSDatePickerStyle::TextFieldAndStepper) {
            let arrow_x = frame.max_x() - 20.0;
            let arrow_width = 20.0;
            
            if click_pos.x >= arrow_x && click_pos.x <= arrow_x + arrow_width {
                let mid_y = frame.mid_y();
                
                // Determine if up or down arrow was clicked
                let increment_days = if click_pos.y < mid_y {
                    -1 // Down arrow
                } else {
                    1 // Up arrow
                };
                
                // Increment/decrement date
                use chrono::Duration;
                let new_date = state.date + Duration::days(increment_days);
                let clamped_date = self.clamp_date(new_date, &state);
                state.date = clamped_date;
                
                let date = state.date;
                drop(state);
                
                // Notify delegate
                if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.date_did_change(date);
                }
                
                return;
            }
        }
        
        // Otherwise, open date picker UI (simplified for now)
        // In a full implementation, this would open a calendar popup
    }
    
    fn mouse_up(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_editing = false;
    }
    
    fn key_down(&mut self, _view: &mut NSView, event: &crate::event::NSEvent) {
        use chrono::Duration;
        
        // Handle arrow keys for date navigation using characters
        if let Some(chars) = event.characters() {
            let mut state = self.state.lock().unwrap();
            
            let increment = match chars.as_str() {
                "ArrowUp" => Some(1),
                "ArrowDown" => Some(-1),
                _ => None,
            };
            
            if let Some(days) = increment {
                let new_date = state.date + Duration::days(days);
                let clamped_date = self.clamp_date(new_date, &state);
                state.date = clamped_date;
                
                let date = state.date;
                drop(state);
                
                // Notify delegate
                if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.date_did_change(date);
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
