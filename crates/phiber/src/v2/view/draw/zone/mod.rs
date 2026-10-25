//! Zone layout and focused area-boundary seam composition.

mod seam;

use gpui::prelude::*;
use gpui::{AnyElement, Bounds, Pixels};
use crate::v2::area::area_fraction;
use crate::v2::space::BSpaceType;
use crate::v2::state::{CtxKey};
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::layout::{BAxis, BZoneLayout};
use crate::v2::zone::BZoneType;
use super::flex::{BDim, BFill, BFlex, BFlexed, BLen, BRadius, BSpace};
use super::ctx::BRenderCtx;
use super::walk;
pub use seam::AreaDrag;
use seam::BSeam;
use std::sync::{Arc, Mutex};
/// The parts of a zone — one enum for the level, as every other level states
/// its parts (`BCardPart`, `BModalPart`, `BAreaPart`).
enum BZonePart {
    /// The zone frame: its kind, a size the user dragged, whether all it
    /// holds is a fixed strip, and whether all it holds is a fixed bar (the
    /// status bar's zone, which is one bar tall whatever the Timeline's band
    /// was dragged to).
    Frame { kind: BZoneType, dragged: Option<f32>, fixed_strip: bool, fixed_bar: bool, browsing: bool },
    /// One area's slot in a zone: its share of the zone's length, or — for an
    /// area that is one bar tall — exactly the bar's height.
    Slot { share: f32, fixed_bar: bool },
}

impl BFlexed for BZonePart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        use BZoneType::*;
        match *self {
            Self::Slot { fixed_bar: true, .. } => BFlex::column().unclamped().keeps_size(),
            Self::Slot { share, .. } => BFlex::column().unclamped().shares(share),
            Self::Frame { kind, dragged, fixed_strip, fixed_bar, browsing } => {
                let base = BFlex::along(kind.axis())
                    .fill(BFill::Clear)
                    .radius(BRadius::Role(BStyleRole::Zone))
                    .gap(BSpace::Seam);
                let sized = match kind {
                    // A top bar wrapping onto a second line (`filter::topbar_wraps`)
                    // grows with it: at one bar's height the second line was
                    // painted under the editors below and could not be seen.
                    Top if crate::v2::part::filter::topbar_wraps(_c.state) => base.min_height(BLen::Dim(BDim::HeaderHeight)).width(BLen::Full).keeps_size(),
                    Top => base.height(BLen::Dim(BDim::HeaderHeight)).width(BLen::Full),
                    Bottom if fixed_bar => return base.width(BLen::Full).keeps_size(),
                    Bottom if browsing => base.height(BLen::Dim(BDim::BottomPanelHeight)).width(BLen::Full),
                    Bottom => base.height(BLen::Dim(BDim::BottomStripHeight)).width(BLen::Full),
                    Center | Full => base.fills().height(BLen::Full).min_height(BLen::Zero),
                    Left | Right | Rail => base.height(BLen::Full).min_height(BLen::Zero),
                };
                match (dragged, kind) {
                    (Some(size), Left | Right) => sized.width(BLen::Measured(size)).keeps_size(),
                    (Some(size), Bottom) => sized.height(BLen::Measured(size)).keeps_size(),
                    (Some(_), _) => sized,
                    // A zone whose only content is a fixed-width strip hugs
                    // it. Giving it the side-column default left a wide empty
                    // band beside the Activity rail, which is not a column
                    // anyone can put anything in.
                    (None, _) if fixed_strip => sized.keeps_size(),
                    // No dragged width: pin a side column to a default
                    // instead of letting it size to content. Content-sizing
                    // let a greedy host (File Browser, Image editor) expand
                    // to fill the window and starve the Center zone; a
                    // default keeps Center's share intact and matches
                    // Blender's saved editor column widths.
                    (None, Left | Right) => {
                        sized.width(BLen::Dim(BDim::SideZoneDefaultWidth)).keeps_size()
                    }
                    (None, _) => sized,
                }
            }
        }
    }
}

/// The host an area slot holds, when it holds one.
pub(super) fn area_host(child: &BView) -> Option<crate::v2::host::BHostType> {
    let BView::Area { child, .. } = child else { return None };
    let BView::Host { kind, .. } = child.as_ref() else { return None };
    Some(*kind)
}

