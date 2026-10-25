//! The controls that open or draw something else — a dropdown's menu, a
//! slider's track, a search field, a colour picker. One `impl BCtrlView`
//! block, split out so `mod.rs` stays under the 240-line read cap
//! (`tests/level_structure.rs`); the glyph controls (`check`, `switch`) and
//! the render dispatch (`render`) stays in the mod with the enum they match on,
//! and `number` has its own file.

use super::*;

impl BCtrlView<'_> {
/// A dropdown: the chosen option on a cap, its options in a popover. With
/// nowhere to record open-ness it is a plain button showing the choice.
pub(super) fn select(&self, ctx: &BRenderCtx) -> AnyElement {
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
pub(super) fn slider(&self, ctx: &BRenderCtx) -> AnyElement {
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
pub(super) fn text(&self, ctx: &BRenderCtx) -> AnyElement {
    let ctrl = self.0;
    let placeholder = ctrl.label.clone().unwrap_or_default();
    let shown = ctrl.value.display();
    search::view(&placeholder, &shown, (ctrl.editing, false), &ctrl.set(ctrl.value.clone()), ctx)
}

/// A swatch of the value, and the value as text beside it.
///
/// The swatch takes the value in whichever spelling the control carries: a
/// scene property hands it RGBA, a theme preference hands it the HSL text
/// its descriptor formats. This used to match `BValue::Color` and nothing
/// else, falling back to opaque black — and every one of the 29 colours in
/// Preferences is the text form, so every swatch there was black.
pub(super) fn color(&self, ctx: &BRenderCtx) -> AnyElement {
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
