//! # NSPathControl Delegate
//!
//! Delegate protocol for path control events and customization.
//!
//! ## Apple Documentation
//! - [NSPathControl](https://developer.apple.com/documentation/appkit/nspathcontrol)
//! - [NSPathControlDelegate](https://developer.apple.com/documentation/appkit/nspathcontroldelegate)

use crate::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{NSPathComponent, NSPathStyle};

/// Delegate protocol for NSPathControl events.
///
/// Apple: [NSPathControlDelegate](https://developer.apple.com/documentation/appkit/nspathcontroldelegate)
pub trait NSPathControlDelegate: Send + Sync {
    /// Called when the path changes.
    ///
    /// Apple: Sent via action method
    fn path_did_change(&mut self, path: &Path) {
        let _ = path;
    }
    
    /// Called before displaying the open panel for path selection.
    ///
    /// Allows customization of the open panel before it's shown.
    ///
    /// Apple: `pathControl:willDisplayOpenPanel:`
    fn will_display_open_panel(&mut self, open_panel: &mut crate::open_panel::NSOpenPanel) {
        let _ = open_panel;
    }
    
    /// Called when a path component is clicked.
    ///
    /// Return true to allow navigation, false to prevent it.
    ///
    /// Apple: `pathControl:shouldDragPathComponentCell:withPasteboard:`
    fn should_navigate_to_component(&mut self, component: &NSPathComponent) -> bool {
        let _ = component;
        true
    }
}

/// Internal state for path control
pub(crate) struct PathControlState {
    pub(crate) path: PathBuf,
    pub(crate) components: Vec<NSPathComponent>,
    pub(crate) style: NSPathStyle,
    pub(crate) is_editable: bool,
    pub(crate) background_color: Option<crate::graphics::NSColor>,
    pub(crate) placeholder: Option<String>,
    pub(crate) hovered_component: Option<usize>,
    pub(crate) is_showing_menu: bool,
}

impl Default for PathControlState {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            components: Vec::new(),
            style: NSPathStyle::Standard,
            is_editable: true,
            background_color: None,
            placeholder: None,
            hovered_component: None,
            is_showing_menu: false,
        }
    }
}

/// Internal view delegate for NSPathControl rendering and events.
pub(crate) struct NSPathControlViewDelegate {
    pub(crate) state: Arc<Mutex<PathControlState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSPathControlDelegate>>>>,
}

impl NSPathControlViewDelegate {
    /// Calculate the width needed for a component label
    fn component_width(&self, title: &str) -> f64 {
        // Approximate width: 8 pixels per character + padding
        (title.len() as f64 * 8.0) + 16.0
    }
    
    /// Get the component at a given x position
    fn component_at_position(&self, x: f64, state: &PathControlState) -> Option<usize> {
        let mut current_x = 8.0;
        
        for (index, component) in state.components.iter().enumerate() {
            let width = self.component_width(&component.title);
            
            if x >= current_x && x < current_x + width {
                return Some(index);
            }
            
            current_x += width + 8.0; // Component width + separator width
        }
        
        None
    }
}

