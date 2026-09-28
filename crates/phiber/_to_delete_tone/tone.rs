//! The surface tone and gutter each prim draws with — one tone per role,
//! shared by the column, bar, split, table and sort prims.

use super::color::resolved_gradient;
use super::ctx::BRenderCtx;
use crate::v2::theme::{BColor, BSurfaceTone};
use crate::v2::view::prim::BRowShade;
use gpui::Background;

/// The body column's surface.
pub(super) fn body_bg(ctx: &BRenderCtx) -> Background {
    resolved_gradient(ctx.style.surface(BSurfaceTone::Body))
}

/// A headed body's surface — the file list under its column heading: a
/// section with a title takes the card's surface as it takes its radius
/// (`body`), so its heading lifts off its rows as a panel's does.
pub(super) fn card_bg(ctx: &BRenderCtx) -> Background {
    resolved_gradient(ctx.style.surface(BSurfaceTone::Card))
}

/// A card heading's surface (`heading_surface`) for a heading drawn outside
/// a card — the file list's column heading — so it lifts as a panel
/// header does.
pub(super) fn heading_bg(ctx: &BRenderCtx) -> Background {
    resolved_gradient(ctx.style.heading_surface(BSurfaceTone::Card))
}

/// A property row's surface outside Properties (`split`).
pub(super) fn section_bg(ctx: &BRenderCtx) -> Background {
    resolved_gradient(ctx.style.surface(BSurfaceTone::Section))
}

/// A sidebar holds cards, and a card already carries its own surface, so
/// the column takes the body tone rather than a darker one of its own.
pub(super) fn sidebar_bg(ctx: &BRenderCtx) -> Background {
    resolved_gradient(ctx.style.surface(BSurfaceTone::Body))
}

/// Panel edge to contained controls.
fn surface_gutter(ctx: &BRenderCtx) -> f32 {
    ctx.style.spacing.pad_md
}

/// A row's own text inset. Every row a card holds carries it, so
/// `content_inset + row_inset == surface_gutter` and a row's text lands in the
/// same column as the heading title above it (`tests/layout_gutters.rs`).
pub(crate) fn row_inset(ctx: &BRenderCtx) -> f32 {
    ctx.style.spacing.pad_sm
}

/// Card edge to the rows it holds — the gutter that, with `row_inset`, puts
/// row text under the heading title.
pub(crate) fn content_inset(ctx: &BRenderCtx) -> f32 {
    surface_gutter(ctx) - row_inset(ctx)
}

// ── One row treatment ─────────────────────────────────────────────────────
//
// A tree row, a list row and a table row are one thing drawn three times.
// They used to each map `BRowShade` themselves, and disagreed: the tree
// painted an alternating row in the *hover* colour, so a striped row read as
// permanently hovered and a real hover was invisible on it; and a selected
// tree row answered the pointer with nothing while a selected list or table
// row lightened. The mapping lives here now, once.

/// What a row of `shade` paints at rest — `None` to let its surface through.
///
/// A resting fill is opaque, so a row that has nothing of its own to say
/// paints nothing rather than restating its panel's colour flat: a flat fill
/// over a swept surface would cancel the sweep on every row.
pub(crate) fn row_fill(shade: BRowShade, ctx: &BRenderCtx) -> Option<BColor> {
    match shade {
        BRowShade::Flat => None,
        BRowShade::Alt => Some(ctx.style.colors.row_alt),
        BRowShade::Selected => Some(ctx.style.colors.row_selected),
        BRowShade::Active => Some(ctx.style.colors.row_active),
    }
}

/// What it paints under the pointer. A row that already carries a fill
/// lightens by one ladder step off *that* fill, so the pointer reads on a
/// selected row as clearly as on an empty one; a row with no fill of its own
/// takes the shared item hover.
pub(crate) fn row_hover(shade: BRowShade, ctx: &BRenderCtx) -> BColor {
    match row_fill(shade, ctx) {
        Some(resting) if matches!(shade, BRowShade::Selected | BRowShade::Active) => {
            ctx.style.selected_hover(resting)
        }
        _ => item_hover(ctx),
    }
}

/// The one hover every selectable thing outside a header strip answers with —
/// a row, a column heading, a thumbnail, a window control, a gizmo button.
///
/// Controls *inside* a strip are a separate, deliberate family: they rest on
/// nothing and hover to an accent wash (`chrome::paint`), the way Blender
/// highlights a menu entry in blue while highlighting an Outliner row in
/// grey. The split is intended; what was not intended is ten call sites each
/// naming `colors.bg_hover` for themselves, so this is the one place the
/// neutral hover is chosen.
pub(crate) fn item_hover(ctx: &BRenderCtx) -> BColor {
    ctx.style.host_item_hover(ctx.host)
}
