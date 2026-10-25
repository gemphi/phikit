//! The one renderer for controls (`BView::Ctrl`): a widget per
//! `BCtrlKind`, every edit a `Set` of the control's key.
//!
//! A control composes the atoms that already exist — a button for a
//! checkbox or a step, a tab for a radio option, the field shell for typed
//! text, a popover for a dropdown — so a checkbox in a property panel is
//! the same checkbox as in a filter menu, and a number in the render panel
//! steps the way a frame number does. The number field is `number.rs`.

mod complex;
mod number;
mod picker;
mod swatch;

use self::swatch::BCtrlSwatch;
use super::flex::{BAlign, BDim, BEdge, BFill, BFlex, BFlexed, BJustify, BLen, BRadius, BSpace, BType};
use super::ctx::BRenderCtx;
use super::{walk, button, search, track};
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
    /// A checkbox's row: the box, then its label, a control's height.
    CheckRow,
    /// The box's cell: a tile's square, so the box sits where an icon
    /// button's glyph would.
    CheckBox,
}

/// The switch's track and knob, in pixels — `UISwitch`'s proportions at the
/// row's scale: a track a little wider than two knobs, the knob inset by the
/// track's padding either side.
const SWITCH_TRACK: (f32, f32) = (34.0, 20.0);
const SWITCH_KNOB: f32 = 16.0;

/// An off track: `ink` at the strength that reads on any surface.
const fn switch_off(ink: crate::v2::theme::BColor) -> crate::v2::theme::BColor {
    (ink.0, ink.1, ink.2, 0.35)
}

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
            Self::CheckRow => BFlex::row().align(BAlign::Center).self_align(super::flex::BSelf::Center).gap(BSpace::GapSm).height(BLen::Dim(BDim::ButtonHeight)),
            Self::CheckBox => BFlex::cell().width(BLen::Dim(BDim::Tile)).height(BLen::Dim(BDim::Tile)).keeps_size().align(BAlign::Center).justify(BJustify::Center),
            Self::Track(on) => BFlex::row()
                .width(BLen::Measured(SWITCH_TRACK.0))
                .height(BLen::Measured(SWITCH_TRACK.1))
                .keeps_size()
                .radius(BRadius::All(SWITCH_TRACK.1 / 2.0))
                .align(BAlign::Center)
                .justify(if *on { BJustify::End } else { BJustify::Start })
                .pad_x(BSpace::GapXs)
                // Off, the track is still drawn — the dimmed ink at a third —
                // so a switch keeps its shape either way and only the knob
                // moves; on the card tone it vanished and the knob sat alone.
                .fill(if *on { BFill::SwatchHsl(_c.style.colors.accent) } else { BFill::SwatchHsl(switch_off(_c.style.colors.text_dim)) }),
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
            BCtrlKind::Check => self.flag("check_square", "square", ctx),
            BCtrlKind::Switch => self.switch(ctx),
            BCtrlKind::Select => self.select(ctx),
            BCtrlKind::Int { .. } | BCtrlKind::Number { .. } => self.number(ctx),
            BCtrlKind::Slider { .. } => self.slider(ctx),
            BCtrlKind::Text | BCtrlKind::Search | BCtrlKind::Path => self.text(ctx),
            BCtrlKind::Color => self.color(ctx),
        }
    }

    /// A boolean: one control that answers to its own state, drawn as a
    /// button carrying the glyph for the state it is in.
    ///
    /// It took the glyphs as `Option`s and fell back to a tab — fill says the
    /// state, no glyph — for `BCtrlKind::Toggle`, which no caller ever
    /// produced. The glyphs are glyphs again now that the fallback has gone
    /// with that variant.
    /// A checkbox: the box glyph, the accent when ticked and the dim text
    /// when not, the same size either way, then its label. It is a mark, not
    /// a button — no tile behind it and no fill when on — so a column of
    /// boxes reads as one column whichever are ticked.
    fn flag(&self, on_icon: &str, off_icon: &str, ctx: &BRenderCtx) -> AnyElement {
        let ctrl = self.0;
        let on = matches!(ctrl.value, BValue::Bool(true));
        let (glyph, color) = if on { (on_icon, ctx.style.colors.accent) } else { (off_icon, ctx.style.colors.text_dim) };
        let mut row = BCtrlPart::CheckRow
            .flex(ctx)
            .div(ctx)
            .child(BCtrlPart::CheckBox.flex(ctx).div(ctx).child(super::atom::icon_in(glyph, color, ctx)));
        if let Some(label) = &ctrl.label {
            row = row.child(BCtrlPart::SwitchLabel.flex(ctx).div(ctx).child(gpui::SharedString::from(label.clone())));
        }
        let id = gpui::SharedString::from(format!("check-{}", ctrl.key.encode()));
        let (handler, action) = (ctx.handler.clone(), ctrl.flip());
        row.id(id.clone())
            .debug_selector(move || id.to_string())
            .cursor_pointer()
            .on_click(move |_e: &gpui::ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
                (std::rc::Rc::clone(&handler))(&action, w, cx);
            })
            .into_any_element()
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
}


pub fn view(ctrl: &BCtrl, ctx: &BRenderCtx) -> AnyElement {
    BCtrlView(ctrl).render(ctx)
}
