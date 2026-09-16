//! TabView — single tab item with label, optional icon, and active state.

use super::super::bview::{ActionHandler, BAction};
use super::super::icon::{IconSize, IconView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, Div, InteractiveElement, SharedString, StatefulInteractiveElement,
    Window, div, px,
};

pub struct TabView {
    pub label: String,
    pub icon: Option<String>,
    pub active: bool,
    pub icon_only: bool,
}

impl TabView {
    pub fn new(label: String, icon: Option<String>, active: bool, icon_only: bool) -> Self {
        Self {
            label,
            icon,
            active,
            icon_only,
        }
    }

    pub fn with_elm(&self, p: &BPalette) -> Div {
        let col = if self.active { p.accent } else { p.text };
        let mut el = div().flex().items_center().cursor_pointer();

        if self.icon_only {
            // Vertical tabs in Properties window — exactly matches sidebar/toolbar icon buttons
            el = el
                .justify_center()
                .w(px(p.h_tab))
                .h(px(p.h_tab))
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
        } else {
            // Horizontal tabs (spacebar)
            el = el
                .flex_row()
                .gap(px(p.gap_sm))
                .px(px(p.pad_sm))
                .py(px(p.pad_xs))
                .h(px(p.h_tab))
                .text_xs()
                .rounded_tl(px(p.radius_sm))
                .rounded_tr(px(p.radius_sm))
                .border_1()
                .border_color(if self.active { p.accent } else { p.border });

            if self.active {
                el = el
                    .bg(p.fill_sheen(p.bg_active))
                    .border_t_1()
                    .border_color(p.accent);
            } else {
                el = el
                    .bg(p.fill_sheen(p.bg_icon))
                    .hover(|s| s.bg(p.fill_sheen(p.bg_icon_hover)).text_color(p.accent));
            }
        }

        if let Some(icon) = &self.icon {
            if !icon.is_empty() {
                el = el.child(IconView::new(icon).size(IconSize::Md).into_element(col));
            }
        }
        if !self.icon_only {
            el = el.child(SharedString::from(self.label.clone()));
        }
        el
    }

    pub fn into_elm(
        &self,
        p: &BPalette,
        action: Option<&BAction>,
        on_action: ActionHandler,
    ) -> AnyElement {
        let el = self.with_elm(p);
        if action.is_none() {
            return el.into_any_element();
        }
        let act = action.unwrap().clone();
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        act.hash(&mut hasher);
        let id = format!("tab-{:x}-{}", hasher.finish(), self.label);
        let mut el = el.id(id);
        let handler = on_action.clone();
        el = el.on_click(move |_e: &ClickEvent, w: &mut Window, cx: &mut App| {
            handler(&act, w, cx);
        });
        el.into_any_element()
    }
}
