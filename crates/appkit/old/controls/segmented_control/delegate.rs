//! # NSSegmentedControl Delegate
//!
//! Delegate protocol for segmented control selection.
//!
//! ## Apple Documentation
//! - [NSSegmentedControl](https://developer.apple.com/documentation/appkit/nssegmentedcontrol)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSSegmentedControl events.
///
/// Apple: NSSegmentedControl uses target-action pattern
pub trait NSSegmentedControlDelegate: Send + Sync {
    /// Called when a segment is selected.
    ///
    /// Apple: Sent via action method
    fn did_select_segment(&mut self, index: usize, label: &str) {
        let _ = (index, label);
    }
    
    /// Called to validate whether a segment should be selected.
    fn should_select_segment(&mut self, index: usize) -> bool {
        let _ = index;
        true
    }
}

/// A single segment in the control
#[derive(Clone)]
pub struct Segment {
    pub label: String,
    pub enabled: bool,
    pub width: f32,
}

/// Internal state for segmented control
pub(crate) struct SegmentedControlState {
    pub(crate) segments: Vec<Segment>,
    pub(crate) selected_index: Option<usize>,
    pub(crate) tracking_mode: TrackingMode,
}

/// Tracking mode for segmented control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackingMode {
    /// Only one segment can be selected at a time
    SelectOne,
    /// Any number of segments can be selected
    SelectAny,
    /// Momentary - segments don't stay selected
    Momentary,
}

impl Default for SegmentedControlState {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            selected_index: None,
            tracking_mode: TrackingMode::SelectOne,
        }
    }
}

/// Internal view delegate for NSSegmentedControl rendering and events.
pub(crate) struct NSSegmentedControlViewDelegate {
    pub(crate) state: Arc<Mutex<SegmentedControlState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSSegmentedControlDelegate>>>>,
}

impl NSSegmentedControlViewDelegate {
    fn segment_at_position(&self, state: &SegmentedControlState, x: f32, frame: NSRect) -> Option<usize> {
        if state.segments.is_empty() {
            return None;
        }
        
        let total_width: f32 = state.segments.iter().map(|s| s.width).sum();
        let segment_width = if total_width > 0.0 {
            frame.width() / state.segments.len() as f32
        } else {
            frame.width() / state.segments.len() as f32
        };
        
        let relative_x = x - frame.min_x();
        let index = (relative_x / segment_width) as usize;
        
        if index < state.segments.len() {
            Some(index)
        } else {
            None
        }
    }
}

impl NSViewDelegate for NSSegmentedControlViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        if state.segments.is_empty() {
            return;
        }
        
        let segment_width = frame.width() / state.segments.len() as f32;
        
        for (i, segment) in state.segments.iter().enumerate() {
            let x = frame.min_x() + i as f32 * segment_width;
            let segment_rect = NSRect::new(x, frame.min_y(), segment_width, frame.height());
            
            // Background
            let is_selected = state.selected_index == Some(i);
            let bg_color = if is_selected {
                crate::theme::color(crate::theme::keys::CLR_BG_SELECTED)
            } else {
                crate::theme::color(crate::theme::keys::CLR_BG_CONTROL)
            };
            
            context.set_color(bg_color);
            
            // Rounded corners for first and last segments
            if i == 0 && i == state.segments.len() - 1 {
                // Single segment - round both sides
                context.fill_rounded_rect(segment_rect, 6.0);
            } else if i == 0 {
                // First segment - round left side
                context.fill_rect(segment_rect);
            } else if i == state.segments.len() - 1 {
                // Last segment - round right side
                context.fill_rect(segment_rect);
            } else {
                // Middle segment - no rounding
                context.fill_rect(segment_rect);
            }
            
            // Separator (except for last segment)
            if i < state.segments.len() - 1 {
                let sep_x = x + segment_width;
                context.set_color(crate::theme::color(crate::theme::keys::CLR_SEPARATOR));
                context.fill_rect(NSRect::new(sep_x, frame.min_y(), 1.0, frame.height()));
            }
            
            // Label
            let text_color = if is_selected {
                crate::theme::color(crate::theme::keys::CLR_TEXT_SELECTED)
            } else {
                crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL)
            };
            
            context.set_color(text_color);
            let text_x = x + segment_width / 2.0 - 20.0; // Rough centering
            let text_y = frame.mid_y() - 6.0;
            context.draw_text(&segment.label, NSPoint::new(text_x, text_y), 13.0);
        }
    }
    
    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let state = self.state.lock().unwrap();
        let x = event.location_in_window().x;
        
        if let Some(index) = self.segment_at_position(&state, x, view.frame()) {
            // Check if segment is enabled
            if !state.segments[index].enabled {
                return;
            }
            
            drop(state);
            
            // Check if should select
            let should_select = if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.should_select_segment(index)
            } else {
                true
            };
            
            if !should_select {
                return;
            }
            
            // Update selection
            let mut state = self.state.lock().unwrap();
            match state.tracking_mode {
                TrackingMode::SelectOne => {
                    state.selected_index = Some(index);
                }
                TrackingMode::SelectAny => {
                    // Toggle selection (would need Vec<bool> for multiple selection)
                    state.selected_index = Some(index);
                }
                TrackingMode::Momentary => {
                    // Don't change selection
                }
            }
            
            let label = state.segments[index].label.clone();
            drop(state);
            
            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.did_select_segment(index, &label);
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
