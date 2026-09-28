//! The one renderer for controls (`BView::Ctrl`): a widget per
//! `BCtrlKind`, every edit a `Set` of the control's key.
//!
//! A control composes the atoms that already exist — a button for a
//! checkbox or a step, a tab for a radio option, the field shell for typed
//! text, a popover for a dropdown — so a checkbox in a property panel is
//! the same checkbox as in a filter menu, and a number in the render panel
//! steps the way a frame number does. The number field is `number.rs`.

mod number;

use self::number::number;
use super::flex::{BAlign, BEdge, BFill, BFlex, BFlexed, BJustify, BLen, BRadius, BSpace};
use super::ctx::BRenderCtx;
use super::{walk, button, search, tab};
use crate::v2::state::val::BValue;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl, BCtrl, BCtrlKind, BPopoverSide};
use gpui::prelude::*;
use gpui::AnyElement;

enum BCtrlPart {
    /// A row of the control's pieces: steps around a number, options of a radio.
    Row,
    /// The number's read-out between its steps, and the bar of a slider.
    Value,
    /// A slider's filled part, `share` of the bar.
    Bar { share: f32 },
    /// A colour's swatch, filled with the value.
    Swatch([f32; 4]),
}

impl BFlexed for BCtrlPart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        match self {
            Self::Row => BFlex::row().align(BAlign::Center).gap(BSpace::GapXs).min_width(BLen::Zero),
            Self::Value => BFlex::row()
                .fills()
                .min_width(BLen::Zero)
                .align(BAlign::Center)
                .justify(BJustify::Center)
                .height(BLen::Dim(super::flex::BDim::ButtonHeight))
                .radius(BRadius::Role(BStyleRole::Field))
                .border(BEdge::All)
                .overflow(super::flex::BOverflow::Clip),
            Self::Bar { share } => BFlex::cell()
                .width(BLen::Share(*share))
                .height(BLen::Full)
                .fill(BFill::Tone(crate::v2::theme::BSurfaceTone::Header)),
            Self::Swatch(rgba) => BFlex::cell()
                .width(BLen::Dim(super::flex::BDim::ButtonHeight))
                .height(BLen::Dim(super::flex::BDim::ButtonHeight))
                .keeps_size()
                .radius(BRadius::Role(BStyleRole::Field))
                .border(BEdge::All)
                .fill(BFill::Swatch(*rgba)),
        }
    }
}

/// One control, drawn.
///
/// A control composes atoms that already exist — a button for a checkbox or
/// a step, a tab for a radio option, the field shell for typed text, a
/// popover for a dropdown — so a checkbox in a property panel is the same
/// checkbox as in a filter menu. Each kind is a method of this type rather
/// than a free function beside it, so the widget a kind draws as is reached
/// through the control, not from anywhere.
pub struct BCtrlView<'a>(pub &'a BCtrl);

