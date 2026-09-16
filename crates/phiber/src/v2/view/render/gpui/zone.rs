//! Zone layout and focused area-boundary seam composition.

mod seam;

use super::{ctx::BRenderCtx, walk};
use crate::v2::space::BSpaceType;
use crate::v2::state::{area_fraction, key::K_SPACE};
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::render::{BAxis, BZoneLayout};
use crate::v2::zone::BZoneType;
use gpui::prelude::*;
use gpui::{AnyElement, Bounds, Div, Pixels, div, px};
pub use seam::AreaDrag;
use seam::BSeam;
use std::sync::{Arc, Mutex};

fn size_for(kind: BZoneType, element: Div, c: &BRenderCtx) -> Div {
    match kind {
        BZoneType::Top => element.h(px(c.style.dimensions.header_height)).w_full(),
        BZoneType::Bottom => element
            .h(px(c.style.dimensions.bottom_strip_height))
            .w_full(),
        BZoneType::Center | BZoneType::Full => element.flex_1().h_full().min_h_0(),
        BZoneType::Left | BZoneType::Right => element.h_full().min_h_0(),
    }
}

pub fn view(kind: BZoneType, children: &[BView], c: &BRenderCtx) -> AnyElement {
    let vertical = matches!(kind.axis(), BAxis::Row);
    let base = match kind.axis() {
        BAxis::Row => div().flex().flex_row(),
        BAxis::Column => div().flex().flex_col(),
    }
    .bg(gpui::transparent_black())
    .rounded(px(c.style.radius_for(BStyleRole::Zone)))
    .gap(px(c.style.resize.gap));
    let mut element = size_for(kind, base, c);
    let space =
        c.s.get(&K_SPACE)
            .and_then(BSpaceType::from_slug)
            .unwrap_or(BSpaceType::Layout);
    let explicit_size = crate::v2::state::zone_size(c.s, space, kind);
    if let Some(size) = explicit_size {
        element = match kind {
            BZoneType::Left | BZoneType::Right => element.w(px(size)).flex_shrink_0(),
            BZoneType::Bottom => element.h(px(size)).flex_shrink_0(),
            _ => element,
        };
    } else if matches!(kind, BZoneType::Left | BZoneType::Right) {
        // No explicit width: pin the side column to a default width instead
        // of letting it size to content. Content-sizing lets a greedy host
        // (File Browser, Image editor — flex Custom regions) expand to fill
        // the whole window, starving the Center zone (its canvas collapsed to
        // 0 px wide, pushing the opposite column off-screen). A fixed default
        // keeps Center's flex_1 share intact and matches Blender's saved
        // editor column widths.
        element = element
            .w(px(c.style.dimensions.side_zone_default_width))
            .flex_shrink_0();
    }

    let child_count = children.iter().filter(|child| !child.is_empty()).count();
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

    let mut area_index = 0usize;
    for child in children {
        if child.is_empty() {
            continue;
        }
        let fraction = area_fraction(c.s, space, kind, area_index);
        let selector = format!("slot-{}-{area_index}", kind.slug());
        let mut slot = div()
            .id(gpui::SharedString::from(selector.clone()))
            .debug_selector(move || selector.clone())
            .flex()
            .flex_col()
            .min_w_0()
            .min_h_0();
        {
            let style = slot.style();
            style.flex_grow = Some(fraction);
            style.flex_shrink = Some(1.0);
            style.flex_basis = Some(gpui::relative(0.0).into());
        }
        slot = slot.child(walk(child, c));
        if area_index > 0 && child_count > 1 {
            slot = slot.child(
                BSeam {
                    kind,
                    index: area_index - 1,
                    space,
                    extent: zone_extent.clone(),
                }
                .render(c),
            );
        }
        element = element.child(slot);
        area_index += 1;
    }
    element.into_any_element()
}
