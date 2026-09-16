//! PanelView — styled container for each BPanelRole.
//! Styling is self-contained: structure, areas, headers, parts.

use super::super::bview::{ActionHandler, BView};
use super::super::theme::BPalette;
use crate::store::BPanelRole;
use gpui::prelude::*;
use gpui::{AnyElement, Div, div, px};

pub struct PanelView {
    pub role: BPanelRole,
    pub children: Vec<BView>,
}

impl PanelView {
    pub fn new(role: BPanelRole, _title: String, children: Vec<BView>) -> Self {
        Self { role, children }
    }

    pub fn with_div(&self, p: &BPalette) -> Div {
        let base = div().text_color(p.text).text_xs();
        match self.role {
            // Structure
            BPanelRole::App => base
                .flex()
                .flex_row()
                .size_full()
                .items_stretch()
                .bg(p.bg_app)
                .overflow_hidden(),
            BPanelRole::CenterArea => base
                .flex()
                .flex_col()
                .flex_1()
                .items_stretch()
                .min_h_0()
                .min_w_0()
                .overflow_hidden(),
            BPanelRole::Work => base
                .flex()
                .flex_row()
                .flex_1()
                .items_stretch()
                .gap(px(p.gap_sm))
                .min_h_0()
                .min_w_0()
                .relative()
                .overflow_hidden(),
            BPanelRole::Host => base
                .flex()
                .flex_col()
                .flex_1()
                .items_stretch()
                .min_h_0()
                .min_w_0()
                .relative()
                .overflow_hidden(),

            // Bordered areas
            BPanelRole::TopArea => base
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .h(px(p.h_header))
                .flex_shrink_0()
                .bg(bg_fill(p, self.role))
                .border_1()
                .border_color(p.border)
                .rounded_tl(px(p.radius_md))
                .rounded_tr(px(p.radius_md))
                .overflow_hidden(),
            BPanelRole::BottomArea => base
                .flex()
                .flex_col()
                .h(px(p.h_bottom))
                .flex_shrink_0()
                .bg(bg_fill(p, self.role))
                .border_1()
                .border_color(p.border)
                .rounded_bl(px(p.radius_md))
                .rounded_br(px(p.radius_md))
                .min_h_0()
                .overflow_hidden(),
            BPanelRole::LeftArea => base
                .flex()
                .flex_col()
                .w(px(p.w_left))
                .flex_shrink_0()
                .bg(bg_fill(p, self.role))
                .border_1()
                .border_color(p.border)
                .rounded_tl(px(p.radius_md))
                .rounded_bl(px(p.radius_md))
                .min_h_0()
                .min_w_0()
                .overflow_hidden(),
            BPanelRole::RightArea => base
                .flex()
                .flex_col()
                .flex_1()
                .bg(bg_fill(p, self.role))
                .border_1()
                .border_color(p.border)
                .rounded_tr(px(p.radius_md))
                .rounded_br(px(p.radius_md))
                .min_h_0()
                .min_w_0()
                .overflow_hidden(),
            BPanelRole::RightColumn => base
                .flex()
                .flex_col()
                .w(px(p.w_right))
                .flex_shrink_0()
                .bg(bg_fill(p, self.role))
                .min_h_0()
                .min_w_0()
                .overflow_hidden(),

            // Bars
            BPanelRole::Header => base
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .h(px(p.h_header))
                .flex_shrink_0()
                .px(px(p.pad_sm))
                .bg(bg_fill(p, self.role))
                .overflow_hidden(),
            BPanelRole::Footer => base
                .flex()
                .flex_col()
                .flex_shrink_0()
                .h(px(p.h_footer))
                .p(px(p.pad_sm))
                .bg(bg_fill(p, self.role))
                .overflow_hidden(),
            BPanelRole::Statusbar => base
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .h(px(p.h_tab - p.pad_sm * 2.))
                .flex_shrink_0()
                .px(px(p.pad_sm))
                .gap(px(p.gap_md * 2.))
                .bg(bg_fill(p, self.role))
                .border_1()
                .border_color(p.border)
                .overflow_hidden(),

            // Parts (inside header — transparent bg, header provides bg)
            BPanelRole::Menubar
            | BPanelRole::Modebar
            | BPanelRole::Switcher
            | BPanelRole::Scrubbing => base
                .flex()
                .flex_row()
                .items_center()
                .h(px(p.h_header))
                .gap(px(p.gap_sm))
                .overflow_hidden(),
            BPanelRole::Spacebar => base
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .h(px(p.h_header))
                .gap(px(p.gap_sm))
                .flex_1()
                .overflow_hidden()
                .min_w_0(),
            // Viewport overlays — transparent floating strips on top of the 100% viewport.
            BPanelRole::Toolbar => base
                .flex()
                .flex_col()
                .gap(px(p.gap_sm))
                .p(px(p.pad_xs))
                .items_center()
                .overflow_hidden()
                .absolute()
                .top(px(p.gap_sm))
                .left_0()
                .bottom_0(),
            BPanelRole::Sidebar => base
                .flex()
                .flex_col()
                .gap(px(p.gap_sm))
                .p(px(p.pad_xs))
                .items_center()
                .overflow_hidden()
                .min_h_0()
                .absolute()
                .top(px(p.gap_sm))
                .right_0()
                .bottom_0(),
            BPanelRole::PaneLeft => base
                .flex()
                .flex_row()
                .items_center()
                .justify_start()
                .h(px(p.h_header))
                .gap(px(p.gap_sm))
                .flex_shrink_0()
                .overflow_hidden(),
            BPanelRole::PaneCenter => base
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .h(px(p.h_header))
                .gap(px(p.gap_sm))
                .flex_1()
                .overflow_hidden()
                .min_w_0(),
            BPanelRole::PaneRight => base
                .flex()
                .flex_row()
                .items_center()
                .justify_end()
                .ml_auto()
                .h(px(p.h_header))
                .gap(px(p.gap_sm))
                .flex_shrink_0()
                .overflow_hidden(),
            BPanelRole::Panel => base
                .flex()
                .flex_col()
                .flex_1()
                .gap(px(p.gap_sm))
                .overflow_hidden()
                .min_h_0()
                .min_w_0()
                .bg(bg_fill(p, self.role)),
            BPanelRole::Channels => base
                .flex()
                .flex_col()
                .w(px(p.w_channels))
                .flex_shrink_0()
                .p(px(p.pad_sm))
                .gap(px(p.gap_sm))
                .bg(bg_fill(p, self.role))
                .overflow_hidden(),
            BPanelRole::Section
            | BPanelRole::ToolProps
            | BPanelRole::AssetShelf
            | BPanelRole::HUD
            | BPanelRole::NavBar
            | BPanelRole::Execute => base.flex().flex_col().gap(px(p.gap_xs)),
            BPanelRole::Viewport => base
                .flex()
                .flex_col()
                .flex_1()
                .bg(p.fill_sheen(p.bg_viewport))
                .min_h_0()
                .min_w_0()
                .overflow_hidden(),

            BPanelRole::Modal => base
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .bg(gpui::hsla(0., 0., 0., 0.6))
                .flex()
                .items_center()
                .justify_center()
                .occlude(),
        }
    }

