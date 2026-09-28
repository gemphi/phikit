//! The bottom band — Blender's Timeline or Dope Sheet strip. It sits under
//! the centre editor only: the side columns run down to the status bar,
//! which is the window's, so a Bottom zone is split into the bars that go
//! under the whole middle row and the editors that go under the centre.

use super::super::ctx::BRenderCtx;
use super::super::{walk, zone};
use super::seam::{BMeasure, BSeam};
use crate::v2::space::BSpaceType;
use crate::v2::view::BView;
use crate::v2::zone::BZoneType;
use gpui::prelude::*;
use std::sync::{Arc, Mutex};

/// The editors under the centre, and the bars under the row.
pub(super) fn split_bottom(list: &[&BView]) -> (Vec<BView>, Vec<BView>) {
    let mut center = Vec::new();
    let mut bars = Vec::new();
    for view in list {
        let BView::Zone { kind, list } = view else { continue };
        let (bar, rest): (Vec<BView>, Vec<BView>) =
            list.iter().cloned().partition(|area| zone::area_host(area).is_some_and(|host| host.is_fixed_bar()));
        if !rest.is_empty() {
            center.push(BView::Zone { kind: *kind, list: rest });
        }
        if !bar.is_empty() {
            bars.push(BView::Zone { kind: *kind, list: bar });
        }
    }
    (center, bars)
}

/// The bottom band under the centre editor: measured for its seam, the seam
/// on the first zone.
pub(super) fn bottom_band(list: &[BView], space: BSpaceType, ctx: &BRenderCtx) -> Vec<gpui::Div> {
    let measured: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
    list.iter()
        .enumerate()
        .map(|(index, child)| {
            let mut wrapped = BMeasure {
                cell: measured.clone(),
                column: false,
                lead: false,
                child: walk(child, ctx),
            }
            .render(ctx);
            if index == 0 {
                wrapped = wrapped.child(
                    BSeam {
                        zone: BZoneType::Bottom,
                        space,
                        measured: measured.clone(),
                    }
                    .render(ctx),
                );
            }
            wrapped
        })
        .collect()
}
