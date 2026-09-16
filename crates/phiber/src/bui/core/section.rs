//! SectionView — collapsible section with header (arrow + icon + label) and body.
//! Header has bg color (tab colors: bg_active when expanded, bg_toolbar when collapsed).
//! Content has a light shade (slightly lighter than header).
//! Sections are rounded with gaps between them (applied by parent ScrollView).

use super::super::bview::{ActionHandler, BAction, BView};
use super::super::icon::{IconSize, IconView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, Div, InteractiveElement, SharedString, StatefulInteractiveElement,
    Window, div, px,
};

pub struct SectionView {
    pub label: String,
    pub icon: Option<String>,
    pub expanded: bool,
    pub children: Vec<BView>,
}

impl SectionView {
    pub fn new(label: String, icon: Option<String>, expanded: bool, children: Vec<BView>) -> Self {
        Self {
            label,
            icon,
            expanded,
            children,
        }
    }

    pub fn with_elm(&self, p: &BPalette) -> Div {
        // Outer wrapper: rounded, overflow hidden. No bg — heading has its own, content is transparent (area bg shows through).
        let mut el = div()
            .flex()
            .flex_col()
            .w_full()
            .text_color(p.text)
            .text_xs()
            .rounded(px(p.radius_sm))
            .overflow_hidden();

        // Header: same gradient shade as the header/viewport.
        let header_bg = p.bg_viewport;
        let mut header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(p.gap_sm))
            .px(px(p.pad_sm))
            .py(px(p.pad_xs))
            .w_full()
            .h(px(p.h_tab))
            .flex_shrink_0()
            .bg(p.fill_sheen(header_bg))
            .cursor_pointer()
            .hover(|s| s.bg(p.fill_sheen(p.bg_hover)));

        let arrow_icon = if self.expanded {
            "chevron_down"
        } else {
            "chevron_right"
        };
        let mut left = div().flex().flex_row().items_center().gap(px(p.gap_sm));
        if let Some(icon) = &self.icon {
            if !icon.is_empty() {
                left = left.child(
                    IconView::new(icon)
                        .size(IconSize::Md)
                        .into_element(p.text_dim),
                );
            }
        }
        left = left.child(SharedString::from(self.label.clone()));
        header = header.child(left);
        header = header.child(
            IconView::new(arrow_icon)
                .size(IconSize::Md)
                .into_element(p.text_dim),
        );
        el = el.child(header);

        if self.expanded {
            // Body: slightly lighter than header for contrast.
            let body = div()
                .flex()
                .flex_col()
                .px(px(p.pad_sm))
                .py(px(p.pad_xs))
                .gap(px(p.gap_xs))
                .w_full()
                .bg(p.fill_sheen(p.bg_panel));
            el = el.child(body);
        }
        el
    }

    pub fn body_div(&self, p: &BPalette) -> Div {
        div()
            .flex()
            .flex_col()
            .px(px(p.pad_sm))
            .py(px(p.pad_xs))
            .gap(px(p.gap_xs))
            .w_full()
            .bg(p.fill_sheen(p.bg_panel))
    }

    pub fn into_elm(
        &self,
        p: &BPalette,
        action: Option<&BAction>,
        on_action: ActionHandler,
    ) -> AnyElement {
        let sv = SectionView::new(self.label.clone(), self.icon.clone(), self.expanded, vec![]);
        let mut el = sv.with_elm(p);
        if self.expanded {
            let mut body = sv.body_div(p);
            for c in &self.children {
                body = body.child(c.into_elm(p, on_action.clone()));
            }
            el = el.child(body);
        }
        if action.is_none() {
            return el.into_any_element();
        }
        let act = action.unwrap().clone();
        let mut el = el.id(self.label.clone());
        let handler = on_action.clone();
        el = el.on_click(move |_e: &ClickEvent, w: &mut Window, cx: &mut App| {
            handler(&act, w, cx);
        });
        el.into_any_element()
    }
}
