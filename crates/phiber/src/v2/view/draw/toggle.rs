//! Stable, rounded Topbar controls for canonical zone visibility — the
//! button and the glyph inside it both take the theme's radius.

use super::ctx::BRenderCtx;
use super::flex::{
    BAlign, BDim, BEdge, BFill, BFlex, BFlexed, BInk, BJustify, BLen, BOverflow, BRadius, BSpace,
};
use crate::v2::theme::BStyleRole;
use crate::v2::view::layout::BAxis;
use crate::v2::view::BZoneToggle;
use crate::v2::zone::BZoneType;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString};
use std::rc::Rc;

/// The glyph: a window frame with the toggled zone filled — its left third,
/// its right third or its bottom third — drawn as the theme draws a button,
/// so its corners take the theme's radius at the glyph's scale (a square
/// theme draws it square, a pill theme rounds it) instead of a fixed icon.
/// A hidden zone shows the frame and the zone's outline only.
///
/// The glyph is a DIAGRAM of the window, and only one part of that diagram
/// is in question. The frame is the window: it is there either way, so it
/// holds one ink and does not move. The zone's third is the answer, so it
/// takes the accent when the zone is shown and falls to a dim outline when
/// it is not.
///
/// Both took the same ink before, so toggling a zone recoloured the whole
/// glyph — frame and all — which reads as "this button turned off" rather
/// than "that zone went away", and left the accent spread across a shape
/// that was mostly not the thing it was describing.
struct BZoneGlyph {
    zone: BZoneType,
    visible: bool,
}

impl BZoneGlyph {
    /// The window, which is always there.
    fn frame_ink(&self) -> BInk {
        BInk::Text
    }

    /// The zone, which is the question.
    fn zone_ink(&self) -> BInk {
        if self.visible { BInk::Accent } else { BInk::Dim }
    }

    fn radius(&self, ctx: &BRenderCtx) -> f32 {
        let scale = ctx.style.dimensions.icon_size / ctx.style.dimensions.button_height;
        ctx.style.radius_for(BStyleRole::Button) * scale
    }

    /// Where in the frame this zone sits: the axis it is laid along, the end
    /// it is pushed to, and how it sits across that axis.
    ///
    /// A space is divided left / centre / right across and top / centre /
    /// bottom down, so there are five places a zone can be and the glyph owes
    /// each of them its own picture. Centre, `Full` and `Rail` used to share
    /// the Left arm of this match, which drew a Centre toggle as a Left one —
    /// the glyph said the wrong zone, which is the whole job it has.
    fn place(&self) -> (BAxis, BJustify, BAlign) {
        match self.zone {
            BZoneType::Left => (BAxis::Row, BJustify::Start, BAlign::Stretch),
            BZoneType::Right => (BAxis::Row, BJustify::End, BAlign::Stretch),
            BZoneType::Top => (BAxis::Column, BJustify::Start, BAlign::Stretch),
            BZoneType::Bottom => (BAxis::Column, BJustify::End, BAlign::Stretch),
            // The middle, flanked on all four sides.
            BZoneType::Center => (BAxis::Row, BJustify::Center, BAlign::Center),
            // The whole window, and the rail down its far left.
            BZoneType::Full => (BAxis::Row, BJustify::Center, BAlign::Stretch),
            BZoneType::Rail => (BAxis::Row, BJustify::Start, BAlign::Stretch),
        }
    }

    /// The frame: as tall as an icon, the zone laid where `place` puts it.
    fn frame(&self, ctx: &BRenderCtx) -> BFlex {
        let size = BLen::Measured(ctx.style.dimensions.icon_size);
        let (axis, justify, align) = self.place();
        BFlex::along(axis)
            .keeps_size()
            .width(size)
            .height(size)
            .justify(justify)
            .align(align)
            .radius(BRadius::All(self.radius(ctx)))
            .border(BEdge::Ink(self.frame_ink()))
            .overflow(BOverflow::Clip)
    }

    /// The zone's share of the frame: a third against the side it is on, the
    /// middle third both ways for Centre, a thin rail, the lot for `Full`.
    /// Filled when shown, outlined when not.
    fn zone(&self, ctx: &BRenderCtx) -> BFlex {
        let icon = ctx.style.dimensions.icon_size;
        let third = BLen::Measured(icon / 3.0);
        let rail = BLen::Measured((icon / 5.0).max(ctx.style.strokes.hairline * 2.0));
        let cell = BFlex::cell().keeps_size();
        let cell = match self.zone {
            BZoneType::Left | BZoneType::Right => cell.width(third).height(BLen::Full),
            BZoneType::Top | BZoneType::Bottom => cell.width(BLen::Full).height(third),
            // Flanked both ways, so it is the middle third of each.
            BZoneType::Center => cell.width(third).height(third),
            BZoneType::Rail => cell.width(rail).height(BLen::Full),
            BZoneType::Full => cell.full(),
        };
        let inner = (self.radius(ctx) - ctx.style.strokes.hairline).max(0.0);
        let cell = cell.radius(BRadius::All(inner));
        match self.visible {
            true => cell.fill(BFill::Ink(self.zone_ink())),
            false => cell.border(BEdge::Ink(self.zone_ink())),
        }
    }

    fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        self.frame(ctx).div(ctx).child(self.zone(ctx).div(ctx)).into_any_element()
    }
}

/// The zone toggles in the topbar, drawn.
///
/// The group and the one control are methods of the same type: a group is
/// its controls, and a control is its glyph on a button — neither is a free
/// function that happens to take the other's data.
pub struct BZoneToggles<'a>(pub &'a [BZoneToggle]);

impl BZoneToggles<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let selector = "topbar-zone-toggle-group";
        let mut group = BZoneTogglePart::Group
            .flex(ctx)
            .div(ctx)
            .id(selector)
            .debug_selector(|| selector.to_string());
        for item in self.0 {
            group = group.child(Self::control(item, ctx));
        }
        group.into_any_element()
    }

    /// One toggle: the zone's glyph on a button that lights under the
    /// pointer, as every other control outside a header strip does.
    ///
    /// The on/off state is the control's `active` now, not a bool this file
    /// reads and colours for itself; the glyph asks it rather than being told.
    ///
    /// The button does NOT take the active fill, though the control is
    /// `active` and every other control with `active` does. A zone toggle's
    /// glyph already draws its answer, so a fill behind it would say the same
    /// thing twice — and say it in the accent, over an accent third, which
    /// loses the part that carries the meaning. It is the same call Blender
    /// makes for a restriction toggle: the glyph swaps and the button stays
    /// quiet (`IconToggle` over a pair of icons, `part/outliner/toggle::set`).
    /// With most zones shown most of the time, the alternative is a row of
    /// filled blocks in the topbar reporting what the window already shows.
    fn control(toggle: &BZoneToggle, ctx: &BRenderCtx) -> AnyElement {
        let id_text = format!("topbar-zone-toggle-{}", toggle.zone.slug());
        let handler = ctx.handler.clone();
        let Some(action) = toggle.control.action.clone() else {
            // A toggle with nothing to fire is a picture; draw it as one
            // rather than a button that does nothing when pressed.
            return BZoneGlyph { zone: toggle.zone, visible: toggle.control.active }.render(ctx);
        };
        let mut button = BZoneTogglePart::Button
            .flex(ctx)
            .div(ctx)
            .id(SharedString::from(id_text.clone()))
            .debug_selector(move || id_text)
            .cursor_pointer()
            .child(BZoneGlyph { zone: toggle.zone, visible: toggle.control.active }.render(ctx));
        // Attached before `on_click`, which takes the element by value
        // (`draw/tab/mod.rs` says the same where it does this).
        if let Some(tip) = toggle.control.tooltip.as_deref() {
            let (tip, style) = (tip.to_string(), *ctx.style);
            button = button.tooltip(move |_, cx| cx.new(|_| super::tab::cap::BTabTooltip::of(&tip, &style)).into());
        }
        button
            .on_click(move |_event: &ClickEvent, window: &mut gpui::Window, cx: &mut gpui::App| {
                (Rc::clone(&handler))(&action, window, cx);
            })
            .into_any_element()
    }
}

pub fn group(controls: &[BZoneToggle], ctx: &BRenderCtx) -> AnyElement {
    BZoneToggles(controls).render(ctx)
}

/// The parts of the zone-toggle group.
enum BZoneTogglePart {
    /// The group: the toggles pushed to the strip's end.
    Group,
    /// One toggle's button — square, one button height, rounded as every
    /// button is.
    Button,
}

impl BFlexed for BZoneTogglePart {
    fn flex(&self, _ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Group => BFlex::row()
                .align(BAlign::Center)
                .justify(BJustify::End)
                .keeps_size()
                .gap(BSpace::GapXs),
            Self::Button => {
                let size = BLen::Dim(BDim::ButtonHeight);
                BFlex::row()
                    .align(BAlign::Center)
                    .justify(BJustify::Center)
                    .keeps_size()
                    .width(size)
                    .height(size)
                    .radius(BRadius::Role(BStyleRole::Button))
                    .hovered(BFill::ItemHover)
            }
        }
    }
}

#[cfg(test)]
#[path = "toggle_tests.rs"]
mod tests;
