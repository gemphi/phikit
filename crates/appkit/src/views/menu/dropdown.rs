//! # Menu Dropdown and Overlay Delegates
//!
//! Handles rendering, hovering, clicking, and dismissal of menu dropdown items.
//!
//! All layout values come from NSLayoutConstants — no hardcoded numbers.

use super::item::MenuItem;
use super::MenuAction;
use crate::render::RenderContext;
use crate::NSLayoutConstants as L;
use crate::{NSColor, NSEvent, NSPoint, NSRect, NSView, NSViewDelegate};
use std::sync::{Arc, Mutex};

/// Delegate that dismisses the menu dropdown when clicking outside of it.
pub struct MenuOverlayDelegate {
    pub(crate) active_idx: Arc<Mutex<Option<usize>>>,
}

impl NSViewDelegate for MenuOverlayDelegate {
    fn mouse_down(&mut self, _view: &mut NSView, _event: &NSEvent) {
        *self.active_idx.lock().unwrap() = None;
        let _ = crate::application::NSApplication::shared()
            .try_lock()
            .map(|mut app| {
                app.pending_actions.push(Box::new(|root| {
                    root.subviews_mut()
                        .retain(|v| !v.name.starts_with("MenuOverlay"));
                }));
            });
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

/// Delegate that draws the items inside a dropdown and tracks selection clicks.
pub struct MenuDropdownDelegate {
    pub(crate) items: Vec<MenuItem>,
    pub(crate) hover_idx: Mutex<Option<usize>>,
    pub(crate) active_idx: Arc<Mutex<Option<usize>>>,
    cached_action: Mutex<Option<MenuAction>>,
    pressed_idx: Mutex<Option<usize>>,
}

impl MenuDropdownDelegate {
    pub fn new(items: Vec<MenuItem>, active_idx: Arc<Mutex<Option<usize>>>) -> Self {
        Self {
            items,
            hover_idx: Mutex::new(None),
            active_idx,
            cached_action: Mutex::new(None),
            pressed_idx: Mutex::new(None),
        }
    }
}

impl NSViewDelegate for MenuDropdownDelegate {
    fn draw(&self, view: &NSView, ctx: &mut RenderContext) {
        let b = view.bounds();
        let pad_x = L::MENU_DROPDOWN_PADDING_X;
        let item_h = L::MENU_DROPDOWN_ITEM_HEIGHT;
        let font_sz = L::MENU_DROPDOWN_FONT_SIZE;
        let radius = L::MENU_DROPDOWN_RADIUS;
        let item_radius = L::MENU_DROPDOWN_ITEM_RADIUS;
        let item_inset = L::MENU_DROPDOWN_ITEM_INSET;
        let text_y_offset = (item_h - font_sz) / 2.0;

        // Card background — simple rounded rect (no glass blur for performance)
        ctx.set_color(crate::theme::color(crate::theme::keys::CLR_BG_DROPDOWN));
        ctx.fill_rounded_rect(b, radius);

        // Border
        ctx.draw_rounded_rect_border(
            b, radius, L::SEPARATOR_THICKNESS,
            crate::theme::color(crate::theme::keys::CLR_BORDER),
        );

        let hover_idx = *self.hover_idx.lock().unwrap();
        let pressed_idx = *self.pressed_idx.lock().unwrap();

        let mut y = L::MENU_DROPDOWN_PADDING_Y;
        for (i, item) in self.items.iter().enumerate() {
            match item.title.as_str() {
                "-" => {
                    ctx.set_color(crate::theme::color(crate::theme::keys::CLR_SEPARATOR));
                    ctx.fill_rect(NSRect::new(
                        pad_x, y + item_h / 2.0,
                        b.width() - pad_x * 2.0, L::SEPARATOR_THICKNESS,
                    ));
                }
                _ => {
                    if pressed_idx == Some(i) && item.enabled {
                        ctx.set_color(crate::theme::color(crate::theme::keys::CLR_BG_SELECTED));
                        ctx.fill_rounded_rect(
                            NSRect::new(item_inset, y, b.width() - item_inset * 2.0, item_h),
                            item_radius,
                        );
                    } else if hover_idx == Some(i) && item.enabled {
                        ctx.set_color(crate::theme::color(crate::theme::keys::CLR_BG_HOVER));
                        ctx.fill_rounded_rect(
                            NSRect::new(item_inset, y, b.width() - item_inset * 2.0, item_h),
                            item_radius,
                        );
                    }

                    let text_color = match item.enabled {
                        true => crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL),
                        false => crate::theme::color(crate::theme::keys::CLR_TEXT_DISABLED),
                    };
                    ctx.set_color(text_color);
                    ctx.draw_text(
                        &item.title,
                        NSPoint::new(pad_x + item_inset, y + text_y_offset),
                        font_sz,
                    );
                }
            }
            y += item_h;
        }
    }

    fn mouse_moved(&mut self, view: &mut NSView, event: &NSEvent) {
        let pos = event.location_in_window();
        let local_pos = NSPoint::new(pos.x - view.frame().x(), pos.y - view.frame().y());
        let idx = ((local_pos.y - L::MENU_DROPDOWN_PADDING_Y) / L::MENU_DROPDOWN_ITEM_HEIGHT)
            .max(0.0) as usize;
        let new_hover = if idx < self.items.len() { Some(idx) } else { None };

        let mut h = self.hover_idx.lock().unwrap();
        if *h != new_hover {
            *h = new_hover;
            drop(h);

            let mut cached = self.cached_action.lock().unwrap();
            *cached = new_hover
                .and_then(|idx| self.items.get(idx))
                .filter(|item| item.enabled && item.title != "-")
                .and_then(|item| item.action.clone());

            view.set_needs_display();
        }
    }

    fn mouse_down(&mut self, view: &mut NSView, event: &NSEvent) {
        let pos = event.location_in_window();
        let local_pos = NSPoint::new(pos.x - view.frame().x(), pos.y - view.frame().y());
        let idx = ((local_pos.y - L::MENU_DROPDOWN_PADDING_Y) / L::MENU_DROPDOWN_ITEM_HEIGHT)
            .max(0.0) as usize;

        let item_valid = self.items.get(idx)
            .map(|item| item.enabled && item.title != "-")
            .unwrap_or(false);

        if !item_valid { return; }

        *self.pressed_idx.lock().unwrap() = Some(idx);
        view.set_needs_display();

        let deferred_action = {
            let cached = self.cached_action.lock().unwrap();
            if *self.hover_idx.lock().unwrap() == Some(idx) {
                cached.clone()
            } else {
                self.items.get(idx).and_then(|item| item.action.clone())
            }
        };

        *self.active_idx.lock().unwrap() = None;
        let _ = crate::application::NSApplication::shared()
            .try_lock()
            .map(|mut app| {
                app.pending_actions.push(Box::new(|root| {
                    root.subviews_mut()
                        .retain(|v| !v.name.starts_with("MenuOverlay"));
                }));
                if let Some(action) = deferred_action {
                    app.pending_actions.push(Box::new(move |_root| {
                        action();
                    }));
                }
            });
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
