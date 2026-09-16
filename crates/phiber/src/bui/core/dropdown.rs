//! DropdownView — dropdown menu with popup aligned at bottom of trigger.
//! Trigger button uses menu button colors (no bg, hover only).
//! Popup items are plain (no bg) with lighter color only on hover.
//! Submenu children open on hover via GPUI group_hover or state toggle.
//! When align_right is true, popup is anchored to the right edge and opens towards the left.

use super::super::bview::{ActionHandler, BAction, BView};
use super::super::icon::{IconSize, IconView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, Div, InteractiveElement, IntoElement, SharedString,
    StatefulInteractiveElement, Window, deferred, div, px,
};

pub struct DropdownView {
    pub label: String,
    pub icon: Option<String>,
    pub icon_only: bool,
    pub align_right: bool,
    pub children: Vec<BView>,
    pub hover_action: Option<BAction>,
}

impl DropdownView {
    pub fn new(
        label: String,
        icon: Option<String>,
        icon_only: bool,
        align_right: bool,
        children: Vec<BView>,
    ) -> Self {
        Self {
            label,
            icon,
            icon_only,
            align_right,
            children,
            hover_action: None,
        }
    }

    /// Trigger button — uses section color (bg_toolbar), hover shows lighter color.
    pub fn with_elm(&self, p: &BPalette) -> Div {
        let base_color = if self.icon_only { p.text_dim } else { p.text };
        let hover_bg = p.fill_sheen(p.bg_hover);
        let hover_color = if self.icon_only { p.text } else { p.text };
        let mut el = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(p.gap_sm))
            .px(px(p.pad_sm))
            .py(px(p.pad_xs))
            .h(px(p.h_tab))
            .text_color(base_color)
            .text_sm()
            .cursor_pointer()
            .rounded_tl(px(p.radius_sm))
            .rounded_tr(px(p.radius_sm))
            .hover(move |s| s.bg(hover_bg).text_color(hover_color));
        let mut left = div().flex().flex_row().items_center().gap(px(p.gap_sm));
        if let Some(icon) = &self.icon {
            if !icon.is_empty() {
                left = left.child(IconView::new(icon).size(IconSize::Sm).into_element(p.text));
            }
        }
        left = left.child(SharedString::from(self.label.clone()));
        el = el.child(left).child(
            IconView::new("chevron_down")
                .size(IconSize::Md)
                .into_element(p.text_dim),
        );
        el
    }

    pub fn into_elm(
        &self,
        p: &BPalette,
        open: bool,
        action: Option<&BAction>,
        on_action: ActionHandler,
    ) -> AnyElement {
        let el = self.with_elm(p);
        let id = match action {
            Some(BAction::Toggle(key)) => key.key(),
            _ => format!("dropdown-{}", self.label),
        };
        let mut el = el.id(id.clone()).relative();

        // When open, show hover bg persistently.
        if open {
            el = el.bg(p.fill_sheen(p.bg_active));
        }

        // Click toggles the dropdown.
        if let Some(act) = action {
            let act = act.clone();
            let handler = on_action.clone();
            el = el.on_click(move |_e: &ClickEvent, w: &mut Window, cx: &mut App| {
                handler(&act, w, cx);
            });
        }

        // Hover-to-switch: when hover_action is set (menubar mode), hovering
        // the trigger opens this dropdown and closes others. Only fires when
        // the dropdown is not already open.
        if let Some(hact) = &self.hover_action {
            if !open {
                let hact = hact.clone();
                let handler = on_action.clone();
                el = el.on_hover(move |hovered: &bool, w: &mut Window, cx: &mut App| {
                    if *hovered {
                        handler(&hact, w, cx);
                    }
                });
            }
        }

        if open && !self.children.is_empty() {
            let mut popup = div();
            if self.align_right {
                popup = popup.absolute().top_full().right_0().mt(px(p.gap_xs));
            } else {
                popup = popup.absolute().top_full().left_0().mt(px(p.gap_xs));
            }

            popup = popup
                .flex()
                .flex_col()
                .py(px(p.pad_xs))
                .gap(px(p.gap_xs))
                .min_w(px(p.w_popup))
                .bg(p.fill_sheen(p.bg_viewport))
                .border_1()
                .border_color(p.border)
                .rounded(px(p.radius_md))
                .px(px(p.pad_xs))
                .occlude()
                .shadow_xl();

            for c in &self.children {
                popup = popup.child(c.into_popup_item(p, on_action.clone()));
            }

            // Click-outside backdrop covering the entire window.
            let close_action = action.cloned();
            let close_handler = on_action;
            let backdrop = div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .id(format!("{}-backdrop", id))
                .on_click(move |_e: &ClickEvent, w: &mut Window, cx: &mut App| {
                    if let Some(act) = &close_action {
                        close_handler(act, w, cx);
                    }
                });

            div()
                .relative()
                .child(el)
                .child(deferred(backdrop))
                .child(deferred(popup))
                .into_any_element()
        } else {
            el.into_any_element()
        }
    }

    /// Popup item (inside a dropdown menu) — plain, no bg, lighter color on hover only.
    /// Submenu children open on hover via group_hover or state toggle.
    pub fn into_popup_item(
        &self,
        p: &BPalette,
        open: bool,
        action: Option<&BAction>,
        on_action: ActionHandler,
    ) -> AnyElement {
        let has_children = !self.children.is_empty();
        let group_id = SharedString::from(format!("submenu-grp-{}", self.label));

        // Plain item: no bg by default, only hover shows lighter color.
        let mut el = div()
            .group(group_id.clone())
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(p.gap_sm))
            .px(px(p.pad_sm))
            .py(px(p.pad_xs))
            .h(px(p.h_tab))
            .text_color(p.text)
            .text_xs()
            .cursor_pointer()
            .rounded(px(p.radius_sm))
            .hover(|s| s.bg(p.fill_solid(p.bg_hover)))
            .relative()
            .id(format!("popup-item-{}", self.label));

        let mut left = div().flex().flex_row().items_center().gap(px(p.gap_sm));
        if let Some(ic) = &self.icon {
            if !ic.is_empty() {
                left = left.child(IconView::new(ic).size(IconSize::Sm).into_element(p.text));
            }
        }
        left = left.child(SharedString::from(self.label.clone()));
        el = el.child(left);

        if has_children {
            el = el.child(
                IconView::new("chevron_right")
                    .size(IconSize::Md)
                    .into_element(p.text_dim),
            );

            // Build submenu popup — visible on hover via group_hover or if open in state.
            let mut sub_popup = div()
                .absolute()
                .left_full()
                .top_0()
                .ml(px(-2.0))
                .flex()
                .flex_col()
                .py(px(p.pad_xs))
                .gap(px(p.gap_xs))
                .min_w(px(p.w_popup))
                .bg(p.fill_sheen(p.bg_viewport))
                .border_1()
                .border_color(p.border)
                .rounded(px(p.radius_md))
                .px(px(p.pad_xs))
                .occlude()
                .shadow_xl();

            if !open {
                sub_popup = sub_popup.invisible().group_hover(group_id, |s| s.visible());
            }

            for c in &self.children {
                sub_popup = sub_popup.child(c.into_popup_item(p, on_action.clone()));
            }

            if let Some(act) = action {
                let act = act.clone();
                let handler = on_action;
                el = el.on_click(move |_e: &ClickEvent, w: &mut Window, cx: &mut App| {
                    handler(&act, w, cx);
                });
            }

            el.child(sub_popup).into_any_element()
        } else {
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
}
