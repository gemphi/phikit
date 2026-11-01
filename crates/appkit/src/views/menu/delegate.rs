//! Menu Bar Delegate Implementation
//!
//! this file implements the view delegate for the main application menu bar.
//! it handles drawing menu titles, tracking hover/active states, and
//! coordinating the display of dropdown overlays during user interaction.

use super::{Menu, MenuDropdownDelegate, MenuOverlayDelegate};
use crate::render::RenderContext;
use crate::{NSColor, NSEvent, NSPoint, NSRect, NSView, NSViewDelegate};
use std::sync::{Arc, Mutex};

pub const MENU_START_X: f32 = crate::NSLayoutConstants::MENU_BAR_START_X;
pub const MENU_PADDING_X: f32 = crate::NSLayoutConstants::MENU_BAR_PADDING_X;
pub const MENU_HEIGHT: f32 = crate::NSLayoutConstants::MENU_BAR_BUTTON_HEIGHT;

/// Compute the pixel width of a menu bar button from its title.
/// Uses actual font metrics for accurate per-character width.
fn menu_btn_width(title: &str) -> f32 {
    crate::render::text::measure_text(title, crate::NSLayoutConstants::MENU_BAR_FONT_SIZE)
        + MENU_PADDING_X * 2.0
}

/// Compute the dropdown height from item count.
fn dropdown_height(item_count: usize) -> f32 {
    item_count as f32 * crate::NSLayoutConstants::MENU_DROPDOWN_ITEM_HEIGHT
        + crate::NSLayoutConstants::MENU_DROPDOWN_PADDING_Y * 2.0
}

/// Internal delegate that drives the main menu bar rendering and interaction.
///
/// tracks the available menus and their interactive states (hover/active).
/// coordinates the creation of dropdown views when a menu title is clicked.
pub struct MenuBarDelegate {
    pub(crate) menus: Vec<Menu>,
    pub(crate) hover_idx: Mutex<Option<usize>>,
    pub(crate) active_idx: Arc<Mutex<Option<usize>>>,
}

impl NSViewDelegate for MenuBarDelegate {
    /// Draws the top-level menu titles with hover and active highlighting.
    /// calculates dynamic widths based on label lengths and theme padding.
    fn draw(&self, view: &NSView, ctx: &mut RenderContext) {
        use crate::NSLayoutConstants as L;
        let text_color = crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL);
        let font_size = L::MENU_BAR_FONT_SIZE;
        let bar_h = view.frame().size.height;
        let btn_h = L::MENU_BAR_BUTTON_HEIGHT;
        let btn_y = (bar_h - btn_h) / 2.0;
        let radius = L::MENU_BAR_BUTTON_RADIUS;
        let pad_x = L::MENU_BAR_PADDING_X;
        let hover_idx = *self.hover_idx.lock().unwrap();
        let active_idx = *self.active_idx.lock().unwrap();

