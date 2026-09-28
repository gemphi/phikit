//! The 3D view's own operators, as the View menu fires them.
//!
//! Blender ref: `view3d.view_all` (Home) and `view3d.view_selected`
//! (Numpad .) frame the scene or the selection — the pivot moves to the
//! bounds' centre and the distance fits them; `view3d.view_orbit` turns
//! the view a step; `view3d.zoom` steps the distance; `view3d.view_persportho`
//! flips projection. Each is one `Set` of `ctx.op.view3d`, turned here into
//! the view keys the canvas already reads.

use crate::v2::scene::BObject3D;
use crate::v2::scene::camera::zoom_for;
use crate::v2::state::{BKey as _, BPath, BState, OpKey, View3dKey, log, rule::BRule, val::{V_FALSE, V_TRUE}};
use glam::{Mat4, Vec3};

/// `view3d.view_orbit`'s step, degrees (Blender's `Preferences.view.rotation_angle`).
pub const ORBIT_STEP: f32 = 15.0;
/// `view3d.zoom`'s step, the gizmo's.
pub const ZOOM_STEP: f32 = 1.2;
/// Room left around what is framed (Blender's `VIEW3D_MARGIN`-ish).
const FRAME_MARGIN: f32 = 1.15;

/// The world-space bounds of `objects`: centre and the widest extent.
pub fn bounds_of(objects: &[&BObject3D]) -> Option<(Vec3, f32)> {
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for object in objects {
        let (olo, ohi) = extent_of(object);
        lo = lo.min(olo);
        hi = hi.max(ohi);
    }
    lo.is_finite().then(|| ((lo + hi) * 0.5, (hi - lo).max_element().max(1e-3)))
}