    pub fn into_elm(
        &self,
        p: &BPalette,
        width: Option<f32>,
        height: Option<f32>,
        on_action: ActionHandler,
    ) -> AnyElement {
        if self.role == BPanelRole::Modal {
            let backdrop = self.with_div(p);
            let mut inner = div()
                .flex()
                .flex_col()
                .w(px(640.0))
                .h(px(460.0))
                .max_w(px(800.0))
                .max_h(px(600.0))
                .bg(p.bg_panel)
                .border_1()
                .border_color(p.border)
                .rounded(px(p.radius_md))
                .shadow_xl()
                .overflow_hidden();
            let _ = width;
            let _ = height;
            for c in &self.children {
                inner = inner.child(c.into_elm(p, on_action.clone()));
            }
            return backdrop.child(inner).into_any_element();
        }

        let mut el = self.with_div(p);
        if let Some(w) = width {
            el = el.w(px(w));
        }
        if let Some(h) = height {
            el = el.h(px(h)).flex_shrink_0();
        }
        for c in &self.children {
            el = el.child(c.into_elm(p, on_action.clone()));
        }
        el.into_any_element()
    }
}

pub fn bg_fill(p: &BPalette, role: BPanelRole) -> gpui::Fill {
    match role {
        BPanelRole::RightArea
        | BPanelRole::RightColumn
        | BPanelRole::Panel
        | BPanelRole::Channels => p.fill_sheen(p.bg_panel).into(),
        BPanelRole::Header | BPanelRole::Footer | BPanelRole::Statusbar => {
            p.fill_sheen(p.bg_header).into()
        }
        BPanelRole::TopArea => p.fill_sheen(p.bg_top).into(),
        BPanelRole::BottomArea => p.fill_sheen(p.bg_toolbar).into(),
        BPanelRole::LeftArea => p.fill_sheen(p.bg_panel).into(),
        BPanelRole::Viewport => p.fill_sheen(p.bg_viewport).into(),
        BPanelRole::App => p.fill_solid(p.bg_app).into(),
        BPanelRole::Modal => p.fill_sheen(p.bg_panel).into(),
        _ => p.fill_solid(p.bg_panel).into(),
    }
}
