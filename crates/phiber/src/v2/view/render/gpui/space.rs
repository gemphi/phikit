//! Workspace strip composition and focused zone-boundary seam integration.

mod seam;

use super::{color::to_hsla, ctx::BRenderCtx, walk};
use crate::v2::space::BSpaceType;
use crate::v2::state::key::K_SPACE;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::zone::BZoneType;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};
pub use seam::ZoneDrag;
use seam::{BMeasure, BSeam};
use std::sync::{Arc, Mutex};

fn is_middle(kind: BZoneType) -> bool {
    matches!(
        kind,
        BZoneType::Left | BZoneType::Center | BZoneType::Right | BZoneType::Full
    )
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

fn composition(children: &[BView]) -> BSpaceComposition<'_> {
    let mut out = BSpaceComposition {
        top: Vec::new(),
        middle: Vec::new(),
        bottom: Vec::new(),
        extras: Vec::new(),
    };
    for child in children {
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
pub fn composition_shape(children: &[BView]) -> BSpaceCompositionShape {
    let out = composition(children);
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

pub fn view(children: &[BView], c: &BRenderCtx) -> AnyElement {
    let BSpaceComposition {
        top,
        middle,
        bottom,
        extras,
    } = composition(children);
    let space =
        c.s.get(&K_SPACE)
            .and_then(BSpaceType::from_slug)
            .unwrap_or(BSpaceType::Layout);
    let mut root = div()
        .flex()
        .flex_col()
        .size_full()
        .bg(to_hsla(c.style.resize_gutter_background()))
        .text_color(to_hsla(c.style.colors.text))
        .rounded(px(c.style.radius_for(BStyleRole::Workspace)))
        .p(px(c.style.resize.gap))
        .gap(px(c.style.resize.gap));
    for child in top {
        root = root.child(walk(child, c));
    }

    let had_middle = !middle.is_empty();
    if had_middle {
        let left: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
        let right: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
        let mut row = div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h_0()
            .w_full()
            .gap(px(c.style.resize.gap));
        let mut previous = None;
        for child in middle {
            let kind = zone_kind(child).unwrap_or(BZoneType::Center);
            let seam = match (previous, kind) {
                (Some(BZoneType::Left), BZoneType::Center | BZoneType::Right) => {
                    Some((BZoneType::Left, left.clone()))
                }
                (Some(BZoneType::Center), BZoneType::Right) => {
                    Some((BZoneType::Right, right.clone()))
                }
                _ => None,
            };
            let child = walk(child, c);
            let mut wrapped = match kind {
                BZoneType::Left => BMeasure {
                    cell: left.clone(),
                    column: true,
                    child,
                }
                .render(),
                BZoneType::Right => BMeasure {
                    cell: right.clone(),
                    column: true,
                    child,
                }
                .render(),
                _ => div().flex_1().h_full().min_h_0().min_w_0().child(child),
            };
            if let Some((zone, measured)) = seam {
                wrapped = wrapped.child(
                    BSeam {
                        zone,
                        space,
                        measured,
                    }
                    .render(c),
                );
            }
            row = row.child(wrapped);
            previous = Some(kind);
        }
        root = root.child(row);
    }

    if had_middle && !bottom.is_empty() {
        let measured: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
        for (index, child) in bottom.iter().enumerate() {
            let mut wrapped = BMeasure {
                cell: measured.clone(),
                column: false,
                child: walk(child, c),
            }
            .render();
            if index == 0 {
                wrapped = wrapped.child(
                    BSeam {
                        zone: BZoneType::Bottom,
                        space,
                        measured: measured.clone(),
                    }
                    .render(c),
                );
            }
            root = root.child(wrapped);
        }
    } else {
        for child in bottom {
            root = root.child(walk(child, c));
        }
    }
    for child in extras {
        root = root.child(walk(child, c));
    }
    root.into_any_element()
}
