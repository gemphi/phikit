//! Boot defaults — the initial state a fresh `BState` starts with.
//! Every write uses a `pub const BPath` key + a `pub const &str` value —
//! zero raw strings.

use super::{BState, key::*, val::*};
use crate::v2::space::BSpaceType;

pub fn default_state() -> BState {
    let mut s = BState::new();
    s.set(&K_SPACE, BSpaceType::Layout.slug());
    s.set(&K_SCENE, V_SCENE_DEFAULT);
    s.set(&K_VIEW_LAYER, V_VIEW_LAYER_DEFAULT);
    s.set(&K_OBJECT, V_OBJECT_DEFAULT);
    s.set(&K_MODE, V_MODE_OBJECT);
    s.set(&K_TOOL, V_TOOL_SELECT_BOX);
    s.set(&K_FRAME, V_FRAME_START);
    s.set(&K_PLAYING, V_FALSE);
    s.set(&K_THEME, V_THEME_APPLE_DARK);
    // Boot into the welcome window (Blender's splash). Any click dismisses
    // it; `Blender ▸ Splash Screen` brings it back.
    s.set(&K_MODAL, V_MODAL_SPLASH);
    s.set(&K_ZONE_LEFT, V_TRUE);
    s.set(&K_ZONE_RIGHT, V_TRUE);
    s.set(&K_ZONE_BOTTOM, V_TRUE);
    // Properties context — Blender's default active tab is "Active Tool".
    s.set(
        &crate::v2::state::BPath::owned(vec!["buts", "navbar", "tab"]),
        "Active Tool",
    );

    // Blender Shading defaults: upper editors retain useful working room,
    // while Properties gets the taller share of the right column. These are
    // ordinary BState area fractions consumed by the generic zone renderer.
    for (zone, fractions) in [
        ("left", [0.58_f32, 0.42_f32]),
        ("center", [0.55_f32, 0.45_f32]),
        ("right", [0.22_f32, 0.78_f32]),
    ] {
        for (index, fraction) in fractions.into_iter().enumerate() {
            s.set_encoded(
                &format!("ctx.area_size.Shading.{zone}.{index}"),
                fraction.to_string(),
            );
        }
    }

    s
}
