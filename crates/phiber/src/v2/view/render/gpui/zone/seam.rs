//! Area-boundary resize payload and zero-footprint seam.

use super::super::{color::to_hsla, ctx::BRenderCtx};
use crate::v2::space::BSpaceType;
use crate::v2::state::boundary_pair;
use crate::v2::view::render::{BAxis, BZoneLayout};
use crate::v2::zone::BZoneType;
use gpui::prelude::*;
use gpui::{
    AnyElement, Context, IntoElement, Pixels, Point, Render, SharedString, Window, div, px,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AreaDrag {
    pub space: BSpaceType,
    pub zone: BZoneType,
    pub index: usize,
    pub vertical: bool,
    pub anchor_i: f32,
    pub anchor_j: f32,
    pub origin: Arc<Mutex<Option<Point<Pixels>>>>,
    pub zone_extent: Arc<Mutex<Option<f32>>>,
    pub snap_threshold: f32,
    pub min_extent: f32,
    pub dragging: Arc<AtomicBool>,
}

impl AreaDrag {
    pub fn total(&self, now: Point<Pixels>) -> f32 {
        let mut guard = self.origin.lock().unwrap();
        let origin = *guard.get_or_insert(now);
        if self.vertical {
            f32::from(now.x - origin.x)
        } else {
            f32::from(now.y - origin.y)
        }
    }

    pub fn extent_or(&self, fallback: f32) -> f32 {
        self.zone_extent
            .lock()
            .unwrap()
            .filter(|extent| *extent > 1.0)
            .unwrap_or(fallback)
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

impl Render for BGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub(super) struct BSeam {
    pub kind: BZoneType,
    pub index: usize,
    pub space: BSpaceType,
    pub extent: Arc<Mutex<Option<f32>>>,
}

impl BSeam {
    pub fn render(self, c: &BRenderCtx) -> AnyElement {
        let vertical = matches!(self.kind.axis(), BAxis::Row);
        let seam = px(c.style.resize.visible_seam);
        let base = to_hsla(c.style.resize_seam_resting_color());
        let active = to_hsla(c.style.resize_seam_active_color());
        let group = SharedString::from(format!("resize-group-{}-{}", self.kind.slug(), self.index));
        let dragging = Arc::new(AtomicBool::new(false));
        let line = div()
            .bg(base)
            .group_hover(group.clone(), move |style| style.bg(active));
        let line = if vertical {
            line.w(seam).h_full()
        } else {
            line.h(seam).w_full()
        };
        let selector = format!("resize-{}-{}", self.kind.slug(), self.index);
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
            .cursor(if vertical {
                gpui::CursorStyle::ResizeLeftRight
            } else {
                gpui::CursorStyle::ResizeUpDown
            });
        let (anchor_i, anchor_j) =
            boundary_pair(c.s, self.space, self.kind, self.index).unwrap_or((0.5, 0.5));
        let payload = AreaDrag {
            space: self.space,
            zone: self.kind,
            index: self.index,
            vertical,
            anchor_i,
            anchor_j,
            origin: Arc::new(Mutex::new(None)),
            zone_extent: self.extent,
            snap_threshold: c.style.resize.snap_threshold,
            min_extent: c.style.resize.min_extent,
            dragging: dragging.clone(),
        };
        handle = handle.on_drag(payload, |drag: &AreaDrag, _, _, cx| {
            drag.dragging.store(true, Ordering::Relaxed);
            let flag = drag.dragging.clone();
            cx.stop_propagation();
            cx.new(move |_| BGhost { flag })
        });
        let hit = px(c.style.resize.hit_width);
        let inset = px(c.style.resize.inset);
        let overlay = if vertical {
            div().absolute().top_0().bottom_0().left(inset).w(hit)
        } else {
            div().absolute().left_0().right_0().top(inset).h(hit)
        };
        overlay.child(handle).into_any_element()
    }
}
