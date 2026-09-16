//! Minimal Blender-style node graph used by Shader/Node-family hosts.

use super::super::super::{color::to_hsla, ctx::BRenderCtx};
use crate::v2::host::host_slug;
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
        c: &BRenderCtx,
    ) -> AnyElement {
        let mut card = div()
            .flex()
            .flex_col()
            .w(px(width))
            .rounded(px(c.style.radius_for(BStyleRole::Card)))
            .bg(to_hsla(c.style.colors.bg_panel))
            .border(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.border))
            .overflow_hidden()
            .child(
                div()
                    .h(px(c.style.dimensions.header_height))
                    .px(px(c.style.spacing.pad_sm))
                    .flex()
                    .items_center()
                    .bg(to_hsla(header))
                    .text_color(to_hsla(c.style.colors.text))
                    .text_size(px(c.style.typography.header))
                    .child(SharedString::from(title)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(c.style.spacing.gap_sm))
                    .p(px(c.style.spacing.pad_sm))
                    .text_color(to_hsla(c.style.colors.text_dim))
                    .text_size(px(c.style.typography.caption))
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
                .border(px(c.style.strokes.border * 2.0))
                .border_color(to_hsla(c.style.colors.accent));
        }
        let handler = c.h.clone();
        let action = BAction::Set(
            BPath::owned(vec![host_slug(c.host), "selected"]),
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

    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let selected = c.s.host(c.host).get("selected").unwrap_or("principled");
        let grid = c.style.node_grid_stroke();
        let bg = to_hsla(c.style.viewport_surface());
        let mut canvas = div()
            .flex_1()
            .min_h_0()
            .size_full()
            .relative()
            .overflow_hidden()
            .bg(bg);
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
        let primary_width = c.style.dimensions.channels_width + c.style.spacing.gap_lg;
        let output_width =
            c.style.dimensions.sidebar_min_width - c.style.spacing.gap_xs - c.style.strokes.border;
        canvas
            .child(
                div()
                    .absolute()
                    .left(gpui::relative(0.43))
                    .top(gpui::relative(0.47))
                    .w(gpui::relative(0.22))
                    .h(px(c.style.strokes.grid * 3.0))
                    .rounded_full()
                    .bg(to_hsla(c.style.colors.online)),
            )
            .child(self.card(
                "principled",
                "Principled BSDF",
                "Base Color   Roughness  0.500",
                0.12,
                0.18,
                primary_width,
                c.style.colors.online,
                selected == "principled",
                c,
            ))
            .child(self.card(
                "material_output",
                "Material Output",
                "Target: All",
                0.65,
                0.31,
                output_width,
                c.style.colors.accent,
                selected == "material_output",
                c,
            ))
            .into_any_element()
    }
}

use crate::v2::view::render::gpui::prim::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BNode {
    fn render(&self, c: &crate::v2::view::render::gpui::ctx::BRenderCtx) -> gpui::AnyElement {
        BNode::render(self, c)
    }
}
