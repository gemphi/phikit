//! IconListView / IconButtonView — vertical icon list and individual icon buttons.
//! Icon buttons carry their own backgrounds (theme-tied), containers are transparent.
//! Tooltips show on hover via GPUI's .tooltip() API.

use super::super::bview::{ActionHandler, BAction, BView};
use super::super::icon::{IconSize, IconView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, Div, InteractiveElement, SharedString, StatefulInteractiveElement,
    Window, div, px,
};
use ui::Tooltip;

pub struct IconListView {
    pub children: Vec<BView>,
}

impl IconListView {
    pub fn new(children: Vec<BView>) -> Self {
        Self { children }
    }

    pub fn with_div(&self, _p: &BPalette) -> Div {
        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(_p.gap_sm))
            .w_full()
    }

    pub fn into_elm(&self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        let mut el = self.with_div(p);
        for c in &self.children {
            el = el.child(c.into_elm(p, on_action.clone()));
        }
        el.into_any_element()
    }
}

#[derive(Clone)]
pub struct IconButtonView {
    pub icon: String,
    pub label: Option<String>,
    pub tooltip: Option<String>,
    pub active: bool,
}

impl IconButtonView {
    pub fn new(icon: String, label: Option<String>, tooltip: Option<String>) -> Self {
        Self {
            icon,
            label,
            tooltip,
            active: false,
        }
    }

    pub fn with_elm(&self, p: &BPalette) -> Div {
        let icon_color = if self.active { p.accent } else { p.text };
        let mut el = div()
            .flex()
            .items_center()
            .cursor_pointer()
            .rounded(px(p.radius_sm))
            .border_1()
            .border_color(if self.active { p.accent } else { p.border });

        if self.active {
            el = el.bg(p.fill_sheen(p.bg_icon_active));
        } else {
            el = el
                .bg(p.fill_sheen(p.bg_icon))
                .hover(|s| s.bg(p.fill_sheen(p.bg_icon_hover)).border_color(p.accent));
        }

        if let Some(label) = &self.label {
            el = el
                .flex_row()
                .gap(px(p.gap_sm))
                .w_full()
                .px(px(p.pad_sm))
                .h(px(p.h_tab))
                .child(
                    IconView::new(&self.icon)
                        .size(IconSize::Md)
                        .into_element(icon_color),
                )
                .child(
                    div()
                        .text_color(icon_color)
                        .child(SharedString::from(label.clone())),
                );
        } else {
            el = el.justify_center().w(px(p.h_tab)).h(px(p.h_tab)).child(
                IconView::new(&self.icon)
                    .size(IconSize::Md)
                    .into_element(icon_color),
            );
        }
        el
    }

    pub fn into_elm(
        &self,
        p: &BPalette,
        active: bool,
        action: Option<&BAction>,
        on_action: ActionHandler,
    ) -> AnyElement {
        let view = IconButtonView {
            active,
            ..self.clone()
        };
        let el = view.with_elm(p);
        let needs_id = action.is_some() || self.tooltip.is_some();
        if !needs_id {
            return el.into_any_element();
        }
        let id = match &self.label {
            Some(l) => format!("{}-{}", self.icon, l),
            None => self.icon.clone(),
        };
        let mut el = el.id(id);

        if let Some(tip) = &self.tooltip {
            let tip = tip.clone();
            el = el.tooltip(move |_, cx| Tooltip::simple(tip.clone(), cx));
        }

        if let Some(act) = action {
            let act = act.clone();
            let handler = on_action;
            el = el.on_click(move |_e: &ClickEvent, w: &mut Window, cx: &mut App| {
                handler(&act, w, cx);
            });
        }
        el.into_any_element()
    }

    /// Popup item version — plain, no bg by default, lighter on hover only.
    pub fn into_popup_item(
        &self,
        p: &BPalette,
        active: bool,
        action: Option<&BAction>,
        on_action: ActionHandler,
    ) -> AnyElement {
        let icon_color = if active { p.accent } else { p.text };
        let mut el = div()
            .flex()
            .flex_row()
            .items_center()
            .cursor_pointer()
            .gap(px(p.gap_sm))
            .px(px(p.pad_sm))
            .py(px(p.pad_xs))
            .h(px(p.h_tab))
            .text_color(icon_color)
            .text_xs()
            .rounded(px(p.radius_sm))
            .hover(|s| s.bg(p.fill_solid(p.bg_hover)));

        el = el.child(
            IconView::new(&self.icon)
                .size(IconSize::Sm)
                .into_element(icon_color),
        );
        if let Some(label) = &self.label {
            el = el.child(SharedString::from(label.clone()));
        }

        let needs_id = action.is_some();
        if !needs_id {
            return el.into_any_element();
        }
        let id = match &self.label {
            Some(l) => format!("popup-{}-{}", self.icon, l),
            None => format!("popup-{}", self.icon),
        };
        let mut el = el.id(id);
        if let Some(act) = action {
            let act = act.clone();
            let handler = on_action;
            el = el.on_click(move |_e: &ClickEvent, w: &mut Window, cx: &mut App| {
                handler(&act, w, cx);
            });
        }
        el.into_any_element()
    }
}