impl NSViewDelegate for NSPathControlViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();
        
        // Draw background
        let bg_color = state.background_color.unwrap_or_else(|| {
            crate::theme::color(crate::theme::keys::CLR_BG_CONTROL)
        });
        context.set_color(bg_color);
        context.fill_rounded_rect(frame, 4.0);
        
        // Draw border
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BORDER));
        context.stroke_rounded_rect(frame, 4.0, 1.0);
        
        // Draw content based on style
        match state.style {
            NSPathStyle::Standard | NSPathStyle::NavigationBar => {
                self.draw_breadcrumb_path(&state, frame, context);
            }
            NSPathStyle::PopUp => {
                self.draw_popup_path(&state, frame, context);
            }
        }
    }
    
    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        
        if !state.is_editable {
            return;
        }
        
        let frame = view.frame();
        let click_pos = event.location_in_window();
        let relative_x = click_pos.x - frame.min_x();
        
        // Check if a component was clicked
        if let Some(component_index) = self.component_at_position(relative_x as f64, &state) {
            let component = state.components[component_index].clone();
            
            // Check with delegate if navigation is allowed
            let should_navigate = if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.should_navigate_to_component(&component)
            } else {
                true
            };
            
            if should_navigate {
                // Update path to the clicked component
                state.path = component.path.clone();
                let new_path = state.path.clone();
                drop(state);
                
                // Notify delegate of path change
                if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.path_did_change(&new_path);
                }
            }
        } else {
            // Clicked in empty area - show open panel if editable
            if state.is_editable {
                drop(state);
                self.show_open_panel(view);
            }
        }
    }
    
    fn mouse_moved(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        let frame = view.frame();
        let mouse_pos = event.location_in_window();
        let relative_x = mouse_pos.x - frame.min_x();
        
        // Update hovered component
        state.hovered_component = self.component_at_position(relative_x as f64, &state);
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl NSPathControlViewDelegate {
    /// Draw breadcrumb-style path
    fn draw_breadcrumb_path(
        &self,
        state: &PathControlState,
        frame: NSRect,
        context: &mut crate::render::RenderContext,
    ) {
        if state.components.is_empty() {
            // Draw placeholder if present
            if let Some(ref placeholder) = state.placeholder {
                let text_color = crate::theme::color(crate::theme::keys::CLR_TEXT_SECONDARY);
                context.set_color(text_color);
                
                let text_x = frame.min_x() + 8.0;
                let text_y = frame.mid_y() - 6.0;
                
                context.draw_text(placeholder, NSPoint::new(text_x, text_y), 14.0);
            }
            return;
        }
        
        let mut current_x = frame.min_x() + 8.0;
        let text_y = frame.mid_y() - 6.0;
        
        for (index, component) in state.components.iter().enumerate() {
            // Determine if this component is hovered
            let is_hovered = state.hovered_component == Some(index);
            
            // Draw component text
            let text_color = if is_hovered {
                crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL)
            } else {
                crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL)
            };
            context.set_color(text_color);
            
            context.draw_text(&component.title, NSPoint::new(current_x, text_y), 14.0);
            
            let component_width = self.component_width(&component.title);
            current_x += component_width as f32;
            
            // Draw separator (chevron) if not the last component
            if index < state.components.len() - 1 {
                let separator_color = crate::theme::color(crate::theme::keys::CLR_TEXT_SECONDARY);
                context.set_color(separator_color);
                context.draw_text("›", NSPoint::new(current_x, text_y), 14.0);
                current_x += 8.0;
            }
        }
    }
    
    /// Draw popup-style path (only last component visible)
    fn draw_popup_path(
        &self,
        state: &PathControlState,
        frame: NSRect,
        context: &mut crate::render::RenderContext,
    ) {
        if state.components.is_empty() {
            // Draw placeholder if present
            if let Some(ref placeholder) = state.placeholder {
                let text_color = crate::theme::color(crate::theme::keys::CLR_TEXT_SECONDARY);
                context.set_color(text_color);
                
                let text_x = frame.min_x() + 8.0;
                let text_y = frame.mid_y() - 6.0;
                
                context.draw_text(placeholder, NSPoint::new(text_x, text_y), 14.0);
            }
            return;
        }
        
        // Draw only the last component
        if let Some(last_component) = state.components.last() {
            let text_color = crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL);
            context.set_color(text_color);
            
            let text_x = frame.min_x() + 8.0;
            let text_y = frame.mid_y() - 6.0;
            
            context.draw_text(&last_component.title, NSPoint::new(text_x, text_y), 14.0);
            
            // Draw dropdown arrow
            let arrow_x = frame.max_x() - 20.0;
            let arrow_color = crate::theme::color(crate::theme::keys::CLR_TEXT_SECONDARY);
            context.set_color(arrow_color);
            context.draw_text("▼", NSPoint::new(arrow_x, text_y), 10.0);
        }
    }
    
    /// Show open panel for path selection
    fn show_open_panel(&mut self, _view: &mut NSView) {
        // Create and configure open panel
        let mut open_panel = crate::open_panel::NSOpenPanel::new();
        open_panel.set_can_choose_directories(true);
        open_panel.set_can_choose_files(false);
        
        // Allow delegate to customize open panel
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.will_display_open_panel(&mut open_panel);
        }
        
        // Show panel and handle result
        if let Some(selected_path) = open_panel.run_modal() {
            let mut state = self.state.lock().unwrap();
            state.path = selected_path.clone();
            state.components = super::NSPathControl::path_to_components(&selected_path);
            drop(state);
            
            // Notify delegate of path change
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.path_did_change(&selected_path);
            }
        }
    }
}
