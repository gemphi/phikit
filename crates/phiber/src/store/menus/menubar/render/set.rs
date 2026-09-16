//! Render menu sets - labels, icons, shortcuts.

use super::def::*;
use crate::store::menus::MenuItem;

pub static ITEMS: &[MenuItem] = &[
    MenuItem::node(RENDER_IMAGE, RENDER_IMAGE_ICON).with_shortcut(RENDER_IMAGE_SHORTCUT),
    MenuItem::node(RENDER_ANIMATION, RENDER_ANIMATION_ICON)
        .with_shortcut(RENDER_ANIMATION_SHORTCUT),
    MenuItem::node(RENDER_AUDIO, RENDER_AUDIO_ICON),
    MenuItem::separator(),
    MenuItem::node(VIEW_RENDER, VIEW_RENDER_ICON).with_shortcut(VIEW_RENDER_SHORTCUT),
    MenuItem::node(PLAY_RENDERED_ANIMATION, PLAY_RENDERED_ANIMATION_ICON)
        .with_shortcut(PLAY_RENDERED_ANIMATION_SHORTCUT),
    MenuItem::separator(),
    MenuItem::node(LOCK_INTERFACE, LOCK_INTERFACE_ICON),
];