impl BCtrlView<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        match self.0.kind {
            BCtrlKind::Check => self.flag(Some("check_square"), Some("square"), ctx),
            BCtrlKind::Toggle => self.flag(None, None, ctx),
            BCtrlKind::Radio => self.radio(ctx),
            BCtrlKind::Select => self.select(ctx),
            BCtrlKind::Int { .. } | BCtrlKind::Number { .. } => number(self.0, ctx),
            BCtrlKind::Slider { .. } => self.slider(ctx),
            BCtrlKind::Text | BCtrlKind::Search | BCtrlKind::Path => self.text(ctx),
            BCtrlKind::Color => self.color(ctx),
        }
    }

    /// A boolean: one control that answers to its own state, drawn as a
    /// button carrying a box glyph (a checkbox) or as a tab filled while on
    /// (a toggle). The two differed only in the glyph and the atom, so they
    /// are one method rather than two that drift apart.
    fn flag(&self, on_icon: Option<&str>, off_icon: Option<&str>, ctx: &BRenderCtx) -> AnyElement {
        let ctrl = self.0;
        let on = matches!(ctrl.value, BValue::Bool(true));
        let mut ctl = BControl::new().with_active(on).with_action(ctrl.flip());
        if let Some(label) = &ctrl.label {
            ctl = ctl.with_label(label.clone());
        }
        match if on { on_icon } else { off_icon } {
            Some(icon) => button::view(&ctl.with_icon(icon), ctx),
            // No glyph: the fill is the state, which is what a tab says.
            None => tab::view(&ctl, ctx),
        }
    }

    /// One option per tab, the chosen one filled.
    fn radio(&self, ctx: &BRenderCtx) -> AnyElement {
        let ctrl = self.0;
        let chosen = ctrl.chosen().map(|o| o.key.clone()).unwrap_or_default();
        let mut row = BCtrlPart::Row.flex(ctx).div(ctx);
        for option in &ctrl.options {
            let mut ctl = BControl::new()
                .with_label(option.label.clone())
                .with_active(option.key == chosen)
                .with_action(ctrl.set(BValue::Enum(option.key.clone())));
            if !option.icon.is_empty() {
                ctl = ctl.with_icon(option.icon.clone());
            }
            row = row.child(tab::view(&ctl, ctx));
        }
        row.into_any_element()
    }

    /// A dropdown: the chosen option on a cap, its options in a popover. With
    /// nowhere to record open-ness it is a plain button showing the choice.
    fn select(&self, ctx: &BRenderCtx) -> AnyElement {
        let ctrl = self.0;
        let chosen = ctrl.chosen();
        let shown = chosen
            .map(|o| o.label.clone())
            .or_else(|| ctrl.label.clone())
            .unwrap_or_else(|| "—".to_string());
        let Some(open_key) = ctrl.open_key.clone() else {
            return button::view(&BControl::new().with_label(shown), ctx);
        };
        let anchor = BView::MenuAnchor(
            BControl::new()
                .with_icon("chevron_down")
                .with_label(shown)
                .with_active(ctrl.open)
                .with_action(BAction::Toggle(open_key.clone())),
        );
        let body = ctrl
            .options
            .iter()
            .map(|o| {
                let on = chosen.is_some_and(|ch| ch.key == o.key);
                BView::MenuItem(
                    BControl::new()
                        .with_icon(if on { "check" } else if o.icon.is_empty() { "" } else { o.icon.as_str() })
                        .with_label(o.label.clone())
                        .with_active(on)
                        .with_action(ctrl.set(BValue::Enum(o.key.clone()))),
                )
            })
            .collect();
        walk(&BView::popover(anchor, body, ctrl.open, BAction::Reset(open_key), BPopoverSide::Below), ctx)
    }

    /// A bar filled to the value, a step either side of it.
    fn slider(&self, ctx: &BRenderCtx) -> AnyElement {
        let ctrl = self.0;
        let BCtrlKind::Slider { min, max } = ctrl.kind else { return number(ctrl, ctx) };
        let now = match ctrl.value {
            BValue::Float(v) => v,
            BValue::Int(v) => v as f32,
            _ => min,
        };
        let share = if max > min { ((now - min) / (max - min)).clamp(0.0, 1.0) } else { 0.0 };
        let bar = BCtrlPart::Value
            .flex(ctx)
            .div(ctx)
            .child(BCtrlPart::Bar { share }.flex(ctx).div(ctx))
            .child(gpui::SharedString::from(ctrl.value.display()));
        let step = |icon: &str, steps: f32| {
            let mut ctl = BControl::new().with_icon(icon);
            if let Some(action) = ctrl.nudge(steps) {
                ctl = ctl.with_action(action);
            }
            button::view(&ctl, ctx)
        };
        BCtrlPart::Row
            .flex(ctx)
            .div(ctx)
            .child(step("chevron_left", -1.0))
            .child(bar)
            .child(step("chevron_right", 1.0))
            .into_any_element()
    }

    /// Typed text: the field shell, with the label as its placeholder.
    fn text(&self, ctx: &BRenderCtx) -> AnyElement {
        let ctrl = self.0;
        let placeholder = ctrl.label.clone().unwrap_or_default();
        let shown = ctrl.value.display();
        search::view(&placeholder, &shown, ctrl.editing, &ctrl.set(ctrl.value.clone()), ctx)
    }

    /// A swatch of the value, and the value as text beside it.
    fn color(&self, ctx: &BRenderCtx) -> AnyElement {
        let rgba = match self.0.value {
            BValue::Color(rgba) => rgba,
            _ => [0.0, 0.0, 0.0, 1.0],
        };
        BCtrlPart::Row
            .flex(ctx)
            .div(ctx)
            .child(BCtrlPart::Swatch(rgba).flex(ctx).div(ctx))
            .child(self.text(ctx))
            .into_any_element()
    }
}

pub fn view(ctrl: &BCtrl, ctx: &BRenderCtx) -> AnyElement {
    BCtrlView(ctrl).render(ctx)
}
