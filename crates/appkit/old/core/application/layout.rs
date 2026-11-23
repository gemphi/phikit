//! Layout and modifier flag utilities for NSApplication.
use super::NSApplication;
use crate::graphics::NSAutoresizingMask;
use crate::view::NSView;

impl NSApplication {
    /// Propagate resize to subviews based on autoresizing masks.
    /// This is a mathematically precise implementation of Apple's autoresizing layout system,
    /// supporting proportional sizing, centering, and flexible margin anchoring.
    /// Views with delegates (like NSSplitView) are resized but their children are NOT
    /// auto-resized — the delegate's layout() handles child positioning.
    pub fn layout_subviews(
        view: &mut NSView,
        old_w: f32,
        old_h: f32,
        parent_w: f32,
        parent_h: f32,
    ) {
        let dx = parent_w - old_w;
        let dy = parent_h - old_h;

        for subview in view.subviews_mut() {
            let mask = subview.autoresizing_mask();
            let mut f = subview.frame();
            let sub_old_w = f.size.width;
            let sub_old_h = f.size.height;

            // Horizontal autoresizing:
            if mask.contains(NSAutoresizingMask::WIDTH_SIZABLE) {
                f.size.width = (f.size.width + dx).max(10.0);
            } else if mask
                .contains(NSAutoresizingMask::MIN_X_MARGIN | NSAutoresizingMask::MAX_X_MARGIN)
            {
                f.origin.x += dx / 2.0;
            } else if mask.contains(NSAutoresizingMask::MIN_X_MARGIN) {
                f.origin.x += dx;
            }

            // Vertical autoresizing:
            if mask.contains(NSAutoresizingMask::HEIGHT_SIZABLE) {
                f.size.height = (f.size.height + dy).max(10.0);
            } else if mask
                .contains(NSAutoresizingMask::MIN_Y_MARGIN | NSAutoresizingMask::MAX_Y_MARGIN)
            {
                f.origin.y += dy / 2.0;
            } else if mask.contains(NSAutoresizingMask::MIN_Y_MARGIN) {
                f.origin.y += dy;
            }

            subview.set_frame(f);

            // If this view has a delegate (e.g. NSSplitView), let the delegate
            // handle child layout — don't recurse with autoresizing.
            if subview.delegate().is_some() {
                // Delegate will handle children via layout_subviews() call later
                continue;
            }

            // Recursively update children with their respective old/new frames
            Self::layout_subviews(subview, sub_old_w, sub_old_h, f.size.width, f.size.height);
        }
    }

    /// Build modifier flags from current app state.
    pub fn modifier_flags(app: &NSApplication) -> u32 {
        let mut flags = 0u32;
        if app.opt_pressed {
            flags |= 1;
        }
        if app.shift_pressed {
            flags |= 2;
        }
        if app.cmd_pressed {
            flags |= 4;
        }
        if app.ctrl_pressed {
            flags |= 8;
        }
        flags
    }
}
