//! Minimal Blender-style node graph used by Shader/Node-family hosts.

use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use crate::v2::state::BPath;
use crate::v2::theme::{BStyleRole, palette::BColor};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px};
use std::rc::Rc;

pub struct BNode;

impl BNode {
    #[allow(clippy::too_many_arguments)]
    fn card(
        &self,
        id: &'static str,
        title: &'static str,
        subtitle: &'static str,
        x: f32,
        y: f32,
        width: f32,
        header: BColor,
        selected: bool,
        ctx: &BRenderCtx,
    ) -> AnyElement {
        let mut card = div()
            .flex()
            .flex_col()
            .w(px(width))
            .rounded(px(ctx.style.radius_for(BStyleRole::Card)))
            .bg(to_hsla(ctx.style.colors.bg_panel))
            .border(px(ctx.style.strokes.border))
            .border_color(to_hsla(ctx.style.colors.border))
            .overflow_hidden()
            .child(
                div()
                    .h(px(ctx.style.dimensions.header_height))
                    .px(px(ctx.style.spacing.pad_sm))
                    .flex()
                    .items_center()
                    .bg(to_hsla(header))
                    .text_color(to_hsla(ctx.style.colors.text))
                    .text_size(px(ctx.style.typography.header))
                    .child(SharedString::from(title)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(ctx.style.spacing.gap_sm))
                    .p(px(ctx.style.spacing.pad_sm))
                    .text_color(to_hsla(ctx.style.colors.text_dim))
                    .text_size(px(ctx.style.typography.caption))
                    .child(SharedString::from(subtitle))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(SharedString::from("Surface"))
                            .child(SharedString::from("●")),
                    ),
            );
        if selected {
            card = card
                .border(px(ctx.style.strokes.border * 2.0))
                .border_color(to_hsla(ctx.style.colors.accent));
        }
        let handler = ctx.handler.clone();
        let action = BAction::Set(
            BPath::owned(vec![ctx.host.slug(), "selected"]),
            id.to_string(),
        );
        div()
            .id(SharedString::from(format!("node-{id}")))
            .absolute()
            .left(gpui::relative(x))
            .top(gpui::relative(y))
            .cursor_pointer()
            .child(card)
            .on_click(move |_e: &ClickEvent, w, cx| {
                (Rc::clone(&handler))(&action, w, cx);
            })
            .into_any_element()
    }

    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let selected = ctx.state.host(ctx.host).get("selected").unwrap_or("principled");
        let grid = ctx.style.node_grid_stroke();
        // The canvas, with the theme's sweep when it has one.
        let bg = super::super::color::resolved_gradient(ctx.style.viewport_gradient());
        let mut canvas = super::base(ctx).bg(bg);
        // Fractions are canvas coordinates, not design-system spacing.
        for fraction in [0.125_f32, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875] {
            canvas = canvas
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .h_full()
                        .w(px(grid.width))
                        .left(gpui::relative(fraction))
                        .bg(to_hsla(grid.color)),
                )
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .w_full()
                        .h(px(grid.width))
                        .top(gpui::relative(fraction))
                        .bg(to_hsla(grid.color)),
                );
        }
        let primary_width = ctx.style.dimensions.channels_width + ctx.style.spacing.gap_lg;
        let output_width =
            ctx.style.dimensions.sidebar_min_width - ctx.style.spacing.gap_xs - ctx.style.strokes.border;
        canvas
            .child(
                div()
                    .absolute()
                    .left(gpui::relative(0.43))
                    .top(gpui::relative(0.47))
                    .w(gpui::relative(0.22))
                    .h(px(ctx.style.strokes.grid * 3.0))
                    .rounded_full()
                    .bg(to_hsla(ctx.style.colors.online)),
            )
            .child(self.card(
                "principled",
                "Principled BSDF",
                "Base Color   Roughness  0.500",
                0.12,
                0.18,
                primary_width,
                ctx.style.colors.online,
                selected == "principled",
                ctx,
            ))
            .child(self.card(
                "material_output",
                "Material Output",
                "Target: All",
                0.65,
                0.31,
                output_width,
                ctx.style.colors.accent,
                selected == "material_output",
                ctx,
            ))
            .into_any_element()
    }
}
