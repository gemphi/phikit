//! # NSToolbar Rendering & Drawing
//!
//! Handles layout compilation and view building for the toolbar.

use super::item::NSToolbarItem;
use super::NSToolbar;
use crate::graphics::NSRect;
use crate::view::NSView;
use crate::{theme, NSLayoutConstants};

impl NSToolbar {
    /// Compile and layout all items inside a single NSView of height `Self::height()`.
    /// Automatically performs Left / Center / Right alignment using Flexible Spaces.
    pub fn build_view(&self, win_w: f32) -> NSView {
        let h = self.height();
        let mut parent = NSView::new(NSRect::sized(win_w, h));

        parent.set_background_color(Some(theme::color(theme::keys::CLR_BG_TITLEBAR)));

        // Group items using "NSToolbarFlexibleSpaceItem" as dividers
        let mut left_group = Vec::new();
        let mut center_group = Vec::new();
        let mut right_group = Vec::new();
        let mut group_idx = 0;

        for item in &self.items {
            if item.item_identifier == "NSToolbarFlexibleSpaceItem" {
                group_idx += 1;
                continue;
            }
            match group_idx {
                0 => left_group.push(item),
                1 => center_group.push(item),
                _ => right_group.push(item),
            }
        }

        let gap = theme::f32_or(theme::keys::SPACING_SM, 8.0);

        // Measure measured width of item view
        let get_width = |item: &NSToolbarItem| -> f32 {
            if let Some(v) = item.view() {
                v.frame().width()
            } else {
                32.0
            }
        };

        // Measure group total width
        let measure_group = |group: &[&NSToolbarItem]| -> f32 {
            if group.is_empty() {
                return 0.0;
            }
            let sum: f32 = group.iter().map(|item| get_width(item)).sum();
            sum + (group.len() - 1) as f32 * gap
        };

        // 1. Layout Left Group (Start after traffic lights)
        let mut current_x = NSLayoutConstants::TRAFFIC_LIGHT_MARGIN_LEFT
            + (NSLayoutConstants::TRAFFIC_LIGHT_SIZE + NSLayoutConstants::TRAFFIC_LIGHT_SPACING) * 3.0
            + NSLayoutConstants::MARGIN_LARGE;

        for item in &left_group {
            if let Some(view) = item.view() {
                let w = get_width(item);
                let vh = view.frame().height();
                let vy = (h - vh) / 2.0;

                let mut cloned = view.clone();
                cloned.set_frame(NSRect::new(current_x, vy, w, vh));
                parent.add_subview(cloned);
                current_x += w + gap;
            }
        }

        // 2. Layout Center Group (Centered within the window width)
        let center_w = measure_group(&center_group);
        if center_w > 0.0 {
            let mut center_x = (win_w - center_w) / 2.0;
            for item in &center_group {
                if let Some(view) = item.view() {
                    let w = get_width(item);
                    let vh = view.frame().height();
                    let vy = (h - vh) / 2.0;

                    let mut cloned = view.clone();
                    cloned.set_frame(NSRect::new(center_x, vy, w, vh));
                    parent.add_subview(cloned);
                    center_x += w + gap;
                }
            }
        }

        // 3. Layout Right Group (Right-aligned inside the toolbar width)
        let right_w = measure_group(&right_group);
        if right_w > 0.0 {
            let mut right_x = win_w - right_w - theme::f32_or(theme::keys::SPACING_LG, 12.0);
            for item in &right_group {
                if let Some(view) = item.view() {
                    let w = get_width(item);
                    let vh = view.frame().height();
                    let vy = (h - vh) / 2.0;

                    let mut cloned = view.clone();
                    cloned.set_frame(NSRect::new(right_x, vy, w, vh));
                    parent.add_subview(cloned);
                    right_x += w + gap;
                }
            }
        }

        parent
    }
}
