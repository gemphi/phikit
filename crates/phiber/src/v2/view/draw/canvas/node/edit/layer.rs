//! The clickable overlay layer: a `div` per value target and per colour swatch,
//! laid over the painted cards so a click edits a value or opens a colour's
//! picker instead of selecting the node. The geometry is `hit.rs`; this is only
//! the GPUI wiring and the write actions (`overlay::BNodeOverlay`).

use super::hit::{BSwatchHit, swatch_hits, value_targets};
use super::super::link;
use super::super::overlay::BNodeOverlay;
use super::super::plan::BCardSize;
use super::super::view::BNodeView;
use super::BValueEdits;
use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeLink};
use crate::phiphx::blend::shade::socket::BSocketDefault;
use crate::v2::view::draw::{BPickSpace, BWheel};
use crate::v2::state::{BKey as _, NodeKey};
use crate::v2::view::draw::ctx::BRenderCtx;
use crate::v2::view::prim::BAction;
use crate::v2::view::draw::flex::{BFlex, BPlace};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString};
use std::rc::Rc;

/// A canvas-level clickable `div` at a fraction rect, running `action` on click
/// and stopping propagation so the card beneath neither selects nor drags.
fn target(id: String, at: (f32, f32), size: (f32, f32), action: BAction, ctx: &BRenderCtx) -> AnyElement {
    let handler = ctx.handler.clone();
    BFlex::cell()
        .place(BPlace::Fraction { x: at.0, y: at.1, w: size.0, h: size.1 })
        .div(ctx)
        .id(SharedString::from(id))
        .on_click(move |_e: &ClickEvent, w, cx| {
            cx.stop_propagation();
            (Rc::clone(&handler))(&action, w, cx);
        })
        .into_any_element()
}

/// The open swatch's wheel, hung under the four channel rows the card
/// draws below the swatch — the popup Blender opens on a colour socket,
/// `block_colorpicker`'s wheel. It writes the editor's value overlay.
fn wheel(node: &BlendNode, sh: &BSwatchHit, edits: &BValueEdits, ctx: &BRenderCtx) -> AnyElement {
    let rgba = match node.inputs.iter().find(|s| s.identifier == sh.socket).and_then(|s| s.default.as_ref()) {
        Some(BSocketDefault::Color(c)) => *c,
        _ => [0.8, 0.8, 0.8, 1.0],
    };
    let space = BPickSpace::Socket { node: node.name.clone(), socket: sh.socket.clone(), edits: edits.clone() };
    let wheel = BWheel::of_socket(NodeKey::Values(ctx.host).path(), rgba, space);
    // Below the swatch and its R, G, B, A rows: five rows of the swatch's height.
    let below = (sh.at.0, sh.at.1 + 5.0 * sh.size.1);
    BFlex::cell()
        .place(BPlace::Fraction { x: below.0, y: below.1, w: sh.size.0, h: sh.size.1 })
        .div(ctx)
        .child(crate::v2::view::draw::track::wheel_view(&wheel, ctx))
        .into_any_element()
}

/// A clickable overlay for every value target across the drawn nodes, laid over
/// the cards so a click edits the value instead of selecting the node.
///
/// `nodes` is the drawn graph with edits already applied, so each target reads
/// the CURRENT value and writes the next from it; `edits` is the raw overlay the
/// next is written on top of. The click stops propagation so the card beneath
/// neither selects nor starts a drag.
pub fn value_layer(
    nodes: &[BlendNode],
    placed: &[(String, (f32, f32))],
    view: &BNodeView,
    size: BCardSize,
    wired: &[BlendNodeLink],
    edits: &BValueEdits,
    ctx: &BRenderCtx,
) -> Vec<AnyElement> {
    // The colour socket whose inline channel picker is open, as `node/socket`.
    let picking = ctx.state.get(&NodeKey::Picking(ctx.host).path());
    let mut out = Vec::new();
    for node in nodes {
        let Some((_, at)) = placed.iter().find(|(n, _)| *n == node.name) else { continue };
        let linked = link::linked_inputs(wired, &node.name);
        // Value writes: a bool toggle, a number step, a vector or open-colour channel.
        for (i, hit) in value_targets(node, *at, size, view, &linked, picking).into_iter().enumerate() {
            let mut next = edits.clone();
            next.set(&node.name, &hit.socket, hit.set);
            let id = format!("knob-{}-{}-{i}", node.name, hit.socket);
            out.push(target(id, hit.at, hit.size, next.write(ctx.host), ctx)); // shared overlay write (`overlay`)
        }
        // Colour swatches: a click opens the picker, a second click closes it.
        for sh in swatch_hits(node, *at, size, view, &linked) {
            let open = format!("{}/{}", node.name, sh.socket);
            let is_open = picking == Some(open.as_str());
            if is_open {
                out.push(wheel(node, &sh, edits, ctx));
            }
            let next = if is_open { String::new() } else { open.clone() };
            let action = BAction::Set(NodeKey::Picking(ctx.host).path(), next);
            out.push(target(format!("swatch-{open}"), sh.at, sh.size, action, ctx));
        }
    }
    out
}
