//! Boot defaults — the initial state a fresh `BState` starts with.
//! Every write uses a `pub const BPath` key + a `pub const &str` value —
//! zero raw strings.

use crate::v2::state::{AppKey, CtxKey, ModalKey};
use super::{BState, val::*};
use crate::v2::space::BSpaceType;
use crate::v2::zone::BZoneType;

pub fn default_state() -> BState {
    let mut s = BState::new();
    s.set(CtxKey::Space, BSpaceType::Layout.slug());
    s.set(CtxKey::Scene, V_SCENE_DEFAULT);
    s.set(CtxKey::ViewLayer, V_VIEW_LAYER_DEFAULT);
    s.set(CtxKey::Object, V_OBJECT_DEFAULT);
    s.set(CtxKey::Mode, V_MODE_OBJECT);
    s.set(CtxKey::Tool, V_TOOL_SELECT_BOX);
    s.set(CtxKey::Frame, V_FRAME_START);
    s.set(CtxKey::Playing, V_FALSE);
    // The default palette has one owner — `BPaletteName::DEFAULT`. Naming
    // a theme here as well meant two, and they disagreed: the constant
    // said Silver and the seed wrote Sphere over it on every fresh state.
    s.set(AppKey::Theme, crate::v2::theme::BPaletteName::DEFAULT.slug());
    // Boot into the welcome window (Blender's splash). Any click dismisses
    // it; `Blender ▸ Splash Screen` brings it back.
    s.set(ModalKey::Open, V_MODAL_SPLASH);
    for zone in crate::v2::zone::visibility::TOGGLEABLE {
        s.set(super::ZoneKey::Shown(zone), V_TRUE);
    }
    // Properties context — Blender's default active tab is "Active Tool".
    s.set(crate::v2::state::ButsKey::NavbarTab, "Active Tool");

    // Blender Shading defaults: upper editors retain useful working room,
    // while Properties gets the taller share of the right column. These are
    // ordinary BState area fractions consumed by the generic zone renderer.
    for (zone, fractions) in [
        (BZoneType::Left, [0.58_f32, 0.42_f32]),
        (BZoneType::Center, [0.55_f32, 0.45_f32]),
        (BZoneType::Right, [0.22_f32, 0.78_f32]),
    ] {
        for (index, fraction) in fractions.into_iter().enumerate() {
            s.set(super::AreaKey::Size(BSpaceType::Shading, zone, index), fraction.to_string());
        }
    }
    // Zone sizes read from Blender's `startup.blend` (1920 × 1080, see
    // `tests/truth/workspaces.txt`), scaled to the 1600-px window the layout
    // golden records: the Animation workspace's Dope Sheet takes 433 of
    // 1054 px and its camera viewport 559 of 1920; the Compositing Dope
    // Sheet 189 px; the Geometry Nodes node editor and Timeline 551 px
    // together, with the Spreadsheet 654 wide; the Scripting column 558 wide;
    // the UV and Texture Paint image editors 786 wide.
    for (space, zone, size) in [
        (BSpaceType::Animation, BZoneType::Bottom, V_ANIMATION_BOTTOM_HEIGHT),
        (BSpaceType::Animation, BZoneType::Left, V_CAMERA_VIEW_WIDTH),
        (BSpaceType::Compositing, BZoneType::Bottom, V_COMPOSITING_BOTTOM_HEIGHT),
        (BSpaceType::GeometryNodes, BZoneType::Bottom, V_GEOMETRY_NODES_BOTTOM_HEIGHT),
        (BSpaceType::GeometryNodes, BZoneType::Left, V_SPREADSHEET_WIDTH),
        (BSpaceType::Scripting, BZoneType::Left, V_SCRIPTING_LEFT_WIDTH),
        (BSpaceType::UvEditing, BZoneType::Left, V_IMAGE_COLUMN_WIDTH),
        (BSpaceType::TexturePaint, BZoneType::Left, V_IMAGE_COLUMN_WIDTH),
    ] {
        s.set(super::ZoneKey::Size(space, zone), size);
    }
    // Shares inside a zone, from the same file: the Geometry Nodes node
    // editor over its Timeline (477 : 74, the Timeline kept tall enough for its
    // strip and a row), and Scripting's viewport, console
    // and Info log (393 : 463 : 169).
    for (space, zone, fractions) in [
        (BSpaceType::GeometryNodes, BZoneType::Bottom, &[0.8_f32, 0.2_f32][..]),
        (BSpaceType::Scripting, BZoneType::Left, &[0.38_f32, 0.45_f32, 0.17_f32][..]),
    ] {
        for (index, fraction) in fractions.iter().enumerate() {
            s.set(super::AreaKey::Size(space, zone, index), fraction.to_string());
        }
    }

    s
}
