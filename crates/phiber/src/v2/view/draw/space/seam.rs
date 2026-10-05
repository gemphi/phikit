//! Zone-boundary resize payload, measurement, and zero-footprint seam.

use super::super::flex::{BFlex, BFlexed, BLen, BSpace};
use super::super::ctx::BRenderCtx;
use crate::v2::space::BSpaceType;
use crate::v2::zone::{zone_size, BZoneType};
use gpui::prelude::*;
use gpui::{AnyElement, Bounds, Div, IntoElement, Pixels, Point, SharedString, div};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ZoneDrag {
    pub space: BSpaceType,
    pub zone: BZoneType,
    pub sign: f32,
    pub anchor: Option<f32>,
    pub origin: Arc<Mutex<Option<Point<Pixels>>>>,
    pub measured: Arc<Mutex<Option<f32>>>,
    pub snap_threshold: f32,
    pub min_extent: f32,
    pub dragging: Arc<AtomicBool>,
}

impl ZoneDrag {
    pub fn is_column(&self) -> bool {
        matches!(self.zone, BZoneType::Left | BZoneType::Right)
    }

    pub fn total(&self, now: Point<Pixels>) -> f32 {
        let mut guard = self.origin.lock().unwrap();
        let origin = *guard.get_or_insert(now);
        if self.is_column() {
            f32::from(now.x - origin.x)
        } else {
            f32::from(now.y - origin.y)
        }
    }

    pub fn anchor_size(&self) -> f32 {
        self.anchor
            .filter(|anchor| *anchor > 1.0)
            .or_else(|| *self.measured.lock().unwrap())
            .unwrap_or(0.0)
    }
}

struct BGhost {
    flag: Arc<AtomicBool>,
}

impl Drop for BGhost {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::Relaxed);
    }
}

impl gpui::Render for BGhost {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div()
    }
}

pub(super) struct BMeasure {
    pub cell: Arc<Mutex<Option<f32>>>,
    pub column: bool,
    /// Follows another zone, so it keeps the gutter before it.
    pub lead: bool,
    pub child: AnyElement,
}

impl BFlexed for BMeasure {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        let flex = BFlex::cell().keeps_size().unclamped();
        let flex = if self.column {
            flex.height(BLen::Full)
        } else {
            flex.width(BLen::Full)
        };
        if self.lead {
            flex.margin_left(BSpace::Seam)
        } else {
            flex
        }
    }
}

impl BMeasure {
    pub fn render(self, ctx: &BRenderCtx) -> Div {
        let frame = self.flex(ctx);
        let BMeasure {
            cell,
            column,
            child,
            ..
        } = self;
        frame
            .div(ctx)
            .on_children_prepainted(move |bounds: Vec<Bounds<Pixels>>, _, _| {
                if let Some(bound) = bounds.first() {
                    let value = f32::from(if column {
                        bound.size.width
                    } else {
                        bound.size.height
                    });
                    if value > 1.0 {
                        *cell.lock().unwrap() = Some(value);
                    }
                }
            })
            .child(child)
    }
}

pub(super) struct BSeam {
    pub zone: BZoneType,
    pub space: BSpaceType,
    pub measured: Arc<Mutex<Option<f32>>>,
}

impl BSeam {
    pub fn render(self, ctx: &BRenderCtx) -> AnyElement {
        let column = matches!(self.zone, BZoneType::Left | BZoneType::Right);
        let group = SharedString::from(format!("zone-seam-{}", self.zone.slug()));
        let dragging = Arc::new(AtomicBool::new(false));
        let line = super::super::resize::BSeamLine { vertical: column, group: group.clone() }.render(ctx);
        let selector = format!("zone-seam-{}", self.zone.slug());
        let mut handle = div()
            .id(SharedString::from(selector.clone()))
            .group(group)
            .debug_selector(move || selector)
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .occlude()
            .child(line)
            .cursor(if column {
                gpui::CursorStyle::ResizeLeftRight
            } else {
                gpui::CursorStyle::ResizeUpDown
            });
        let payload = ZoneDrag {
            space: self.space,
            zone: self.zone,
            sign: if self.zone == BZoneType::Left {
                1.0
            } else {
                -1.0
            },
            anchor: zone_size(ctx.state, self.space, self.zone),
            origin: Arc::new(Mutex::new(None)),
            measured: self.measured,
            snap_threshold: ctx.style.resize.snap_threshold,
            min_extent: ctx.style.resize.min_extent,
            dragging: dragging,
        };
        handle = handle.on_drag(payload, |drag: &ZoneDrag, _, _, cx| {
            drag.dragging.store(true, Ordering::Relaxed);
            let flag = drag.dragging.clone();
            cx.stop_propagation();
            cx.new(move |_| BGhost { flag })
        });
        // One seam geometry for the whole app: a zone boundary, an area
        // boundary and a panel edge are the same zero-footprint overlay
        // (`resize::overlay`).
        super::super::resize::overlay(column, true, ctx)
            .child(handle)
            .into_any_element()
    }
}
