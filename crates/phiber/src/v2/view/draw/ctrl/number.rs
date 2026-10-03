//! Blender's number button (`UI_BTYPE_NUM`): one rounded field with the
//! label at its start and the value at its end, a caret in it while it is
//! typed into, and a step arrow at each end that shows only while the
//! pointer is over the field. Clicking the text puts the caret there; the
//! arrows step the value.

use super::super::flex::{BAlign, BFlex, BFlexed, BJustify, BLen, BPlace, BSide, BSpace};
use super::super::ctx::BRenderCtx;
use super::super::{atom, button, field};
use crate::v2::view::prim::{BControl, BCtrl};
use gpui::prelude::*;
use gpui::AnyElement;

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

pub(super) fn number(ctrl: &BCtrl, ctx: &BRenderCtx) -> AnyElement {
    let step = |icon: &str, steps: f32, side: BSide| {
        let mut ctl = BControl::new().with_icon(icon);
        if let Some(action) = ctrl.nudge(steps) {
            ctl = ctl.with_action(action);
        }
        BNumberPart::Step(side).flex(ctx).div(ctx).child(button::view(&ctl, ctx))
    };
    let shown = ctrl.value.display();
    let text = if ctrl.editing { format!("{shown}|") } else { shown };
    let focus = ctrl.focus();
    let id = button::element_id("number", ctrl.label.as_deref(), &focus);
    let handler = ctx.handler.clone();
    let mut body = BNumberPart::Body { labelled: ctrl.label.is_some() }
        .flex(ctx)
        .div(ctx)
        .id(gpui::SharedString::from(id.clone()))
        .debug_selector(move || id.clone())
        .cursor_pointer()
        .on_click(move |_, window, app| (std::rc::Rc::clone(&handler))(&focus, window, app));
    if let Some(label) = &ctrl.label {
        body = body.child(atom::label(label, ctx));
    }
    body = body.child(atom::label(&text, ctx));
    BNumberPart::Frame
        .flex(ctx)
        .div(ctx)
        .child(field::shell(ctrl.editing, ctx).child(body))
        .child(step("chevron_left", -1.0, BSide::Left))
        .child(step("chevron_right", 1.0, BSide::Right))
        .into_any_element()
}
