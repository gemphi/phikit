//! The one renderer for controls (`BView::Ctrl`): a widget per
//! `BCtrlKind`, every edit a `Set` of the control's key.
//!
//! A control composes the atoms that already exist — a button for a
//! checkbox or a step, a tab for a radio option, the field shell for typed
//! text, a popover for a dropdown — so a checkbox in a property panel is
//! the same checkbox as in a filter menu, and a number in the render panel
//! steps the way a frame number does. The number field is `number.rs`.

mod number;
mod picker;
mod swatch;

use self::swatch::BCtrlSwatch;
use super::flex::{BAlign, BEdge, BFill, BFlex, BFlexed, BJustify, BLen, BRadius, BSpace, BType};
use super::ctx::BRenderCtx;
use super::{walk, button, search, tab, track};
use super::track::BTrack;
use crate::v2::state::val::BValue;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl, BCtrl, BCtrlKind, BPopoverSide};
use gpui::prelude::*;
use gpui::AnyElement;

enum BCtrlPart {
    /// A row of the control's pieces: steps around a number, options of a radio.
    Row,
    /// A colour's swatch, filled with the value — in whichever spelling the
    /// control carries it (`BCtrlSwatch`).
    Swatch(BCtrlSwatch),
    /// A switch's row: the label, then the track at the end.
    SwitchRow,
    /// The switch's label, taking the slack so the track sits at the end.
    SwitchLabel,
    /// The pill track, `true` while on: accent-filled with the knob at the
    /// end, else the card tone with the knob at the start.
    Track(bool),
    /// The knob: a white disc with a hairline, so it reads on either fill.
    Knob,
}

/// The switch's track and knob, in pixels — `UISwitch`'s proportions at the
/// row's scale: a track a little wider than two knobs, the knob inset by the
/// track's padding either side.
const SWITCH_TRACK: (f32, f32) = (34.0, 20.0);
const SWITCH_KNOB: f32 = 16.0;

impl BFlexed for BCtrlPart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        match self {
            Self::Row => BFlex::row().align(BAlign::Center).gap(BSpace::GapXs).min_width(BLen::Zero),
            Self::SwitchRow => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Dim(super::flex::BDim::ButtonHeight))
                .keeps_size()
                .align(BAlign::Center)
                .gap(BSpace::GapSm)
                .min_width(BLen::Zero),
            Self::SwitchLabel => BFlex::row().fills().min_width(BLen::Zero).ink().type_size(BType::Body),
            Self::Track(on) => BFlex::row()
                .width(BLen::Measured(SWITCH_TRACK.0))
                .height(BLen::Measured(SWITCH_TRACK.1))
                .keeps_size()
                .radius(BRadius::All(SWITCH_TRACK.1 / 2.0))
                .align(BAlign::Center)
                .justify(if *on { BJustify::End } else { BJustify::Start })
                .pad_x(BSpace::GapXs)
                .fill(if *on { BFill::SwatchHsl(_c.style.colors.accent) } else { BFill::Tone(crate::v2::theme::BSurfaceTone::Card) }),
            Self::Knob => BFlex::cell()
                .width(BLen::Measured(SWITCH_KNOB))
                .height(BLen::Measured(SWITCH_KNOB))
                .keeps_size()
                .radius(BRadius::All(SWITCH_KNOB / 2.0))
                .fill(BFill::Swatch([1.0, 1.0, 1.0, 0.95]))
                .border(BEdge::All),
            Self::Swatch(value) => BFlex::cell()
                .width(BLen::Dim(super::flex::BDim::ButtonHeight))
                .height(BLen::Dim(super::flex::BDim::ButtonHeight))
                .keeps_size()
                .radius(BRadius::Role(BStyleRole::Field))
                .border(BEdge::All)
                .fill(match value {
                    BCtrlSwatch::Rgba(rgba) => BFill::Swatch(*rgba),
                    BCtrlSwatch::Hsl(color) => BFill::SwatchHsl(*color),
                }),
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
            BCtrlKind::Switch => self.switch(ctx),
            BCtrlKind::Toggle => self.flag(None, None, ctx),
            BCtrlKind::Radio => self.radio(ctx),
            BCtrlKind::Select => self.select(ctx),
            BCtrlKind::Int { .. } | BCtrlKind::Number { .. } => self.number(ctx),
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

    /// A switch (`UISwitch`): the label fills the row and the pill track sits
    /// at its end; the knob is justified to the end it is on, so no offset
    /// is computed — on, the track takes the accent, off, the card tone.
    /// Full-width by construction, which the box-button it replaces, hugging
    /// its text, never was: a column of these lines up.
    fn switch(&self, ctx: &BRenderCtx) -> AnyElement {
        let ctrl = self.0;
        let on = matches!(ctrl.value, BValue::Bool(true));
        let label = ctrl.label.clone().unwrap_or_default();
        let knob = BCtrlPart::Knob.flex(ctx).div(ctx);
        let track = BCtrlPart::Track(on).flex(ctx).div(ctx).child(knob);
        let row = BCtrlPart::SwitchRow
            .flex(ctx)
            .div(ctx)
            .child(BCtrlPart::SwitchLabel.flex(ctx).div(ctx).child(gpui::SharedString::from(label)))
            .child(track);
        let id = gpui::SharedString::from(format!("switch-{}", ctrl.key.encode()));
        let (handler, action) = (ctx.handler.clone(), ctrl.flip());
        row.id(id.clone())
            .debug_selector(move || id.to_string())
            .cursor_pointer()
            .on_click(move |_e: &gpui::ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
                (std::rc::Rc::clone(&handler))(&action, w, cx);
            })
            .into_any_element()
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
        let BCtrlKind::Slider { min, max } = ctrl.kind else { return self.number(ctx) };
        let now = match ctrl.value {
            BValue::Float(v) => v,
            BValue::Int(v) => v as f32,
            _ => min,
        };
        // The bar is the track, and `track.rs` describes what a track looks like
        // and wires the gesture — the slider is one thing that has one.
        let mut track = BTrack::across(ctrl.key.clone(), min, max);
        if matches!(ctrl.value, BValue::Int(_)) {
            track = track.whole();
        }
        let bar = track::view(&track, track::share_of(now, min, max), &ctrl.value.display(), ctx);
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
    ///
    /// The swatch takes the value in whichever spelling the control carries: a
    /// scene property hands it RGBA, a theme preference hands it the HSL text
    /// its descriptor formats. This used to match `BValue::Color` and nothing
    /// else, falling back to opaque black — and every one of the 29 colours in
    /// Preferences is the text form, so every swatch there was black.
    fn color(&self, ctx: &BRenderCtx) -> AnyElement {
        let ctrl = self.0;
        let swatch = BCtrlSwatch::of(&ctrl.value);
        let row = BCtrlPart::Row.flex(ctx).div(ctx);
        // Clicking the colour opens the channel picker. Without somewhere to
        // record open-ness the swatch is just the swatch, the way a `Select`
        // without an `open_key` is a plain button showing its choice.
        match picker::popover(ctrl, swatch, ctx.state) {
            Some(view) => row.child(walk(&view, ctx)),
            None => row.child(BCtrlPart::Swatch(swatch).flex(ctx).div(ctx)),
        }
        .child(self.text(ctx))
        .into_any_element()
    }
}

pub fn view(ctrl: &BCtrl, ctx: &BRenderCtx) -> AnyElement {
    BCtrlView(ctrl).render(ctx)
}
