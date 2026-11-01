//! Widget defaults for separators and grid items.
//!
//! Provides factory methods for creating pre-styled UI widgets
//! used across panel implementations.
//!
//! Note: Button defaults are in `crate::controls::button::NSButtonDefaults`.

use crate::prelude::*;
use crate::theme;

/// Separator defaults.
pub struct SeparatorDefaults;

impl SeparatorDefaults {
    /// Standard horizontal separator.
    pub fn horizontal() -> NSView {
        let mut view = NSView::new(NSRect::sized(100.0, 1.0));
        view.set_background_color(Some(theme::color(theme::keys::CLR_SEPARATOR)));
        view.set_autoresizing_mask(NSAutoresizingMask::WIDTH_SIZABLE);
        view
    }
}

/// Grid item defaults for collection views.
pub struct GridItemDefaults;

impl GridItemDefaults {
    /// Standard file/item grid cell.
    pub fn item_view() -> NSView {
        let mut view = NSView::new(NSRect::sized(80.0, 80.0));
        view.set_background_color(Some(crate::theme::color(crate::theme::keys::CLR_BG_PANEL)));
        view.set_corner_radius(8.0);
        view
    }

    /// Icon label for grid items (large emoji/icon)
    pub fn icon_label(icon: &str) -> NSTextField {
        let mut lbl = NSTextField::label(icon);
        lbl.set_font_size(24.0);
        lbl.set_text_color(NSColor::white());
        lbl
    }

    /// Name label for grid items (filename below icon)
    pub fn name_label(name: &str) -> NSTextField {
        let mut lbl = NSTextField::label(name);
        lbl.set_font_size(10.0);
        lbl.set_text_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
        lbl
    }
}