/// One object's world-space bounds, `(lo, hi)`; infinite when it has no
/// vertices.
pub fn extent_of(object: &BObject3D) -> (Vec3, Vec3) {
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for v in world_vertices(object) {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    (lo, hi)
}

fn world_vertices(object: &BObject3D) -> Vec<Vec3> {
    match object {
        BObject3D::Primitive { shape, x, y, .. } => {
            let (v, ..) = crate::phiphx::spatial::SpatialObject3D::buffers(shape.mesh_type());
            let (ox, oy) = (x / super::MESH_SCALE, y / super::MESH_SCALE);
            v.iter().map(|v| Vec3::new(ox + v.0, oy + v.2, v.1)).collect()
        }
        BObject3D::Loaded { mesh, matrix_world, .. } => {
            let m = Mat4::from_cols_array_2d(matrix_world);
            mesh.vertices.iter().map(|v| m.transform_point3(Vec3::from(*v))).collect()
        }
    }
}

/// Frame `objects`: the pivot on their centre, the pan cleared, the zoom
/// showing their widest side with room around it.
fn frame(state: &mut BState, objects: &[&BObject3D]) -> bool {
    let Some((centre, extent)) = bounds_of(objects) else { return false };
    state.set(&super::pivot_key(), format!("{},{},{}", centre.x, centre.y, centre.z));
    state.set(&super::zoom_key(), zoom_for(extent * FRAME_MARGIN).to_string());
    state.set(&super::pan_x_key(), "0");
    state.set(&super::pan_y_key(), "0");
    true
}

/// `ctx.op.view3d` → the view keys.
pub struct BView3dOpRule;

impl BRule for BView3dOpRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::View3d.path()
    }
    fn after(&self, _: &BPath, op: &str, state: &mut BState) {
        let objects = super::source::scene_objects(state);
        let hidden = crate::v2::part::outliner::key::BHidden::now(state);
        let visible: Vec<&BObject3D> = objects.iter().filter(|o| !hidden.hides(state, o.name())).collect();
        let turn = |state: &mut BState, key: View3dKey, by: f32, wrap: bool| {
            let now = if wrap { super::view_yaw(state) } else { super::view_pitch(state) };
            let next = if wrap { (now + by).rem_euclid(360.0) } else { (now + by).clamp(-90.0, 90.0) };
            state.set(&super::view_slot::BViewSlot::key(key), next.to_string());
        };
        let zoom = |state: &mut BState, factor: f32| {
            let next = super::zoom_step(super::view_zoom(state), factor);
            state.set(&super::zoom_key(), next.to_string());
        };
        let said = match op {
            "frame_all" => frame(state, &visible).then_some("Framed all"),
            "frame_selected" => {
                let selection = super::select::BSelection::now(state);
                let selected: Vec<&BObject3D> = visible.iter().copied().filter(|o| selection.contains(o.name())).collect();
                frame(state, &selected).then_some("Framed the selection")
            }
            "orbit_left" => { turn(state, View3dKey::Yaw, -ORBIT_STEP, true); Some("Orbited left") }
            "orbit_right" => { turn(state, View3dKey::Yaw, ORBIT_STEP, true); Some("Orbited right") }
            "orbit_up" => { turn(state, View3dKey::Pitch, ORBIT_STEP, false); Some("Orbited up") }
            "orbit_down" => { turn(state, View3dKey::Pitch, -ORBIT_STEP, false); Some("Orbited down") }
            "zoom_in" => { zoom(state, ZOOM_STEP); Some("Zoomed in") }
            "zoom_out" => { zoom(state, 1.0 / ZOOM_STEP); Some("Zoomed out") }
            "persportho" => {
                let persp = super::is_perspective(state);
                state.set(&super::perspective_key(), if persp { V_FALSE } else { V_TRUE });
                Some(if persp { "Orthographic view" } else { "Perspective view" })
            }
            // `view3d.view_axis`: the named viewpoint's two angles.
            named if named.starts_with("viewpoint:") => {
                let name = &named["viewpoint:".len()..];
                match super::viewpoint::view_angles(name) {
                    Some((yaw, pitch)) => {
                        state.set(&super::yaw_key(), yaw.to_string());
                        state.set(&super::pitch_key(), pitch.to_string());
                        log::push(format!("View: {}{}", name[..1].to_uppercase(), &name[1..]));
                    }
                    None => {}
                }
                None
            }
            _ => None,
        };
        if let Some(said) = said {
            log::push(said.to_string());
        }
        state.remove(OpKey::View3d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::camera::view_pivot;
    use crate::v2::state::{CtxKey, action::apply_action};
    use crate::v2::view::prim::BAction;

    fn op(state: &mut BState, what: &str) {
        apply_action(state, &BAction::Set(OpKey::View3d.path(), what.into()));
    }

    #[test]
    fn frame_all_puts_the_pivot_on_the_scenes_centre_and_frame_selected_on_the_objects() {
        crate::v2::scene::source::clear_loaded_scene();
        crate::v2::scene::source::clear_scene_objects();
        let mut state = BState::new();
        state.set(View3dKey::PanX, "0.3");
        op(&mut state, "frame_all");
        let all = view_pivot(&state);
        assert_eq!(super::super::view_pan(&state), (0.0, 0.0), "the pan is cleared");
        let objects = crate::v2::scene::current_scene(&state, None);
        let (centre, _) = bounds_of(&objects.iter().collect::<Vec<_>>()).unwrap();
        assert!((Vec3::from(all) - centre).length() < 1e-4);
        state.set(CtxKey::Object, "Gem");
        crate::v2::scene::select::BSelection { list: vec!["Gem".into()] }.write(&mut state);
        op(&mut state, "frame_selected");
        let gem = objects.iter().find(|o| o.name() == "Gem").unwrap();
        let (gem_centre, _) = bounds_of(&[gem]).unwrap();
        assert!((Vec3::from(view_pivot(&state)) - gem_centre).length() < 1e-4);
        assert!(crate::v2::scene::view_zoom(&state) > 1.0, "a single gem is framed closer than the whole scene");
        assert!(state.get(OpKey::View3d).is_none());
    }

    #[test]
    fn orbit_zoom_and_projection_step_the_view() {
        let mut state = BState::new();
        // From the view as drawn — 30° / 30°, Blender's startup angles — not
        // from zero: an orbit used to jump the view before it turned it.
        let (yaw, pitch) = (super::super::view_yaw(&state), super::super::view_pitch(&state));
        op(&mut state, "orbit_right");
        assert_eq!(state.get(View3dKey::Yaw).and_then(|v| v.parse::<f32>().ok()), Some(yaw + ORBIT_STEP));
        op(&mut state, "orbit_up");
        op(&mut state, "orbit_up");
        assert_eq!(state.get(View3dKey::Pitch).and_then(|v| v.parse::<f32>().ok()), Some(pitch + 2.0 * ORBIT_STEP));
        let before = crate::v2::scene::view_zoom(&state);
        op(&mut state, "zoom_in");
        assert!(crate::v2::scene::view_zoom(&state) > before);
        assert!(super::super::is_perspective(&state));
        op(&mut state, "persportho");
        assert!(!super::super::is_perspective(&state));
    }
}
