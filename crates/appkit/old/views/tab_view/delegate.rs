use super::item::NSTabViewItem;
use crate::theme::{color, keys};
use crate::layout::{spacing, hierarchy};
use crate::{NSColor, NSPoint, NSRect, NSView, NSViewDelegate};
use std::sync::{Arc, Mutex};

pub struct NSTabViewDelegate {
    pub(crate) items: Arc<Mutex<Vec<NSTabViewItem>>>,
    pub(crate) selected_index: Arc<Mutex<Option<usize>>>,
    pub(crate) hover_index: Arc<Mutex<Option<usize>>>,
}

impl NSTabViewDelegate {
    pub fn set_selected_index(&self, idx: Option<usize>) {
        *self.selected_index.lock().unwrap() = idx;
    }
}

/// Compute the pixel width of a tab label from character count and padding.
fn tab_width(label: &str) -> f32 {
    let char_w = 7.5; // Average character width
    let pad_x = spacing::padding::TAB_H;
    label.chars().count() as f32 * char_w + pad_x * 2.0
}

impl NSViewDelegate for NSTabViewDelegate {
    fn wants_mouse_down(&self, _point: crate::graphics::NSPoint) -> bool {
        // Always claim mouse events within the tab view bounds
        true
    }

    fn draw(&self, view: &NSView, ctx: &mut crate::render::RenderContext) {
        let items = self.items.lock().unwrap();
        let sel = *self.selected_index.lock().unwrap();
        let bounds = view.bounds();
        let tab_h = 44.0; // Tab bar height
        let radius = crate::theme::ThemeToken::RADIUS_TAB();

        // Tab bar background — uses hierarchy system
        ctx.set_color(hierarchy::bg_color("toolbar"));
        ctx.fill_rect(NSRect::sized(bounds.width(), tab_h));

        let inactive_text = color(keys::CLR_TAB_TEXT_INACTIVE);
        let active_text = color(keys::CLR_TAB_TEXT_ACTIVE);
        let active_bg = hierarchy::hover_color(hierarchy::bg_color("button"));
        let pad_x = spacing::padding::TAB_H;
        let font_sz = 13.0;

        let mut x = spacing::margin::TOOLBAR;
        let hover_idx = *self.hover_index.lock().unwrap();

        // Center tabs horizontally
        let spacing_gap = spacing::gap::TAB;
        let total_tabs_width: f32 = items.iter()
            .map(|item| tab_width(item.label()))
            .sum::<f32>()
            + spacing_gap * (items.len().saturating_sub(1)) as f32;
        let center_offset = ((bounds.width() - total_tabs_width) / 2.0).max(x);
        x = center_offset;

        let tab_item_h = 28.0; // Height of each tab pill
        let tab_y = spacing::padding::TAB_V; // Vertical padding from top

        for (i, item) in items.iter().enumerate() {
            let label = item.label();
            let w = tab_width(label);
            let is_sel = Some(i) == sel;
            let is_hover = Some(i) == hover_idx;

            if is_sel {
                ctx.set_color(active_bg);
                ctx.fill_rounded_rect(NSRect::new(x, tab_y, w, tab_item_h), radius);
                ctx.set_color(active_text);
            } else if is_hover {
                ctx.set_color(hierarchy::hover_color(hierarchy::bg_color("button")));
                ctx.fill_rounded_rect(NSRect::new(x, tab_y, w, tab_item_h), radius);
                ctx.set_color(inactive_text);
            } else {
                ctx.set_color(inactive_text);
            }

            // Vertically center text in tab pill
            let text_y = tab_y + (tab_item_h - font_sz) / 2.0;
            ctx.draw_text(label, NSPoint::new(x + pad_x, text_y), font_sz);
            x += w + spacing_gap;
        }

        // Bottom separator
        ctx.set_color(hierarchy::separator_color("toolbar"));
        ctx.fill_rect(NSRect::new(0.0, tab_h - 1.0, bounds.width(), 1.0));
    }

    fn layout(&mut self, view: &mut NSView) {
        let sel = *self.selected_index.lock().unwrap();
        let bounds = view.bounds();
        let tab_h = 44.0; // Tab bar height
        let content_y = tab_h;

        for (i, subview) in view.subviews_mut().iter_mut().enumerate() {
            if Some(i) == sel {
                subview.set_hidden(false);
                subview.set_frame(NSRect::new(
                    0.0,
                    content_y,
                    bounds.width(),
                    bounds.height() - content_y,
                ));
                subview.layout_subviews();
            } else {
                subview.set_hidden(true);
            }
        }
    }

    fn mouse_moved(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let items = self.items.lock().unwrap();
        let pos = event.location_in_window();
        let tab_h = 44.0;
        let bounds = view.bounds();

        let mut new_hover = None;
        if pos.y >= 0.0 && pos.y <= tab_h {
            let spacing_gap = spacing::gap::TAB;
            let total_tabs_width: f32 = items.iter()
                .map(|item| tab_width(item.label()))
                .sum::<f32>()
                + spacing_gap * (items.len().saturating_sub(1)) as f32;
            let mut x = ((bounds.width() - total_tabs_width) / 2.0).max(spacing::margin::TOOLBAR);

            for (i, item) in items.iter().enumerate() {
                let w = tab_width(item.label());
                if pos.x >= x && pos.x <= x + w {
                    new_hover = Some(i);
                    break;
                }
                x += w + spacing_gap;
            }
        }

        let mut h = self.hover_index.lock().unwrap();
        if *h != new_hover {
            *h = new_hover;
            view.set_needs_display();
        }
    }

    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let items = self.items.lock().unwrap();
        let pos = event.location_in_window();
        let tab_h = 44.0;
        let bounds = view.bounds();

        // Tab bar occupies the top portion
        if pos.y < 0.0 || pos.y > tab_h {
            return;
        }

        let spacing_gap = spacing::gap::TAB;
        let total_tabs_width: f32 = items.iter()
            .map(|item| tab_width(item.label()))
            .sum::<f32>()
            + spacing_gap * (items.len().saturating_sub(1)) as f32;
        let mut x = ((bounds.width() - total_tabs_width) / 2.0).max(spacing::margin::TOOLBAR);

        let mut clicked = None;
        for (i, item) in items.iter().enumerate() {
            let w = tab_width(item.label());
            if pos.x >= x && pos.x <= x + w {
                clicked = Some(i);
                break;
            }
            x += w + spacing_gap;
        }

        if let Some(idx) = clicked {
            drop(items);
            println!("[NSTabView] Switched to tab {}", idx);
            *self.selected_index.lock().unwrap() = Some(idx);
            // Directly show/hide subviews to switch tabs
            let content_y = tab_h;
            let bounds = view.bounds();
            for (i, subview) in view.subviews_mut().iter_mut().enumerate() {
                if i == idx {
                    subview.set_hidden(false);
                    subview.set_frame(NSRect::new(
                        0.0,
                        content_y,
                        bounds.width(),
                        bounds.height() - content_y,
                    ));
                } else {
                    subview.set_hidden(true);
                }
            }
            view.set_needs_display();
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
