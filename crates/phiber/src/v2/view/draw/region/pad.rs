//! A region's padding — the one thing that differs between a rail, a strip
//! and a content region once its axis and fill are settled, so it is said
//! here as one match rather than a ladder inside the region's own flex.

use super::super::ctx::BRenderCtx;
use super::super::flex::{BEdge, BFlex, BSpace};
use super::{rail_region, ruled_header};
use crate::v2::host::BHostType;
use crate::v2::region::BRegionType;

/// `flex` with the padding its region kind calls for.
pub(super) fn padded(flex: BFlex, kind: BRegionType, ctx: &BRenderCtx) -> BFlex {
    use BRegionType::{Center, Execute, Footer, Header, ToolHeader, Tools};
    let panelled = ctx.host.tools_hold_panels();
    match (kind, rail_region(kind, ctx.host), panelled) {
        // Every vertical strip is the topbar turned on its side, so every
        // rail is flush horizontally — the cap owns the gap on its outer edge
        // and runs into the pane on the other (`tab::cap`). Its vertical
        // padding matches the content region beside it, so the first icon
        // sits level with the first accordion header, the same lead top and
        // bottom; the rail's pinned cluster (or the activity bar's foot
        // group) then sits a cap gap off the region's bottom edge, the 2px
        // every cap keeps off its strip's edge (`menu_top_gap`).
        //
        // No horizontal padding: a rail tab spans the strip and meets the
        // pane beside it, which is how the active Properties tab merges into
        // the panel it selects (`layout_gutters`, and the pane-facing bleed
        // in `tab::properties_layers`). Insetting it breaks that join.
        //
        // The gap between PANES stays, though: a rail's caps run flush inside
        // their own pane, but a pane boundary is a pane boundary. The activity
        // bar's secondary column is the Center pane beside the rail's Left
        // one, and zeroing it ran that column into its parent when every
        // other host's panes keep the gutter. A NavBar or Tools region is only
        // a rail and runs to its area's edges; a *Center* region that leads
        // with a rail carries that content beside it, and content wants the
        // gutter every other content region keeps.
        (Center, true, _) => flex.pad_top(BSpace::RailLead).pad_bottom(BSpace::RailLead).pad_right(BSpace::PadSm),
        (_, true, _) => flex.pad_top(BSpace::RailLead).pad_bottom(BSpace::RailLead),
        // The file browser's path strip is the columns under it, read across:
        // each pane pads itself as the region under it does (`pane`), so the
        // strip itself pads nothing and leaves no gap.
        (ToolHeader, _, true) => flex.gap(BSpace::None),
        // A content header closes against what it heads with the same line an
        // accordion draws around itself — one edge of `BEdge::All`, the same
        // stroke and colour — so a header strip and a card heading read as the
        // same kind of boundary. A footer keeps none: the area's own border
        // closes it below.
        (Header | ToolHeader | Footer | Execute, ..) => match ruled_header(kind, ctx.host) {
            true => flex.pad_x(BSpace::PadMd).border(BEdge::Bottom),
            false => flex.pad_x(BSpace::PadMd),
        },
        // A column of panels, inset from the area as every content region is —
        // but sharing its inner edge with the region beside it, which pads it
        // once: two side-by-side regions are one `pad_md` apart, as an area's
        // content is from its edge, not two.
        (Tools, _, true) => flex.pad_left(BSpace::PadMd).pad_y(BSpace::PadMd),
        // Properties content: a small top/bottom pad so the first and last
        // accordion don't touch the edges, and a right gutter so the last
        // property row doesn't butt against the border. The rail beside it
        // derives its lead from the same pad (`rail_lead`).
        (Center, ..) if ctx.host == BHostType::Buts => flex.pad_y(BSpace::ContentPad).pad_right(BSpace::PadSm),
        // Content-fill hosts (3D viewport, node graphs, image / UV / seq /
        // clip, timelines) paint wall to wall: the viewport's clear and the
        // graph's grid must reach every edge. Their overlays are absolute, so
        // they need no padded parent.
        (Center, ..) if ctx.host.fills_content() => flex,
        // The file list beside the source list: the two columns are a seam
        // apart, as two areas of a workspace are (`BSpace::Seam`, the zone
        // gap) — not a content pad, which spaced them twice as far as any
        // zone. The outer edges keep the content inset.
        (Center, _, true) => flex.pad_left(BSpace::Seam).pad_right(BSpace::PadMd).pad_y(BSpace::ContentPad),
        _ => flex.pad_x(BSpace::PadMd).pad_y(BSpace::ContentPad),
    }
}
