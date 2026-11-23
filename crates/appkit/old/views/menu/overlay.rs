//! Menu Overlay Delegate Implementation
//!
//! this file implements the delegate for the menu dismissal overlay.
//! it handles clicks outside of active dropdowns to automatically close the menu.
//! mirrors Apple's native dismissal behavior for popover-style interfaces.

use std::sync::{Arc, Mutex};
use crate::{NSView, NSViewDelegate, NSPoint, NSEvent};
use super::delegate::MENU_HEIGHT;

/// Internal delegate that manages the full-screen dismissal overlay.
///
/// tracks the active menu index to coordinate its dismissal.
/// intercepts mouse events to determine if the user clicked outside the menu area.
pub struct MenuOverlayDelegate { 
    pub(crate) active_idx: Arc<Mutex<Option<usize>>> 
}

impl NSViewDelegate for MenuOverlayDelegate {
    /// Handles mouse clicks on the overlay to dismiss active menus.
    /// only triggers dismissal if the click is below the menu bar threshold.
    fn mouse_down(&mut self, _view: &mut NSView, event: &NSEvent) {
        let pos = event.location_in_window();
        if pos.y > MENU_HEIGHT {
            *self.active_idx.lock().unwrap() = None;
            if let Ok(mut app) = crate::application::NSApplication::shared().try_lock() {
                app.pending_actions.push(Box::new(|root| {
                    root.subviews_mut().retain(|v| !v.name.starts_with("MenuOverlay") && !v.name.starts_with("MenuSubmenu"));
                }));
            }
        }
    }

    /// Determines if the overlay should capture a specific event.
    /// ignores events within the menu bar height to allow normal interaction.
    fn hit_test(&self, _view: &NSView, point: NSPoint) -> bool {
        point.y > MENU_HEIGHT
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
