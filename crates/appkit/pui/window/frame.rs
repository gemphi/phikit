//! The frame: where the window's own furniture goes, and what is left for
//! the space. Every app's window has the same frame: an activity bar, the
//! menu it shows, the tabs of its spaces, and the body. At a desk the bar
//! stands at the left with its menu beside it, and the tabs lie over the
//! body; in a hand the tabs are at the top, the bar lies at the bottom,
//! under a thumb, and its menu is drawn out over the body. The bar and
//! its menu stay a margin clear of the window's edges. In a theme that
//! gives the window a caption, that is the row at its very top, with
//! everything else under it.

use super::PCaption;
use crate::pui::core::{PMode, PRect, PSize};
use crate::theme::PTheme;

/// The theme's token for how high the tabs are.
pub const TAB: &str = "tab.height";
/// How far the activity bar stays from the window's edges.
pub const MARGIN: &str = "activity.margin";

/// Where the activity bar, its menu, the tabs and the space go in a
/// window.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PFrame {
    /// The caption, across the top; nothing in a theme without one.
    pub caption: PRect,
    pub activity: PRect,
    /// The menu the bar shows; nothing while none is shown.
    pub menu: PRect,
    pub tab: PRect,
    pub body: PRect,
}

impl PFrame {
    /// The frame of a window of that size. `thick` is how thick the
    /// activity bar is, nothing for an app without one; `menu` how wide
    /// the menu it shows, nothing while it shows none; `tab` says whether
    /// there are tabs to show, which take no room when there are none.
    pub fn of(size: PSize, mode: PMode, theme: &dyn PTheme, thick: f32, menu: f32, tab: bool) -> PFrame {
        let high = match tab {
            true => theme.size(TAB).unwrap_or_default(),
            false => 0.0,
        };
        let (wide, tall, top) = (size.width, size.height, PCaption::high(theme));
        let caption = PRect::new(0.0, 0.0, wide, top);
        // An app with no bar has no margin round one.
        let edge = match thick > 0.0 {
            true => theme.size(MARGIN).unwrap_or_default(),
            false => 0.0,
        };
        // Under a caption the bar and its menu need no margin at their top.
        let lead = match top > 0.0 {
            true => 0.0,
            false => edge,
        };
        match mode {
            PMode::Mobile => {
                let body = PRect::new(0.0, top + high, wide, tall - top - high - thick - edge * 2.0);
                let over = match menu > 0.0 {
                    true => PRect::new(edge, top + high + lead, wide - edge * 2.0, body.size.height - lead),
                    false => PRect::default(),
                };
                PFrame { caption, tab: PRect::new(0.0, top, wide, high), body, menu: over, activity: PRect::new(edge, tall - thick - edge, wide - edge * 2.0, thick) }
            }
            PMode::Desktop | PMode::Spatial => {
                let (left, head, most) = (edge + thick + menu, top + lead, tall - top - lead - edge);
                PFrame { caption, activity: PRect::new(edge, head, thick, most), menu: PRect::new(edge + thick, head, menu, most), tab: PRect::new(left, top, wide - left, high), body: PRect::new(left, top + high, wide - left, tall - top - high) }
            }
        }
    }
}
