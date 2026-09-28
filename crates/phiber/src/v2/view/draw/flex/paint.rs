//! The one place a `BShadowStyle` meets GPUI's shadow rungs.
//!
//! Split out of `apply.rs` for the file cap. Keeping it alone is the point:
//! a lifted container, a popover and a modal reach their depth through this
//! function, so the elevation scale (`theme::BElevation`) has exactly one
//! place where it becomes pixels.

use crate::v2::theme::BShadowStyle;
use gpui::prelude::*;

/// `element` casting the theme's surface shadow — nothing for `None`. The
/// one place a `BShadowStyle` meets GPUI's shadow rungs, so a lifted
/// container and a popover cast the same shadow in a given theme.
pub(in crate::v2::view::draw) fn shadowed<E: Styled>(element: E, shadow: BShadowStyle) -> E {
    match shadow {
        BShadowStyle::None => element,
        BShadowStyle::Small => element.shadow_sm(),
        BShadowStyle::Medium => element.shadow_md(),
        BShadowStyle::Large => element.shadow_lg(),
        BShadowStyle::ExtraLarge => element.shadow_xl(),
    }
}