/// True when every area in this zone hosts a bar one header tall — the
/// status bar's zone — so the zone is exactly that tall.
pub(super) fn fixed_bar_zone(list: &[BView]) -> bool {
    let mut any = false;
    for child in list.iter().filter(|child| !child.is_empty()) {
        any = true;
        if !area_host(child).is_some_and(|host| host.is_fixed_bar()) {
            return false;
        }
    }
    any
}

/// True when every area in this zone hosts a fixed-width strip, so the zone
/// has no width of its own to give away and no boundary to drag.
pub(super) fn fixed_strip_zone(list: &[BView]) -> bool {
    let mut any = false;
    for child in list.iter().filter(|child| !child.is_empty()) {
        any = true;
        let BView::Area { child, .. } = child else {
            return false;
        };
        let BView::Host { kind, .. } = child.as_ref() else {
            return false;
        };
        if !kind.is_fixed_strip() {
            return false;
        }
    }
    any
}

pub fn view(kind: BZoneType, list: &[BView], ctx: &BRenderCtx) -> AnyElement {
    let vertical = matches!(kind.axis(), BAxis::Row);
    let space =
        ctx.state.get(CtxKey::Space)
            .and_then(BSpaceType::from_slug)
            .unwrap_or(BSpaceType::Layout);
    let frame = BZonePart::Frame {
        kind,
        dragged: crate::v2::zone::zone_size(ctx.state, space, kind),
        fixed_strip: fixed_strip_zone(list),
        fixed_bar: fixed_bar_zone(list),
        browsing: list.iter().filter_map(area_host).any(|host| host.browses()),
    };
    let mut element = frame.flex(ctx).div(ctx);
    // The top zone is the window's title bar (the native one is hidden):
    // empty space in it drags the window.
    if kind == BZoneType::Top {
        element = element.window_control_area(gpui::WindowControlArea::Drag);
    }

    let child_count = list.iter().filter(|child| !child.is_empty()).count();

    let zone_extent: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
    if child_count > 1 {
        let extent = zone_extent.clone();
        element =
            element.on_children_prepainted(move |bounds: Vec<Bounds<Pixels>>, _window, _app| {
                if bounds.is_empty() {
                    return;
                }
                let mut start = f32::INFINITY;
                let mut end = f32::NEG_INFINITY;
                for bound in &bounds {
                    let (low, high) = if vertical {
                        (
                            f32::from(bound.origin.x),
                            f32::from(bound.origin.x + bound.size.width),
                        )
                    } else {
                        (
                            f32::from(bound.origin.y),
                            f32::from(bound.origin.y + bound.size.height),
                        )
                    };
                    start = start.min(low);
                    end = end.max(high);
                }
                if end - start > 1.0 {
                    *extent.lock().unwrap() = Some(end - start);
                }
            });
    }

    // The areas that share the zone's length divide the room the fixed bars
    // leave; their shares are scaled to sum to one, because flex only hands
    // out all of the free space when the grow factors reach one.
    let shared: f32 = list
        .iter()
        .filter(|child| !child.is_empty())
        .enumerate()
        .filter(|(_, child)| !area_host(child).is_some_and(|host| host.is_fixed_bar()))
        .map(|(index, _)| area_fraction(ctx.state, space, kind, index))
        .sum();
    let mut area_index = 0usize;
    for child in list {
        if child.is_empty() {
            continue;
        }
        let fraction = area_fraction(ctx.state, space, kind, area_index) / shared.max(f32::EPSILON);
        let fixed_bar = area_host(child).is_some_and(|host| host.is_fixed_bar());
        let selector = format!("slot-{}-{area_index}", kind.slug());
        let mut slot = BZonePart::Slot { share: fraction, fixed_bar }
            .flex(ctx)
            .div(ctx)
            .id(gpui::SharedString::from(selector.clone()))
            .debug_selector(move || selector)
            .child(walk(child, ctx));
        if area_index > 0 && child_count > 1 {
            slot = slot.child(
                BSeam {
                    kind,
                    index: area_index - 1,
                    space,
                    extent: zone_extent.clone(),
                }
                .render(ctx),
            );
        }
        element = element.child(slot);
        area_index += 1;
    }
    element.into_any_element()
}
