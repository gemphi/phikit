//! Frame nodes: the labelled box drawn behind a group of nodes.
//!
//! Ported from Blender 5.2.0 `node_draw.cc` — `calc_node_frame_dimensions`,
//! `frame_node_layout` and `frame_node_draw_label`. A frame is not a card: it
//! has no sockets, no header and no title bar. It is sized to BOUND the nodes
//! parented into it (a child names its frame by `bNode.parent`), and it is
//! drawn first, behind everything, so the nodes it groups sit on top of it.
//!
//! phiber drew a frame as an ordinary card — a tiny titled box floating among
//! the nodes it was meant to contain. This draws the box.
//!
//! Three deliberate divergences, all for want of `NodeFrame` storage, which
//! phiber does not parse:
//!
//! * the label height is Blender's DEFAULT `label_size` of 20, read as one
//!   widget unit ([`LABEL`]); a frame whose label was resized is drawn at the
//!   default size;
//! * a frame always shrinks to fit its children when it has any (Blender's
//!   `NODE_FRAME_SHRINK`); a manually-sized non-shrink frame is drawn hugging
//!   its children instead of at the size it was dragged to;
//! * the fill is a neutral panel tint rather than the frame's own `TH_NODE_FRAME`
//!   colour, which a frame can override per-node.
//!
//! A childless frame has nothing to bound, so it falls back to its stored
//! rectangle (`node_to_updated_rect`: the location is the top-left, Y up).

use super::super::super::ctx::BRenderCtx;
use super::plan::BCardSize;
use super::view::{BNodeView, node_bounds};
use super::drag;
use crate::phiphx::blend::shade::graph::BlendNode;
use crate::v2::state::{BKey as _, NodeKey};
use crate::v2::theme::BStyleRole;
use crate::v2::view::draw::color::to_hsla;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px};
use std::rc::Rc;

/// The per-node displacements a drag has written, as `set::render` gathers them:
/// a frame is sized off where its children are DRAWN, not where the file put
/// them, so dragging a child grows the frame and dragging the frame moves the
/// whole group (`drag::offset_of`).
pub type Offsets<'a> = &'a [(String, (f32, f32))];

/// One node's displacement from `offsets`, or none.
fn shift(offsets: Offsets, name: &str) -> (f32, f32) {
    offsets.iter().find(|(n, _)| n == name).map(|(_, o)| *o).unwrap_or((0.0, 0.0))
}

/// `1.5f * U.widget_unit` — the side and bottom margin, in widget units.
pub const MARGIN: f32 = 1.5;
/// The label height in widget units: Blender's default `label_size` of 20 is
/// one widget unit, and phiber does not read a frame's own `label_size`.
pub const LABEL: f32 = 1.0;
/// The fraction of the panel colour a frame's fill keeps; a frame is a backdrop,
/// so it is mostly transparent and lets the canvas read through.
const FILL_ALPHA: f32 = 0.5;

/// The side and top margins of a frame, in widget units.
///
/// `frame_node_layout`: the top margin grows to hold a label —
/// `max(margin, 1.5 * label_height + 0.2 * margin)` — and otherwise equals the
/// sides. A reader that used the side margin on top would clip the label.
pub fn margins(has_label: bool) -> (f32, f32) {
    let top = if has_label { MARGIN.max(1.5 * LABEL + 0.2 * MARGIN) } else { MARGIN };
    (MARGIN, top)
}

/// A node's drawn bounds in view units (Y up): its file rect shifted by the
/// displacement a drag has given it, recursing into child frames.
fn child_bounds(node: &BlendNode, nodes: &[BlendNode], size: BCardSize, offsets: Offsets, depth: u8) -> Option<(f32, f32, f32, f32)> {
    if node.frame() {
        return bounds_of(node, nodes, size, offsets, depth + 1);
    }
    let (ox, oy) = shift(offsets, &node.name);
    let b = node_bounds(node, size.height_of(node), size.width);
    Some((b.0 + ox, b.1 + oy, b.2 + ox, b.3 + oy))
}

