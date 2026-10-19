//! Blender's number button (`UI_BTYPE_NUM`): one rounded field with the
//! label at its start and the value at its end, a caret in it while it is
//! typed into, and a step arrow at each end that shows only while the
//! pointer is over the field. Clicking the text puts the caret there; the
//! arrows step the value.

use super::super::flex::{BAlign, BFlex, BFlexed, BJustify, BLen, BPlace, BSide, BSpace};
use super::super::ctx::BRenderCtx;
use super::super::text_input::BFieldText;
use super::super::{atom, button, field};
use crate::v2::view::prim::{BControl, BCtrl};
use gpui::prelude::*;
use gpui::{AnyElement, MouseButton, MouseDownEvent};

/// The hover group a number field and its step arrows share.
const NUMBER_GROUP: &str = "number-field";

enum BNumberPart {
    /// The field's frame: the anchor its arrows float against, and the hover
    /// group that reveals them.
    Frame,
    /// The label and the value — spread apart when there is a label
    /// (Blender: `Start        1`), centred otherwise.
    Body { labelled: bool },
    /// One step arrow, floating over the field's end, shown on hover only —
    /// Blender's number buttons draw theirs the same way.
    Step(BSide),
}

impl BFlexed for BNumberPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Frame => BFlex::row().fills().min_width(BLen::Zero).align(BAlign::Center).place(BPlace::Anchor).hover_group(NUMBER_GROUP),
            Self::Body { labelled } => BFlex::row()
                .fills()
                // Not `Zero`: that is flexbox's permission to shrink below
                // the content, and it is what clipped the value.
                .min_width(BLen::Measured(ctx.style.dimensions.number_min_width))
                .align(BAlign::Center)
                .justify(if *labelled { BJustify::Between } else { BJustify::Center })
                .gap(BSpace::GapSm)
                .pad_x(BSpace::PadMd),
            Self::Step(side) => BFlex::row()
                .align(BAlign::Center)
                .place(BPlace::Float { side: *side, inset: BSpace::None, foot: BSpace::None })
                .revealed_by(NUMBER_GROUP),
        }
    }
}

impl super::BCtrlView<'_> {
    /// A number field: its value between two steppers.
    ///
    /// A method, like every other kind, because `BCtrlView`'s own doc says so
    /// — *"Each kind is a method of this type rather than a free function
    /// beside it, so the widget a kind draws as is reached through the
    /// control, not from anywhere"* — and this was the one arm of `render`
    /// that reached a free function instead, ten lines under that sentence.
    pub(super) fn number(&self, ctx: &BRenderCtx) -> AnyElement {
        build(self.0, ctx)
    }
}

/// The field itself, kept as a function because it is long and the method
/// above is the way in.
fn build(ctrl: &BCtrl, ctx: &BRenderCtx) -> AnyElement {
    let step = |icon: &str, steps: f32, side: BSide| {
        let mut ctl = BControl::new().with_icon(icon);
        if let Some(action) = ctrl.nudge(steps) {
            ctl = ctl.with_action(action);
        }
        BNumberPart::Step(side).flex(ctx).div(ctx).child(button::view(&ctl, ctx))
    };
    let shown = ctrl.value.display();
    let focus = ctrl.focus();
    let id = button::element_id("number", ctrl.label.as_deref(), &focus);
    let handler = ctx.handler.clone();
    // Typed into, with a window behind the pass: the value is the real
    // caret-bearing element, so the platform's text handler inserts into it.
    // Without that element the field had a caret drawn but nowhere for a
    // printable key to land — which is why typing 64 into Max Samples did
    // nothing while Backspace and the arrows worked.
    let typing = ctrl.editing.then(|| BFieldText::of(&shown, "", ctx)).flatten();
    let caret_at = typing.as_ref().map(|t| t.app.clone());
    let mut body = BNumberPart::Body { labelled: ctrl.label.is_some() }
        .flex(ctx)
        .div(ctx)
        .id(gpui::SharedString::from(id.clone()))
        .debug_selector(move || id)
        .cursor_pointer()
        .on_click(move |_, window, app| (std::rc::Rc::clone(&handler))(&focus, window, app));
    if let Some(label) = &ctrl.label {
        body = body.child(atom::label(label, ctx));
    }
    body = match typing {
        Some(field_text) => body.child(field_text),
        // No window behind the pass (a headless render, a test): the pipe
        // stands in for the caret the element would have drawn.
        None if ctrl.editing => body.child(atom::label(&format!("{shown}|"), ctx)),
        None => body.child(atom::label(&shown, ctx)),
    };
    let mut shell = field::shell(ctrl.editing, ctx).child(body);
    if let Some(app) = caret_at {
        shell = shell.cursor_text().on_mouse_down(MouseButton::Left, move |ev: &MouseDownEvent, _window, cx| {
            let (pos, shift) = (ev.position, ev.modifiers.shift);
            app.update(cx, |app, cx| {
                app.place_caret_at_point(pos, shift);
                cx.notify();
            });
        });
    }
    BNumberPart::Frame
        .flex(ctx)
        .div(ctx)
        .child(shell)
        .child(step("chevron_left", -1.0, BSide::Left))
        .child(step("chevron_right", 1.0, BSide::Right))
        .into_any_element()
}
