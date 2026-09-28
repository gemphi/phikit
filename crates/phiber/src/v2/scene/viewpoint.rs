//! Named viewpoints — Blender's `view3d.view_axis` (Numpad 1/3/7 and the
//! navigation gizmo's axis caps).
//!
//! A viewpoint is the pair of angles that define it. Clicking the compass's
//! Z cap has to change the pitch, not the yaw, so the caps write the
//! viewpoint's name and [`BViewpointRule`] turns it into both angles.

use crate::v2::state::{BKey, BPath, BState, View3dKey, rule::BRule};

/// The yaw and pitch of each named viewpoint, in Blender's convention:
/// the front view looks along +Y, and the top view looks straight down.
pub fn view_angles(name: &str) -> Option<(f32, f32)> {
    Some(match name {
        "front" => (0.0, 0.0),
        "back" => (180.0, 0.0),
        "right" => (90.0, 0.0),
        "left" => (270.0, 0.0),
        "top" => (0.0, 90.0),
        "bottom" => (0.0, -90.0),
        _ => return None,
    })
}

/// `view3d.view` → `view3d.yaw` + `view3d.pitch`; the name is not kept.
pub struct BViewpointRule;

impl BRule for BViewpointRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == View3dKey::View.path()
    }
    fn after(&self, _: &BPath, name: &str, s: &mut BState) {
        let Some((yaw, pitch)) = view_angles(name) else {
            return;
        };
        s.set(&super::yaw_key(), yaw.to_string());
        s.set(&super::pitch_key(), pitch.to_string());
        s.remove(View3dKey::View);
    }
}
