//! Menu Bar View Definition
//!
//! this file defines the MenuBarView struct which provides the primary
//! UI container for the application's top-level menu bar.
//! coordinates with the MenuBarDelegate for rendering and event handling.

use super::delegate::{MenuBarDelegate, MENU_PADDING_X, MENU_START_X};
use super::MenuBar;
use crate::{NSPoint, NSRect, NSView};
use std::sync::{Arc, Mutex};

/// A view that displays a horizontal list of menus at the top of a window.
///
/// mirrors Apple's native menu bar appearance and behavior.
/// manages a delegate that handles drawing and interactive dropdowns.
pub struct MenuBarView {
    pub(crate) view: NSView,
}

impl MenuBarView {
    /// Creates a new MenuBarView with the given data and frame.
    /// Only contains menu items — nothing else. The toolbar adds other items around it.
    pub fn new(menubar: MenuBar, frame: NSRect) -> Self {
        let mut view = NSView::new(frame).with_name("MenuBar");
        view.set_background_color(None); // Transparent — toolbar provides the background
        let menus = menubar.menus;
        let active_idx = Arc::new(Mutex::new(None::<usize>));
        view.set_delegate(MenuBarDelegate {
            menus,
            hover_idx: Mutex::new(None),
            active_idx,
        });
        Self { view }
    }

    /// Returns a shared reference to the underlying NSView.
    /// used to add the menu bar to a window's view hierarchy.
    pub fn view(&self) -> &NSView {
        &self.view
    }

    /// Hit tests a point against the menu bar items.
    ///
    /// Returns `Some(index)` if the point falls within the bounds of a menu item,
    /// `None` if the point is outside all menu item bounds (in gaps or outside frame).
    ///
    /// The point is expected in the menu bar's local coordinate space.
    pub fn hit_test(&self, point: NSPoint) -> Option<usize> {
        let frame = self.view.frame();

        // Check if point is within the menu bar's vertical bounds
        if point.y < 0.0 || point.y > frame.size.height {
            return None;
        }

        // Access the delegate to get menu item data
        if let Some(delegate_arc) = self.view.delegate() {
            if let Ok(delegate) = delegate_arc.lock() {
                if let Some(menu_delegate) = delegate.as_any().downcast_ref::<MenuBarDelegate>() {
                    return Self::hit_test_items(&menu_delegate.menus, point.x);
                }
            }
        }

        None
    }

    /// Calculates which menu item index the given x coordinate falls within.
    ///
    /// Iterates through menu items accumulating their widths from MENU_START_X.
    /// Each item's width is calculated as: title_len * 8.0 + MENU_PADDING_X * 2.0
    fn hit_test_items(menus: &[super::Menu], x: f32) -> Option<usize> {
        let mut accumulated_x = MENU_START_X;

        for (index, menu) in menus.iter().enumerate() {
            let item_width = menu.title.len() as f32 * 8.0 + MENU_PADDING_X * 2.0;

            if x >= accumulated_x && x < accumulated_x + item_width {
                return Some(index);
            }

            accumulated_x += item_width;
        }

        // Cursor is outside all menu item bounds (in leading space or trailing space)
        None
    }

    /// Processes a MouseMoved event, forwarding it to the delegate to update hover state.
    ///
    /// Returns `true` if the event was handled (point is within the menu bar bounds),
    /// `false` if the point is outside the menu bar's vertical bounds.
    pub fn mouse_moved(&mut self, event: &crate::event::NSEvent) -> bool {
        let pos = event.location_in_window();
        let bar_h = self.view.frame().size.height;

        // Only handle if within the menu bar's vertical bounds
        if pos.y > bar_h {
            return false;
        }

        if let Some(delegate_arc) = self.view.delegate() {
            if let Ok(mut d) = delegate_arc.lock() {
                d.mouse_moved(&mut self.view, event);
            }
        }
        true
    }

    /// Processes a mouse down event, forwarding it to the delegate to activate a menu dropdown.
    ///
    /// Returns `true` if the click was on a menu item and the dropdown was toggled,
    /// `false` if the click was outside all menu item bounds.
    pub fn mouse_down(&mut self, event: &crate::event::NSEvent) -> bool {
        let pos = event.location_in_window();
        let bar_h = self.view.frame().size.height;

        // Only handle if within the menu bar's vertical bounds
        if pos.y > bar_h {
            return false;
        }

        // Check if the click is over a menu item
        if let Some(delegate_arc) = self.view.delegate() {
            if let Ok(delegate) = delegate_arc.lock() {
                if let Some(menu_delegate) = delegate.as_any().downcast_ref::<MenuBarDelegate>() {
                    if Self::hit_test_items(&menu_delegate.menus, pos.x).is_some() {
                        drop(delegate);
                        if let Ok(mut d) = delegate_arc.lock() {
                            d.mouse_down(&mut self.view, event);
                        }
                        return true;
                    }
                }
            }
        }
        false
    }
}
