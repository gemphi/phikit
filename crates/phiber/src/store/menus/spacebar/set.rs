//! Spacebar menu sets - workspace tab items.

use super::def::*;
use crate::store::menus::MenuItem;

pub static SPACE_BAR_ITEMS: &[MenuItem] = &[
    MenuItem::node(SPACE_BAR_LAYOUT, SPACE_BAR_LAYOUT_ICON),
    MenuItem::node(SPACE_BAR_MODELING, SPACE_BAR_MODELING_ICON),
    MenuItem::node(SPACE_BAR_SCULPTING, SPACE_BAR_SCULPTING_ICON),
    MenuItem::node(SPACE_BAR_UV, SPACE_BAR_UV_ICON),
    MenuItem::node(SPACE_BAR_TEXTURE_PAINT, SPACE_BAR_TEXTURE_PAINT_ICON),
    MenuItem::node(SPACE_BAR_SHADING, SPACE_BAR_SHADING_ICON),
    MenuItem::node(SPACE_BAR_ANIMATION, SPACE_BAR_ANIMATION_ICON),
    MenuItem::node(SPACE_BAR_RENDERING, SPACE_BAR_RENDERING_ICON),
    MenuItem::node(SPACE_BAR_COMPOSITING, SPACE_BAR_COMPOSITING_ICON),
];
