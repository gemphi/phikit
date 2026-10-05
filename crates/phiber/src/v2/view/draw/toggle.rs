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
struct BZoneGlyph {
    zone: BZoneType,
    visible: bool,
}

impl BZoneGlyph {
    fn ink(&self) -> BInk {
        if self.visible { BInk::Accent } else { BInk::Dim }
    }

    fn radius(&self, ctx: &BRenderCtx) -> f32 {
        let scale = ctx.style.dimensions.icon_size / ctx.style.dimensions.button_height;
        ctx.style.radius_for(BStyleRole::Button) * scale
    }

    /// The frame: as tall as an icon, the zone laid against its side.
    fn frame(&self, ctx: &BRenderCtx) -> BFlex {
        let size = BLen::Measured(ctx.style.dimensions.icon_size);
        let (axis, justify) = match self.zone {
            BZoneType::Left => (BAxis::Row, BJustify::Start),
            BZoneType::Right => (BAxis::Row, BJustify::End),
            BZoneType::Top => (BAxis::Column, BJustify::Start),
            BZoneType::Bottom => (BAxis::Column, BJustify::End),
            BZoneType::Center | BZoneType::Full | BZoneType::Rail => (BAxis::Row, BJustify::Start),
        };
        BFlex::along(axis)
            .keeps_size()
            .width(size)
            .height(size)
            .justify(justify)
            .align(BAlign::Stretch)
            .radius(BRadius::All(self.radius(ctx)))
            .border(BEdge::Ink(self.ink()))
            .overflow(BOverflow::Clip)
    }

    /// The zone: a third of the frame, filled when shown, outlined when not.
    fn zone(&self, ctx: &BRenderCtx) -> BFlex {
        let third = BLen::Measured(ctx.style.dimensions.icon_size / 3.0);
        let cell = BFlex::cell().keeps_size();
        let cell = match self.zone {
            BZoneType::Left | BZoneType::Right => cell.width(third).height(BLen::Full),
            BZoneType::Top | BZoneType::Bottom => cell.width(BLen::Full).height(third),
            BZoneType::Center | BZoneType::Full | BZoneType::Rail => cell.full(),
        };
        let inner = (self.radius(ctx) - ctx.style.strokes.hairline).max(0.0);
        let cell = cell.radius(BRadius::All(inner));
        if self.visible { cell.fill(BFill::Ink(self.ink())) } else { cell.border(BEdge::Ink(self.ink())) }
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
    fn control(control: &BZoneToggle, ctx: &BRenderCtx) -> AnyElement {
        let id_text = format!("topbar-zone-toggle-{}", control.zone.slug());
        let handler = ctx.handler.clone();
        let action = control.action.clone();
        BZoneTogglePart::Button
            .flex(ctx)
            .div(ctx)
            .id(SharedString::from(id_text.clone()))
            .debug_selector(move || id_text)
            .cursor_pointer()
            .child(BZoneGlyph { zone: control.zone, visible: control.visible }.render(ctx))
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
