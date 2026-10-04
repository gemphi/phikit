//! The view as the editor keeps it: read from the store, written by a gesture.
//!
//! Blender keeps `cur` in `SpaceNode.v2d`, which lives in the .blend — the file
//! remembers where you were looking. phiber's equivalent is a state key, because
//! the file is read-only here and every other node edit is an overlay over it
//! (`state/key/node.rs` says why for the selection and the drag).
//!
//! The opening framing is the interesting part. With no stored view this returns
//! [`frame::view_all`] of the whole graph, which is exactly what the canvas used
//! to do on EVERY frame. That is the change in one sentence: fitting the graph
//! stopped being the only framing there is and became the one the editor opens
//! on, the way `NODE_OT_view_all` is one framing among many in Blender.

use super::{BNodeView, frame};
use crate::phiphx::blend::shade::graph::BlendNode;
use crate::v2::host::BHostType;
use crate::v2::state::{BKey as _, BState, NodeKey};
use std::cell::RefCell;

thread_local! {
    /// The canvas's size as the last paint pass measured it, per host.
    ///
    /// The view has to be reshaped to the region's aspect before anything is
    /// placed (`BNodeView::fitted_to`, Blender's
    /// `ui_view2d_curRect_validate_resize`) — and the element tree is built one
    /// phase BEFORE the canvas is measured, so the only size available when the
    /// cards are made is the one the last frame saw.
    ///
    /// That is a frame of lag, and only on a resize: the first frame after the
    /// pane changes shape is placed against the old aspect and the next is
    /// right. Blender has no equivalent because it validates `cur` when the
    /// region changes rather than while building a widget tree. The alternative
    /// here is worse — placing cards by fraction and sizing them by a zoom
    /// derived from a different aspect, which slides sockets off their cards.
    static MEASURED: RefCell<Vec<(BHostType, (f32, f32))>> = const { RefCell::new(Vec::new()) };
}

/// The paint pass reporting the canvas's size, for the next frame to place
/// against.
pub fn measured(host: BHostType, region: (f32, f32)) {
    if region.0 <= 0.0 || region.1 <= 0.0 {
        return;
    }
    MEASURED.with_borrow_mut(|seen| match seen.iter_mut().find(|(h, _)| *h == host) {
        Some(entry) => entry.1 = region,
        None => seen.push((host, region)),
    });
}

/// What the last paint pass measured for `host`, if it has painted.
pub fn region_of(host: BHostType) -> Option<(f32, f32)> {
    MEASURED.with_borrow(|seen| seen.iter().find(|(h, _)| *h == host).map(|(_, r)| *r))
}

/// `"xmin, ymin, xmax, ymax"` — the spelling `drag::encode` uses for a pair,
/// widened to a rectangle, so the state log holds one format for a tuple.
pub fn encode(view: &BNodeView) -> String {
    format!("{}, {}, {}, {}", view.xmin, view.ymin, view.xmax, view.ymax)
}

/// A stored view back. Anything unreadable — or degenerate, which would make
/// every fraction a division by zero — reads as no stored view at all, so the
/// editor falls back to framing the graph rather than painting nothing.
pub fn decode(raw: &str) -> Option<BNodeView> {
    let mut parts = raw.split(',').map(|p| p.trim().parse::<f32>());
    let mut next = || parts.next()?.ok();
    let view = BNodeView { xmin: next()?, ymin: next()?, xmax: next()?, ymax: next()? };
    let sane = view.width() > 1e-3
        && view.height() > 1e-3
        && [view.xmin, view.ymin, view.xmax, view.ymax].iter().all(|v| v.is_finite());
    sane.then_some(view)
}

/// What this host is looking at: the stored view, or the framing it opens on.
pub fn view_of(
    state: &BState,
    host: BHostType,
    nodes: &[BlendNode],
    height_of: impl Fn(&BlendNode) -> f32,
    card_width: f32,
) -> BNodeView {
    let view = state
        .get(&NodeKey::View(host).path())
        .and_then(decode)
        .unwrap_or_else(|| opening(nodes, height_of, card_width));
    // Reshaped to the canvas it will be drawn on, so the fraction the cards are
    // placed by and the zoom the painter sizes them by are the same mapping.
    match region_of(host) {
        Some(region) => view.fitted_to(region),
        None => view,
    }
}

