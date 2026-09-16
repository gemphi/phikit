//! Axis compass gizmo that tracks viewport yaw and preserves snap actions.

mod axis;

use self::axis::BAxis;
use super::super::super::super::{color::to_hsla, ctx::BRenderCtx};
use crate::v2::scene::view_yaw;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

pub struct BCompass;

impl BCompass {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let axis_x = to_hsla(c.style.colors.axis_x);
        let axis_y = to_hsla(c.style.colors.axis_y);
        let axis_z = to_hsla(c.style.colors.axis_z);
        let yaw = view_yaw(c.s).to_radians();
        let (sin_yaw, cos_yaw) = yaw.sin_cos();

        // Compass geometry scales from resolved gizmo dimensions. Projection
        // ratios below are mechanics of the yaw visualization, not UI tokens.
        let cap = c.style.dimensions.gizmo_icon_size + c.style.strokes.border * 3.0;
        let box_size = c.style.dimensions.gizmo_button_size * 3.0 + c.style.strokes.border * 2.0;
        let center = box_size / 2.0;
        let ring = c.style.dimensions.gizmo_button_size + c.style.strokes.border;
        let x_dx = cos_yaw * ring;
        let x_depth = sin_yaw;
        let y_dx = -sin_yaw * ring;
        let y_depth = cos_yaw;
        let x_dy = -x_depth * ring * 0.42;
        let y_dy = -y_depth * ring * 0.42;

        let caps = [
            BAxis {
                letter: "X",
                color: axis_x,
                cx: center + x_dx,
                cy: center + x_dy,
                depth: x_depth,
                positive: true,
                cap,
            },
            BAxis {
                letter: "X",
                color: axis_x,
                cx: center - x_dx,
                cy: center - x_dy,
                depth: -x_depth,
                positive: false,
                cap,
            },
            BAxis {
                letter: "Y",
                color: axis_y,
                cx: center + y_dx,
                cy: center + y_dy,
                depth: y_depth,
                positive: true,
                cap,
            },
            BAxis {
                letter: "Y",
                color: axis_y,
                cx: center - y_dx,
                cy: center - y_dy,
                depth: -y_depth,
                positive: false,
                cap,
            },
            BAxis {
                letter: "Z",
                color: axis_z,
                cx: center,
                cy: center - ring,
                depth: 0.4,
                positive: true,
                cap,
            },
            BAxis {
                letter: "Z",
                color: axis_z,
                cx: center,
                cy: center + ring,
                depth: -0.4,
                positive: false,
                cap,
            },
        ];
        let mut order: Vec<usize> = (0..caps.len()).collect();
        order.sort_by(|&left, &right| {
            caps[left]
                .depth
                .partial_cmp(&caps[right].depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let hub_size = (ring + cap / 2.0) * 2.0;
        let hub = div()
            .absolute()
            .top(px(center - hub_size / 2.0))
            .left(px(center - hub_size / 2.0))
            .w(px(hub_size))
            .h(px(hub_size))
            .rounded_full()
            .bg(to_hsla(c.style.colors.bg_modal_dim))
            .border(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.border));
        let dot_size = cap * 0.34;
        let hub_dot = div()
            .absolute()
            .top(px(center - dot_size / 2.0))
            .left(px(center - dot_size / 2.0))
            .w(px(dot_size))
            .h(px(dot_size))
            .rounded_full()
            .bg(to_hsla(c.style.colors.text_dim));

        let mut root = div()
            .id(SharedString::from("compass"))
            .debug_selector(|| "gizmo-compass".to_string())
            .relative()
            .w(px(box_size))
            .h(px(box_size))
            .child(hub)
            .child(BAxis::spoke(
                axis_x, x_dx, x_dy, x_depth, box_size, center, c,
            ))
            .child(BAxis::spoke(
                axis_y, y_dx, y_dy, y_depth, box_size, center, c,
            ))
            .child(BAxis::spoke(axis_z, 0.0, -ring, 0.4, box_size, center, c))
            .child(hub_dot);
        for index in order {
            root = root.child(caps[index].render(c));
        }
        root.into_any_element()
    }
}
