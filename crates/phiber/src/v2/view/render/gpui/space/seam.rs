//! Zone-boundary resize payload, measurement, and zero-footprint seam.

use super::super::{color::to_hsla, ctx::BRenderCtx};
use crate::v2::space::BSpaceType;
use crate::v2::state::zone_size;
use crate::v2::zone::BZoneType;
use gpui::prelude::*;
use gpui::{AnyElement, Bounds, Div, IntoElement, Pixels, Point, SharedString, div, px};
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
    pub child: AnyElement,
}

impl BMeasure {
    pub fn render(self) -> Div {
        let BMeasure {
            cell,
            column,
            child,
        } = self;
        let measured = div()
            .flex_shrink_0()
            .min_h_0()
            .min_w_0()
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
            .child(child);
        if column {
            measured.h_full()
        } else {
            measured.w_full()
        }
    }
}

pub(super) struct BSeam {
    pub zone: BZoneType,
    pub space: BSpaceType,
    pub measured: Arc<Mutex<Option<f32>>>,
}

impl BSeam {
    pub fn render(self, c: &BRenderCtx) -> AnyElement {
        let column = matches!(self.zone, BZoneType::Left | BZoneType::Right);
        let stroke = px(c.style.resize.visible_seam);
        let base = to_hsla(c.style.resize_seam_resting_color());
        let active = to_hsla(c.style.resize_seam_active_color());
        let group = SharedString::from(format!("zone-seam-{}", self.zone.slug()));
        let dragging = Arc::new(AtomicBool::new(false));
        let line = div()
            .bg(base)
            .group_hover(group.clone(), move |style| style.bg(active));
        let line = if column {
            line.w(stroke).h_full()
        } else {
            line.h(stroke).w_full()
        };
        let selector = format!("zone-seam-{}", self.zone.slug());
        let mut handle = div()
            .id(SharedString::from(selector.clone()))
            .group(group)
            .debug_selector(move || selector.clone())
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
            anchor: zone_size(c.s, self.space, self.zone),
            origin: Arc::new(Mutex::new(None)),
            measured: self.measured,
            snap_threshold: c.style.resize.snap_threshold,
            min_extent: c.style.resize.min_extent,
            dragging: dragging.clone(),
        };
        handle = handle.on_drag(payload, |drag: &ZoneDrag, _, _, cx| {
            drag.dragging.store(true, Ordering::Relaxed);
            let flag = drag.dragging.clone();
            cx.stop_propagation();
            cx.new(move |_| BGhost { flag })
        });
        let hit = px(c.style.resize.hit_width);
        let inset = px(c.style.resize.inset);
        let overlay = if column {
            div().absolute().top_0().bottom_0().left(inset).w(hit)
        } else {
            div().absolute().left_0().right_0().top(inset).h(hit)
        };
        overlay.child(handle).into_any_element()
    }
}