/// The framing a graph opens on — `view_all` over the whole tree.
///
/// `single_node` is Blender's own test and matters here: a material with one
/// node is centred at the default zoom rather than filled with one enormous
/// card, which is what fitting it would have done.
pub fn opening(
    nodes: &[BlendNode],
    height_of: impl Fn(&BlendNode) -> f32,
    card_width: f32,
) -> BNodeView {
    let cur = BNodeView::default();
    let bounds = frame::graph_bounds(nodes, height_of, card_width);
    frame::view_all(bounds, &cur, nodes.len() == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(name: &str, x: f32, y: f32) -> BlendNode {
        BlendNode { name: name.into(), x, y, width: 140.0, ..Default::default() }
    }

    #[test]
    fn a_view_survives_the_round_trip_through_the_store() {
        let view = BNodeView { xmin: -12.5, ymin: 3.25, xmax: 400.0, ymax: 240.5 };
        assert_eq!(decode(&encode(&view)), Some(view));
    }

    #[test]
    fn nonsense_reads_as_no_stored_view_rather_than_a_broken_one() {
        // A degenerate rectangle would divide by zero in every fraction, so it
        // is refused here rather than guarded at each use.
        for raw in ["", "1, 2", "a, b, c, d", "0, 0, 0, 0", "5, 5, 5, 9", "inf, 0, 1, 1"] {
            assert_eq!(decode(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn with_nothing_stored_the_editor_opens_framed_on_the_graph() {
        // The old behaviour, kept — but now as the opening framing rather than
        // a fit recomputed on every frame.
        let state = BState::default();
        let nodes = vec![node("a", 0.0, 0.0), node("b", 400.0, 300.0)];
        let view = view_of(&state, BHostType::Shader, &nodes, |_| 80.0, 240.0);
        let bounds = frame::graph_bounds(&nodes, |_| 80.0, 240.0);
        assert!(view.xmin < bounds.0 && view.xmax > bounds.2, "{view:?} does not contain {bounds:?}");
        assert!(view.ymin < bounds.1 && view.ymax > bounds.3);
    }

    #[test]
    fn a_stored_view_wins_over_the_opening_framing() {
        let mut state = BState::default();
        let stored = BNodeView { xmin: 10.0, ymin: 20.0, xmax: 110.0, ymax: 120.0 };
        state.set(NodeKey::View(BHostType::Shader), &encode(&stored));
        let nodes = vec![node("a", 0.0, 0.0), node("b", 4000.0, 3000.0)];
        assert_eq!(view_of(&state, BHostType::Shader, &nodes, |_| 80.0, 240.0), stored);
    }

    #[test]
    fn each_editor_is_looking_somewhere_of_its_own() {
        // Three `edittree`s, three views — the same reason the selection is
        // keyed by host.
        let mut state = BState::default();
        let stored = BNodeView { xmin: 10.0, ymin: 20.0, xmax: 110.0, ymax: 120.0 };
        state.set(NodeKey::View(BHostType::Shader), &encode(&stored));
        let nodes = vec![node("a", 0.0, 0.0), node("b", 400.0, 300.0)];
        assert_eq!(view_of(&state, BHostType::Shader, &nodes, |_| 80.0, 240.0), stored);
        assert_ne!(
            view_of(&state, BHostType::Compositor, &nodes, |_| 80.0, 240.0),
            stored,
            "the compositor is not looking through the shader editor's window",
        );
    }

    #[test]
    fn an_empty_graph_opens_on_the_default_window() {
        // `view_all` refuses an empty tree, so what is left is `tot` — the
        // window a new tree is created in.
        assert_eq!(opening(&[], |_| 80.0, 240.0), BNodeView::default());
    }
}
