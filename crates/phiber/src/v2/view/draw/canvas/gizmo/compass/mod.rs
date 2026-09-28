//! Axis compass gizmo that tracks viewport yaw and preserves snap actions.

mod axis;

use self::axis::BAxis;
use super::super::super::color::to_hsla;
use super::super::super::ctx::BRenderCtx;
use crate::v2::scene::{project3_full, view_pitch, view_yaw};
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

pub struct BCompass;

impl BCompass {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let axis_x = to_hsla(ctx.style.colors.axis_x);
        let axis_y = to_hsla(ctx.style.colors.axis_y);
        let axis_z = to_hsla(ctx.style.colors.axis_z);
        // The compass is the scene's own projection of the three world axes,
        // so it turns with yaw AND tips with pitch — looking down the Z axis
        // puts Z in the middle, exactly as the viewport shows it.
        let (yaw, pitch) = (view_yaw(ctx.state), view_pitch(ctx.state));
        // World (Blender Z-up) → the projector's (x, up, z).
        let axis_at = |v: (f32, f32, f32)| {
            let (sx, sy, depth) = project3_full(v.0, v.2, v.1, yaw, pitch, (0.0, 0.0), 1.0);
            let (ox, oy, _) = project3_full(0.0, 0.0, 0.0, yaw, pitch, (0.0, 0.0), 1.0);
            (sx - ox, sy - oy, depth)
        };

        // Compass geometry scales from resolved gizmo dimensions. Projection
        // ratios below are mechanics of the yaw visualization, not UI tokens.
        let cap = ctx.style.dimensions.gizmo_icon_size + ctx.style.strokes.border * 3.0;
        let box_size = ctx.style.dimensions.gizmo_button_size * 3.0 + ctx.style.strokes.border * 2.0;
        let center = box_size / 2.0;
        let ring = ctx.style.dimensions.gizmo_button_size + ctx.style.strokes.border;
        // Each axis in screen space, scaled so the farthest cap sits on the ring.
        let (ax, ay, az) = (axis_at((1.0, 0.0, 0.0)), axis_at((0.0, 1.0, 0.0)), axis_at((0.0, 0.0, 1.0)));
        let longest = [ax, ay, az]
            .iter()
            .map(|a| (a.0 * a.0 + a.1 * a.1).sqrt())
            .fold(1e-6f32, f32::max);
        let scale = ring / longest;
        let (x_dx, x_dy, x_depth) = (ax.0 * scale, ax.1 * scale, ax.2);
        let (y_dx, y_dy, y_depth) = (ay.0 * scale, ay.1 * scale, ay.2);
        let (z_dx, z_dy, z_depth) = (az.0 * scale, az.1 * scale, az.2);

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
                cx: center + z_dx,
                cy: center + z_dy,
                depth: z_depth,
                positive: true,
                cap,
            },
            BAxis {
                letter: "Z",
                color: axis_z,
                cx: center - z_dx,
                cy: center - z_dy,
                depth: -z_depth,
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
            // The hub is a sphere in the same fill as the toolbar and sidebar
            // (`pane::floating_bg`), not a dark disc of its own.
            .bg(crate::v2::view::draw::pane::floating_bg(ctx))
            .border(px(ctx.style.strokes.border))
            .border_color(to_hsla(ctx.style.colors.border));
        let dot_size = cap * 0.34;
        let hub_dot = div()
            .absolute()
            .top(px(center - dot_size / 2.0))
            .left(px(center - dot_size / 2.0))
            .w(px(dot_size))
            .h(px(dot_size))
            .rounded_full()
            .bg(to_hsla(ctx.style.colors.text_dim));

        let mut root = div()
            .id(SharedString::from("compass"))
            .debug_selector(|| "gizmo-compass".to_string())
            .relative()
            .w(px(box_size))
            .h(px(box_size))
            .child(hub)
            .child(BAxis::spoke(
                axis_x, x_dx, x_dy, x_depth, box_size, center, ctx,
            ))
            .child(BAxis::spoke(
                axis_y, y_dx, y_dy, y_depth, box_size, center, ctx,
            ))
            // Z is projected like X and Y. Pinning it upright with a fixed
            // depth was why the compass never looked like it tipped: the
            // caps moved with pitch but the axis they sat on did not.
            .child(BAxis::spoke(
                axis_z, z_dx, z_dy, z_depth, box_size, center, ctx,
            ))
            .child(hub_dot);
        for index in order {
            root = root.child(caps[index].render(ctx));
        }
        root.into_any_element()
    }
}
