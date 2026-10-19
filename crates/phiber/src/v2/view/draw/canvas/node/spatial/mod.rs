//! The spatial node view: the same graph as solids over a floor.
//!
//! **A VIEW, not an editor** — the decision on record (2026-10-14). You look
//! at it and leave; every gesture that changes the tree stays in Flat, which
//! remains the parity path: Blender has no 3D node editor, so there is no
//! source to port here and no truth test to measure against. Every divergence
//! lives on this side of the toggle, and nothing about Flat changes.
//!
//! What it is, in one pass over the pieces:
//!
//! * `mode.rs` — Flat or Spatial, per editor, and the header radio.
//! * `layout.rs` — the shared layout: every node's rect and every link's two
//!   ends in view units, read off the SAME functions the flat `plan` uses, so
//!   the two modes cannot disagree about where a socket is.
//! * `set.rs` — the table: a corner radius and a height per class, the hex
//!   prism for the field class, and the view's few constants.
//! * `solid.rs` / `tube.rs` — a node as faces, a link as a spine.
//! * `camera.rs` — the orbit over the flat view's own framing, so switching
//!   modes lands on the same thing in both directions.
//! * `ink.rs` / `paint.rs` / `title.rs` — the colours (the theme's), the
//!   painter, and the titles on the top faces.
//! * `gesture.rs` — orbit, pan and dolly.
//!
//! The flat view's `x` and `y` are never re-laid: a node's location is
//! `bNode.location` out of the file, and only the two new axes — height and
//! radius — are the view's to assign. Fold distance, never order: there are no
//! impossible stairs here, because a graph cannot have a cycle and a picture
//! that drew one would be lying.

pub mod camera;
pub mod gesture;
pub mod ink;
pub mod layout;
pub mod mode;
pub mod paint;
pub mod set;
pub mod solid;
pub mod title;
pub mod tube;

pub use mode::BNodeMode;

use super::super::super::ctx::BRenderCtx;
use super::plan::{BCardSize, positions};
use super::view::store as view_store;
use super::{BNode, link, remove, select};
use crate::v2::view::draw::flex::{BFlex, BPlace};
use gpui::prelude::*;
use gpui::AnyElement;
use std::rc::Rc;

pub struct BSpatial;

impl BSpatial {
    /// The graph of the active object's first material, as solids; with no
    /// graph, the floor and its grid alone.
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let canvas_div = super::super::base(ctx);
        let graph = BNode::graph(ctx).unwrap_or_default();
        let size = BCardSize {
            header: ctx.style.dimensions.header_height,
            unit: ctx.style.dimensions.row_height,
            width: ctx.style.dimensions.channels_width,
        };
        // The links as the flat editor draws them — the file's with this
        // host's edits applied and a deleted node's wires pruned — so a tube
        // is a noodle lifted, never a different wiring.
        let mut wired = link::edits_of(ctx.state, ctx.host).apply(&graph.links);
        remove::removed_of(ctx.state, ctx.host).prune(&mut wired);
        let view = view_store::view_of(ctx.state, ctx.host, &graph.nodes, |n| size.height_of(n, &link::linked_inputs(&wired, &n.name)), size.width);
        let (_, offsets, _) = positions(&graph, &view, ctx.state, ctx.host);
        let selection = select::selection_of(ctx.state, ctx.host);
        let layout = layout::BSpatialLayout::of(&graph, &wired, &offsets, size, &selection);
        let orbit = camera::BOrbit::of(ctx.state, ctx.host);
        let grid = ctx.style.node_grid_stroke();
        let ink = ink::BSpatialInk {
            floor: ctx.style.colors.bg_viewport,
            grid: grid.color,
            grid_width: grid.width,
            header: ctx.style.colors.bg_control,
            plate: ctx.style.colors.bg_panel,
            muted_toward: ctx.style.colors.bg_panel,
            tube: ctx.style.colors.text_dim,
            tube_shadow: ctx.style.colors.bg_viewport,
            selected: ctx.style.colors.accent,
            rim_width: ctx.style.strokes.border * 2.0,
            title: ctx.style.colors.text,
            title_muted_alpha: ctx.style.effects.control_muted_alpha,
        };
        let (painter, bounds) = paint::element(
            paint::BSpatialPainting { layout, view, orbit, header: size.header, ink },
            ctx,
        );
        let canvas_div = canvas_div.child(BFlex::column().place(BPlace::Overlay).styled(painter, ctx));
        let gesture = Rc::new(gesture::BSpatialGesture {
            host: ctx.host,
            handler: ctx.handler.clone(),
            bounds,
            view,
            orbit,
        });
        gesture::listen(canvas_div, gesture, ctx).into_any_element()
    }
}

#[cfg(test)]
mod tests;