/// A frame's bounds in view units (Y up): the union of its children's DRAWN
/// bounds, each padded by the margins, or its stored rectangle when it has no
/// children.
///
/// The children's drawn bounds — file place plus drag displacement — are what
/// make the box follow the nodes as they are moved; a box built from file
/// positions alone would sit still while a dragged child slid out of it.
///
/// `depth` guards against a frame parented into itself — a malformed file —
/// which would otherwise recurse forever.
pub fn bounds_of(frame: &BlendNode, nodes: &[BlendNode], size: BCardSize, offsets: Offsets, depth: u8) -> Option<(f32, f32, f32, f32)> {
    if depth > 16 {
        return None;
    }
    let (side, top) = margins(!frame.label.is_empty());
    let (m, mt) = (side * size.unit, top * size.unit);
    let mut rect: Option<(f32, f32, f32, f32)> = None;
    for child in nodes.iter().filter(|n| n.parent == Some(frame.address)) {
        if let Some((x0, y0, x1, y1)) = child_bounds(child, nodes, size, offsets, depth) {
            // Y up: `y1` is the top, so the label's room is added to it.
            let padded = (x0 - m, y0 - m, x1 + m, y1 + mt);
            rect = Some(match rect {
                None => padded,
                Some(r) => (r.0.min(padded.0), r.1.min(padded.1), r.2.max(padded.2), r.3.max(padded.3)),
            });
        }
    }
    rect.or_else(|| {
        // A childless frame keeps its stored rectangle: location is the top-left
        // (`node_to_updated_rect`), Y up, running down from there — shifted by
        // the frame's own displacement, since there are no children to carry it.
        (frame.width > 0.0 && frame.height > 0.0).then(|| {
            let (ox, oy) = shift(offsets, &frame.name);
            (frame.x + ox, frame.y - frame.height + oy, frame.x + frame.width + ox, frame.y + oy)
        })
    })
}

/// The frame's box, placed behind the cards, with its label in the top margin.
///
/// The box is also the grab handle: clicking it selects the frame and dragging
/// it moves the group (`wire::for_frame` expands a selected frame into the nodes
/// parented into it, so the whole group travels under one delta). The cards sit
/// on top, so a click LANDS on a card when there is one under the pointer and on
/// the frame only in its margin — which is how a frame is grabbed in Blender.
///
/// `None` when the frame bounds nothing and has no stored size — there is no
/// box to draw, which is Blender's own outcome for an empty frame.
pub fn handle(frame: &BlendNode, nodes: &[BlendNode], size: BCardSize, view: &BNodeView, offsets: Offsets, ctx: &BRenderCtx) -> Option<AnyElement> {
    let (xmin, ymin, xmax, ymax) = bounds_of(frame, nodes, size, offsets, 0)?;
    // The top-left in canvas fractions — Y up, so the top is `ymax`.
    let (fx, fy) = view.to_fraction((xmin, ymax));
    let (fw, fh) = ((xmax - xmin) / view.width(), (ymax - ymin) / view.height());
    let fill = ctx.style.colors.bg_control;
    let id = frame.name.clone();
    let mut box_div = div()
        .id(SharedString::from(format!("frame-{id}")))
        .absolute()
        .left(gpui::relative(fx))
        .top(gpui::relative(fy))
        .w(gpui::relative(fw))
        .h(gpui::relative(fh))
        .rounded(px(ctx.style.radius_for(BStyleRole::Card)))
        .bg(to_hsla((fill.0, fill.1, fill.2, FILL_ALPHA)))
        .border(px(ctx.style.strokes.border))
        .border_color(to_hsla(ctx.style.colors.border));
    if !frame.label.is_empty() {
        box_div = box_div.child(
            div()
                .px(px(ctx.style.spacing.pad_sm))
                .pt(px(ctx.style.spacing.pad_sm))
                .text_size(px(ctx.style.typography.header))
                .text_color(to_hsla(ctx.style.colors.text))
                .child(SharedString::from(frame.label.clone())),
        );
    }
    let handler = ctx.handler.clone();
    let action = BAction::Set(NodeKey::Selected(ctx.host).path(), id.clone());
    let clickable = box_div.on_click(move |_e: &ClickEvent, w, cx| {
        (Rc::clone(&handler))(&action, w, cx);
    });
    let from = shift(offsets, &frame.name);
    Some(drag::grab(clickable, drag::BNodeDrag::new(crate::v2::state::intern(&id), from)).into_any_element())
}

/// Every frame in the tree, as boxes to draw behind the nodes — the pass
/// Blender runs first (`node_draw_nodetree` draws frames before anything else),
/// so the nodes a frame groups sit on top of it.
pub fn layer(nodes: &[BlendNode], view: &BNodeView, size: BCardSize, offsets: Offsets, ctx: &BRenderCtx) -> Vec<AnyElement> {
    nodes.iter().filter(|n| n.frame()).filter_map(|f| handle(f, nodes, size, view, offsets, ctx)).collect()
}

#[cfg(test)]
mod tests;
