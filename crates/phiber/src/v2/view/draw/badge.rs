//! The badge — an icon with its count, as the Outliner's icon row draws what
//! a collapsed item holds (`outliner_draw_iconrow`): the kind's glyph, and
//! when there is more than one of that kind, the number in a small rounded
//! box on the card shade (`outliner_draw_iconrow_number`, caption size, a
//! step dimmer than the name). One of a kind is the glyph alone. Hovering
//! names what is held ("Mesh × 3"), as Blender's tooltip does.

use super::atom::icon_path;
use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::flex::{BAlign, BFill, BFlex, BInk, BRadius, BSpace, BType};
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, px, svg};
use super::tab::cap::BTabTooltip;

/// `outliner_draw_iconrow_number`: the number is drawn at 0.8 of the icon.
const NUMBER_ICON_SHARE: f32 = 0.8;

/// What the tooltip says: the kind, and the count when there are several.
pub fn caption(icon: &str, count: usize) -> String {
    let mut kind: Vec<char> = icon.replace('_', " ").chars().collect();
    if let Some(first) = kind.first_mut() {
        *first = first.to_ascii_uppercase();
    }
    let kind: String = kind.into_iter().collect();
    if count < 2 { kind } else { format!("{kind} × {count}") }
}

pub fn view(icon: &str, count: usize, ctx: &BRenderCtx) -> AnyElement {
    let size = ctx.style.dimensions.icon_size * NUMBER_ICON_SHARE;
    let tip = caption(icon, count);
    let style = *ctx.style;
    let glyph = svg().path(icon_path(icon)).size(px(size)).text_color(to_hsla(ctx.style.colors.text_dim));
    let badge = BFlex::row()
        .keeps_size()
        .align(BAlign::Center)
        .gap(BSpace::None)
        .type_size(BType::Caption)
        .ink_tone(BInk::Dim)
        .div(ctx)
        .id(SharedString::from(format!("badge-{icon}")))
        .debug_selector(move || format!("badge-{icon}"))
        .tooltip(move |_, cx| cx.new(|_| BTabTooltip::of(&tip, &style)).into())
        .child(glyph);
    if count < 2 {
        return badge.into_any_element();
    }
    let number = BFlex::cell()
        .keeps_size()
        .pad_x(BSpace::PadXs)
        .radius(BRadius::Role(BStyleRole::Field))
        .fill(BFill::Tone(BSurfaceTone::Card))
        .div(ctx)
        .child(SharedString::from(count.to_string()));
    badge.child(number).into_any_element()
}