        let mut x = MENU_START_X;
        for (i, menu) in self.menus.iter().enumerate() {
            let text_w = crate::render::text::measure_text(&menu.title, font_size);
            let btn_w = text_w + pad_x * 2.0;

            // Hover/active background — rounded, centered
            if active_idx == Some(i) || hover_idx == Some(i) {
                ctx.set_color(match active_idx == Some(i) {
                    true => crate::theme::color(crate::theme::keys::CLR_BG_SELECTED),
                    false => crate::theme::color(crate::theme::keys::CLR_BG_HOVER),
                });
                ctx.fill_rounded_rect(NSRect::new(x, btn_y, btn_w, btn_h), radius);
            }

            // Text — horizontally and vertically centered in button
            ctx.set_color(text_color);
            let text_x = x + (btn_w - text_w) / 2.0;
            let text_y = btn_y + (btn_h - font_size) / 2.0;
            ctx.draw_text(&menu.title, NSPoint::new(text_x, text_y), font_size);

            x += btn_w;
        }
    }

    /// Handles mouse movement to update hover states and switch active dropdowns.
    /// mirrors macOS behavior where hovering a title with an open menu switches instantly.
    fn mouse_moved(&mut self, view: &mut NSView, event: &NSEvent) {
        let pos = event.location_in_window();
        let bar_h = view.frame().size.height;
        let mut x = MENU_START_X;
        let mut new_hover = None;
        for (i, menu) in self.menus.iter().enumerate() {
            let w = menu_btn_width(&menu.title);
            if pos.x >= x && pos.x <= x + w && pos.y < bar_h {
                new_hover = Some(i);
                break;
            }
            x += w;
        }
        *self.hover_idx.lock().unwrap() = new_hover;

        let active = *self.active_idx.lock().unwrap();
        let _ = (active.is_some() && new_hover.is_some() && new_hover != active).then(|| {
            let idx = new_hover.unwrap();
            *self.active_idx.lock().unwrap() = Some(idx);

            let active_clone = self.active_idx.clone();
            let items = self.menus[idx].items.clone();
            let mut drop_x = MENU_START_X;
            for i in 0..idx {
                drop_x += menu_btn_width(&self.menus[i].title);
            }

            let _ = crate::application::NSApplication::shared()
                .try_lock()
                .map(|mut app| {
                    app.pending_actions.push(Box::new(move |root| {
                        root.subviews_mut()
                            .retain(|v| !v.name.starts_with("MenuOverlay"));
                        let bounds = root.bounds();
                        let h = dropdown_height(items.len());
                        let w = crate::NSLayoutConstants::MENU_DROPDOWN_WIDTH;
                        let mut overlay = NSView::new(bounds).with_name("MenuOverlay");
                        overlay.set_delegate(MenuOverlayDelegate {
                            active_idx: active_clone.clone(),
                        });
                        let rect = NSRect::new(drop_x, 0.0, w, h);
                        let mut dropdown = NSView::new(rect).with_name("MenuOverlayDropdown");
                        dropdown.set_corner_radius(crate::NSLayoutConstants::MENU_DROPDOWN_RADIUS);
                        dropdown.set_clips_to_bounds(true);
                        dropdown.set_delegate(MenuDropdownDelegate::new(
                            items,
                            active_clone,
                        ));
                        overlay.add_subview(dropdown);
                        root.add_subview(overlay);
                    }));
                });
        });

        view.set_needs_display();
    }

    /// Handles mouse clicks to toggle menu dropdowns or dismiss the active one.
    /// updates the application's pending actions queue to modify the view hierarchy.
    fn mouse_down(&mut self, view: &mut NSView, event: &NSEvent) {
        let pos = event.location_in_window();
        let bar_h = view.frame().size.height;
        let mut x = MENU_START_X;
        let mut clicked = None;
        for (i, menu) in self.menus.iter().enumerate() {
            let w = menu_btn_width(&menu.title);
            if pos.x >= x && pos.x <= x + w && pos.y < bar_h {
                clicked = Some(i);
                break;
            }
            x += w;
        }

        let _ =
            clicked.map(|idx| {
                let mut active = self.active_idx.lock().unwrap();
                match *active == Some(idx) {
                    true => {
                        *active = None;
                        let _ = crate::application::NSApplication::shared().try_lock().map(
                            |mut app| {
                                app.pending_actions.push(Box::new(|root| {
                                    root.subviews_mut()
                                        .retain(|v| !v.name.starts_with("MenuOverlay"));
                                }));
                            },
                        );
                    }
                    false => {
                        *active = Some(idx);
                        let items = self.menus[idx].items.clone();
                        let active_clone = self.active_idx.clone();
                        let mut drop_x = MENU_START_X;
                        for i in 0..idx {
                            drop_x += menu_btn_width(&self.menus[i].title);
                        }

                        let _ = crate::application::NSApplication::shared().try_lock().map(
                            |mut app| {
                                app.pending_actions.push(Box::new(move |root| {
                                    root.subviews_mut()
                                        .retain(|v| !v.name.starts_with("MenuOverlay"));
                                    let bounds = root.bounds();
                                    let h = dropdown_height(items.len());
                                    let w = crate::NSLayoutConstants::MENU_DROPDOWN_WIDTH;
                                    let mut overlay = NSView::new(bounds).with_name("MenuOverlay");
                                    overlay.set_delegate(MenuOverlayDelegate {
                                        active_idx: active_clone.clone(),
                                    });
                                    let rect = NSRect::new(drop_x, 0.0, w, h);
                                    let mut dropdown =
                                        NSView::new(rect).with_name("MenuOverlayDropdown");
                                    dropdown.set_corner_radius(crate::NSLayoutConstants::MENU_DROPDOWN_RADIUS);
                                    dropdown.set_clips_to_bounds(true);
                                    dropdown.set_delegate(MenuDropdownDelegate::new(
                                        items,
                                        active_clone,
                                    ));
                                    overlay.add_subview(dropdown);
                                    root.add_subview(overlay);
                                }));
                            },
                        );
                    }
                }
            });
        view.set_needs_display();
    }

    /// Tests whether a point should intercept events for the menu bar.
    ///
    /// Returns `true` for any point within the menu bar's vertical bounds so that
    /// MouseMoved events always reach the delegate — this allows hover states to be
    /// cleared when the cursor moves outside menu item bounds but remains in the bar.
    /// Points above the bar height only intercept if a dropdown is active.
    fn hit_test(&self, view: &NSView, point: NSPoint) -> bool {
        let bar_h = view.frame().size.height;
        if point.y > bar_h {
            return self.active_idx.lock().unwrap().is_some();
        }
        // Within menu bar height — always accept so MouseMoved can clear hover states
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
