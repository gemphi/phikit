use super::NSView;
use crate::graphics::{NSAutoresizingMask, NSSize};

impl NSView {
    /// Called by set_frame when this view's own size changes.
    /// Repositions and resizes each direct subview according to its autoresizing mask.
    pub fn autoresize_subviews(&mut self, old_size: NSSize) {
        let new_w = self.frame.size.width;
        let new_h = self.frame.size.height;
        let delta_w = new_w - old_size.width;
        let delta_h = new_h - old_size.height;

        if delta_w == 0.0 && delta_h == 0.0 {
            return;
        }

        for subview in &mut self.subviews {
            let mask = subview.autoresizing_mask;
            if mask == NSAutoresizingMask::NONE {
                continue;
            }

            let mut f = subview.frame;

            // Horizontal Axis
            let flex_left = mask.contains(NSAutoresizingMask::MIN_X_MARGIN);
            let flex_width = mask.contains(NSAutoresizingMask::WIDTH_SIZABLE);
            let flex_right = mask.contains(NSAutoresizingMask::MAX_X_MARGIN);

            let mut h_flex_count = 0;
            if flex_left {
                h_flex_count += 1;
            }
            if flex_width {
                h_flex_count += 1;
            }
            if flex_right {
                h_flex_count += 1;
            }

            if h_flex_count > 0 {
                let share = delta_w / h_flex_count as f32;
                if flex_left {
                    f.origin.x += share;
                }
                if flex_width {
                    f.size.width += share;
                }
                // flex_right just absorbs the rest (implicit in superview size)
            }

            // Vertical Axis (Flipped: MIN_Y = Top, MAX_Y = Bottom)
            let flex_top = mask.contains(NSAutoresizingMask::MIN_Y_MARGIN);
            let flex_height = mask.contains(NSAutoresizingMask::HEIGHT_SIZABLE);
            let flex_bottom = mask.contains(NSAutoresizingMask::MAX_Y_MARGIN);

            let mut v_flex_count = 0;
            if flex_top {
                v_flex_count += 1;
            }
            if flex_height {
                v_flex_count += 1;
            }
            if flex_bottom {
                v_flex_count += 1;
            }

            if v_flex_count > 0 {
                let share = delta_h / v_flex_count as f32;
                if flex_top {
                    f.origin.y += share;
                }
                if flex_height {
                    f.size.height += share;
                }
                // flex_bottom absorbs the rest
            }

            subview.set_frame(f);
        }
    }
}
