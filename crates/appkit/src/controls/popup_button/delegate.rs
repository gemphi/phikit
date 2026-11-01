//! # NSPopUpButton Delegate
//!
//! Delegate protocol for popup button menu selection.
//!
//! ## Apple Documentation
//! - [NSPopUpButton](https://developer.apple.com/documentation/appkit/nspopupbutton)
//! - [NSMenu](https://developer.apple.com/documentation/appkit/nsmenu)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSPopUpButton events.
///
/// Apple: NSPopUpButton uses target-action pattern, but we provide delegate for consistency
pub trait NSPopUpButtonDelegate: Send + Sync {
    /// Called when the popup menu is about to be displayed.
    ///
    /// Apple: Equivalent to `willPopUp` notification
    fn will_pop_up(&mut self) {}
    
    /// Called when a menu item is selected.
    ///
    /// Apple: Sent via action method
    /// 
    /// # Parameters
    /// - `index`: The index of the selected item
    /// - `title`: The title of the selected item
    fn did_select_item(&mut self, index: usize, title: &str) {
        let _ = (index, title);
    }
}

/// Internal state for popup button
pub(crate) struct PopUpButtonState {
    pub(crate) items: Vec<String>,
    pub(crate) selected_index: usize,
    pub(crate) is_open: bool,
}

impl Default for PopUpButtonState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            is_open: false,
        }
    }
}

/// Internal view delegate for NSPopUpButton rendering and events.
pub(crate) struct NSPopUpButtonViewDelegate {
    pub(crate) state: Arc<Mutex<PopUpButtonState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSPopUpButtonDelegate>>>>,
}

impl NSPopUpButtonViewDelegate {
    fn item_at_position(&self, state: &PopUpButtonState, y: f32, bounds: NSRect) -> Option<usize> {
        if !state.is_open || state.items.is_empty() {
            return None;
        }
        
        let item_h = crate::theme::f32_or(crate::theme::keys::SIZE_CONTROL_HEIGHT, 24.0);
        let menu_y = bounds.height();
        
        if y < menu_y {
            return None;
        }
        
        let idx = ((y - menu_y) / item_h) as usize;
        
        if idx < state.items.len() {
            Some(idx)
        } else {
            None
        }
    }
}

impl NSViewDelegate for NSPopUpButtonViewDelegate {
    fn draw(&self, view: &NSView, ctx: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let bounds = view.bounds();

        let radius = crate::theme::f32_or(crate::theme::keys::RADIUS_BUTTON, 6.0);
        let font_size = crate::theme::f32_or(crate::theme::keys::SIZE_FONT_UI, 12.0);
        let padding = crate::theme::f32_or(crate::theme::keys::SPACING_MD, 8.0);

        // Button background
        ctx.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CONTROL));
        ctx.fill_rounded_rect(bounds, radius);

        // Border
        ctx.draw_rounded_rect_border(
            bounds,
            radius,
            1.0,
            crate::theme::color(crate::theme::keys::CLR_BORDER),
        );

        // Selected title
        let title = state
            .items
            .get(state.selected_index)
            .map(|s| s.as_str())
            .unwrap_or("—");
        ctx.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
        ctx.draw_text(
            title,
            NSPoint::new(padding, (bounds.height() - font_size) / 2.0),
            font_size,
        );

        // Chevron indicator (right side)
        let chevron_x = bounds.width() - padding - 8.0;
        let chevron_y = (bounds.height() - 6.0) / 2.0;
        ctx.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_TERTIARY));
        ctx.fill_rect(NSRect::new(chevron_x, chevron_y, 8.0, 2.0));
        ctx.fill_rect(NSRect::new(chevron_x + 2.0, chevron_y + 2.0, 4.0, 2.0));

        // Dropdown menu (when open)
        if state.is_open {
            let item_h = crate::theme::f32_or(crate::theme::keys::SIZE_CONTROL_HEIGHT, 24.0);
            let menu_h = state.items.len() as f32 * item_h;
            let menu_y = bounds.height();

            ctx.set_color(crate::theme::color(crate::theme::keys::CLR_BG_DROPDOWN));
            ctx.fill_rounded_rect(NSRect::new(0.0, menu_y, bounds.width(), menu_h), radius);

            for (i, item) in state.items.iter().enumerate() {
                let iy = menu_y + i as f32 * item_h;
                if i == state.selected_index {
                    ctx.set_color(crate::theme::color(crate::theme::keys::CLR_BG_SELECTED));
                    ctx.fill_rect(NSRect::new(0.0, iy, bounds.width(), item_h));
                }
                ctx.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
                ctx.draw_text(
                    item,
                    NSPoint::new(padding, iy + (item_h - font_size) / 2.0),
                    font_size,
                );
            }
        }
    }

    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        
        if state.is_open {
            // Click while open — select item
            let bounds = view.bounds();
            let pos = event.location_in_window();
            
            if let Some(idx) = self.item_at_position(&state, pos.y, bounds) {
                state.selected_index = idx;
                let title = state.items[idx].clone();
                state.is_open = false;
                drop(state);
                
                // Notify delegate
                if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                    delegate.did_select_item(idx, &title);
                }
            } else {
                state.is_open = false;
            }
        } else {
            // Opening the menu
            state.is_open = true;
            drop(state);
            
            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.will_pop_up();
            }
        }
        
        view.set_needs_display();
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
