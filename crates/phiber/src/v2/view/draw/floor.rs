//! The least width a pinned header cell's controls fit in, split out of
//! `pane.rs` for its line cap.

use super::ctx::BRenderCtx;
use crate::v2::view::BView;

/// A floor for a pinned header cell that must not clip: one icon slot per
/// control it holds, plus the gaps between them.
///
/// A flex cell with a zero basis has no content size of its own to fall back
/// on, so a narrow window shrank the top bar's right-hand cell straight
/// through its icons. Counting the controls gives that cell a real minimum
/// without measuring text — everything in it is a square icon button.
pub(super) fn control_floor(list: &[BView], ctx: &BRenderCtx) -> f32 {
    fn count(v: &BView) -> usize {
        match v {
            BView::Button(_) | BView::MenuAnchor(_) | BView::Icon { .. } => 1,
            BView::Row { list } | BView::Column { list } | BView::Split { list } => list.iter().map(count).sum(),
            BView::Card(card) => card.body.iter().chain(&card.foot).map(count).sum(),
            BView::Part { body, .. } => count(body),
            BView::Popover { anchor, .. } => count(anchor),
            BView::ZoneToggleGroup { controls } => controls.len(),
            BView::WindowControls { .. } => super::caption::WINDOW_CONTROLS.len(),
            _ => 0,
        }
    }
    let n: usize = list.iter().map(count).sum();
    if n == 0 {
        return 0.0;
    }
    let slot = ctx.style.dimensions.button_height;
    let gap = ctx.style.spacing.gap_xs;
    n as f32 * slot + (n.saturating_sub(1)) as f32 * gap
}
