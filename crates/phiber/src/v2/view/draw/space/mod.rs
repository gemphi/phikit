//! Workspace strip composition and focused zone-boundary seam integration.

mod band;
mod part;
mod seam;

use gpui::AnyElement;
use gpui::prelude::*;
use crate::v2::space::BSpaceType;
use crate::v2::state::{CtxKey};
use crate::v2::view::BView;
use crate::v2::zone::BZoneType;
use super::flex::BFlexed;
use super::ctx::BRenderCtx;
use super::{walk, zone};
pub use seam::ZoneDrag;
use band::{bottom_band, split_bottom};
use part::BSpacePart;
use seam::{BMeasure, BSeam};
use std::sync::{Arc, Mutex};

fn is_middle(kind: BZoneType) -> bool {
    matches!(
        kind,
        BZoneType::Rail
            | BZoneType::Left
            | BZoneType::Center
            | BZoneType::Right
            | BZoneType::Full
    )
}

/// The areas inside a zone view, for asking what it actually holds.
fn zone_children(view: &BView) -> Option<&[BView]> {
    match view {
        BView::Zone { list, .. } => Some(list.as_slice()),
        _ => None,
    }
}

fn zone_kind(view: &BView) -> Option<BZoneType> {
    match view {
        BView::Zone { kind, .. } => Some(*kind),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BSpaceCompositionShape {
    pub top: Vec<BZoneType>,
    pub middle: Vec<BZoneType>,
    pub bottom: Vec<BZoneType>,
    pub extras: usize,
}

struct BSpaceComposition<'a> {
    top: Vec<&'a BView>,
    middle: Vec<&'a BView>,
    bottom: Vec<&'a BView>,
    extras: Vec<&'a BView>,
}

fn composition(list: &[BView]) -> BSpaceComposition<'_> {
    let mut out = BSpaceComposition {
        top: Vec::new(),
        middle: Vec::new(),
        bottom: Vec::new(),
        extras: Vec::new(),
    };
    for child in list {
        match zone_kind(child) {
            Some(BZoneType::Top) => out.top.push(child),
            Some(BZoneType::Bottom) => out.bottom.push(child),
            Some(kind) if is_middle(kind) => out.middle.push(child),
            _ => out.extras.push(child),
        }
    }
    out
}

/// Returns the same zone-band partition consumed by the focused renderer.
pub fn composition_shape(list: &[BView]) -> BSpaceCompositionShape {
    let out = composition(list);
    BSpaceCompositionShape {
        top: out.top.iter().filter_map(|view| zone_kind(view)).collect(),
        middle: out
            .middle
            .iter()
            .filter_map(|view| zone_kind(view))
            .collect(),
        bottom: out
            .bottom
            .iter()
            .filter_map(|view| zone_kind(view))
            .collect(),
        extras: out.extras.len(),
    }
}

pub fn view(list: &[BView], ctx: &BRenderCtx) -> AnyElement {
    let BSpaceComposition {
        top,
        middle,
        bottom,
        extras,
    } = composition(list);
    let space =
        ctx.state.get(CtxKey::Space)
            .and_then(BSpaceType::from_slug)
            .unwrap_or(BSpaceType::Layout);
    let mut root = BSpacePart::Root.flex(ctx).div(ctx);
    for child in top {
        root = root.child(walk(child, ctx));
    }

    let had_middle = !middle.is_empty();
    let (bottom_center, bottom_bars) = if had_middle { split_bottom(&bottom) } else { (Vec::new(), Vec::new()) };
    if had_middle {
        let left: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
        let right: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
        // The gap between zones exists to hold a draggable boundary, so it is
        // applied per boundary rather than uniformly: a fixed strip has no
        // boundary to drag, and a gap after it just pushed the content away
        // from the rail for no reason.
        let mut row = BSpacePart::Middle.flex(ctx).div(ctx);
        let mut previous = None;
        let mut previous_fixed = false;
        let mut band = Some(bottom_center);
        for child in middle {
            let kind = zone_kind(child).unwrap_or(BZoneType::Center);
            // A zone that only holds a fixed strip has no width to trade, so
            // the boundary beside it is not draggable.
            let fixed = zone_children(child).is_some_and(zone::fixed_strip_zone);
            let seam = match (previous, kind) {
                (Some(BZoneType::Left), BZoneType::Center | BZoneType::Right) => {
                    Some((BZoneType::Left, left.clone()))
                }
                (Some(BZoneType::Center), BZoneType::Right) => {
                    Some((BZoneType::Right, right.clone()))
                }
                _ => None,
            };
            let child = walk(child, ctx);
            // Every boundary keeps the gutter (`BSpacePart::Centre`,
            // `BMeasure`). Whether it can be *dragged* is a different
            // question: a zone holding only a fixed strip has no width to
            // trade, so it gets the gutter but no seam.
            let lead = previous.is_some();
            let mut wrapped = match kind {
                BZoneType::Left => BMeasure {
                    cell: left.clone(),
                    column: true,
                    lead,
                    child,
                }
                .render(ctx),
                BZoneType::Right => BMeasure {
                    cell: right.clone(),
                    column: true,
                    lead,
                    child,
                }
                .render(ctx),
                BZoneType::Rail => BSpacePart::Strip { lead }.flex(ctx).div(ctx).child(child),
                _ => {
                    // The centre column: the editor, and the bottom band under it.
                    let mut column = BSpacePart::Centre { lead }.flex(ctx).div(ctx).child(child);
                    if let Some(list) = band.take() {
                        for wrapped in bottom_band(&list, space, ctx) {
                            column = column.child(wrapped);
                        }
                    }
                    column
                }
            };
            let seam = if fixed || previous_fixed { None } else { seam };
            if let Some((zone, measured)) = seam {
                wrapped = wrapped.child(
                    BSeam {
                        zone,
                        space,
                        measured,
                    }
                    .render(ctx),
                );
            }
            row = row.child(wrapped);
            previous = Some(kind);
            previous_fixed = fixed;
        }
        root = root.child(row);
        // A space with no centre zone keeps its band under the row.
        if let Some(list) = band.take() {
            for wrapped in bottom_band(&list, space, ctx) {
                root = root.child(wrapped);
            }
        }
        for child in &bottom_bars {
            root = root.child(walk(child, ctx));
        }
    } else {
        for child in bottom {
            root = root.child(walk(child, ctx));
        }
    }
    for child in extras {
        root = root.child(walk(child, ctx));
    }
    root.into_any_element()
}
