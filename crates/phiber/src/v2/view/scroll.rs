//! The drawn scroller — standalone, as `structure.md` §3 asks: the geometry
//! of a scroll bar, and nothing about which view scrolls.
//!
//! Blender ref: `editors/include/UI_view2d.hh` and `view2d.cc`
//! (`view2d_scrollers_calc`, `UI_view2d_scrollers_draw`):
//! - the bar is `V2D_SCROLL_WIDTH` = 0.45 widget units + 2 px wide;
//! - the thumb is the visible share of the content, never shorter than
//!   `V2D_SCROLL_THUMB_SIZE_MIN` (30 px), placed by how far the view is
//!   scrolled;
//! - away from the pointer the scroller fades to `V2D_SCROLL_MIN_ALPHA`.

/// `U.widget_unit` at 1× scale.
pub const WIDGET_UNIT: f32 = 20.0;
/// `V2D_SCROLL_WIDTH` at 1× scale.
pub const SCROLL_WIDTH: f32 = 0.45 * WIDGET_UNIT + 2.0;
/// `V2D_SCROLL_THUMB_SIZE_MIN` at 1× scale.
pub const THUMB_MIN: f32 = 30.0;
/// `V2D_SCROLL_MIN_ALPHA`: the scroller's opacity when the pointer is away.
pub const MIN_ALPHA: f32 = 0.4;
/// Space between the bar's edge and the thumb drawn inside it.
pub const THUMB_INSET: f32 = 2.0;

/// The thumb along the bar: where it starts and how long it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BThumb {
    pub start: f32,
    pub len: f32,
}

/// The thumb for a view `viewport` long over `content`, scrolled `offset`
/// in. `None` when everything fits (Blender hides the scroller then).
pub fn thumb(viewport: f32, content: f32, offset: f32) -> Option<BThumb> {
    if viewport <= 0.0 || content <= viewport + 0.5 {
        return None;
    }
    let len = (viewport * viewport / content).max(THUMB_MIN).min(viewport);
    let travel = viewport - len;
    let at = (offset / (content - viewport)).clamp(0.0, 1.0);
    Some(BThumb { start: travel * at, len })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_to_scroll_draws_no_thumb() {
        assert_eq!(thumb(400.0, 400.0, 0.0), None);
        assert_eq!(thumb(400.0, 300.0, 0.0), None);
    }

    #[test]
    fn the_thumb_is_the_visible_share_and_follows_the_offset() {
        assert_eq!(thumb(400.0, 800.0, 0.0), Some(BThumb { start: 0.0, len: 200.0 }));
        assert_eq!(thumb(400.0, 800.0, 400.0), Some(BThumb { start: 200.0, len: 200.0 }));
        assert_eq!(thumb(400.0, 800.0, 200.0), Some(BThumb { start: 100.0, len: 200.0 }));
    }

    #[test]
    fn a_long_list_keeps_the_thumb_grabbable() {
        let t = thumb(300.0, 30_000.0, 29_700.0).unwrap();
        assert_eq!(t.len, THUMB_MIN);
        assert_eq!(t.start + t.len, 300.0, "scrolled to the end, the thumb sits at the end");
    }

    #[test]
    fn the_bar_is_blenders_width() {
        assert_eq!(SCROLL_WIDTH, 11.0);
    }
}
