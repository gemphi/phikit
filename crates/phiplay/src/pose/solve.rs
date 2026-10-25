//! Turning a move's rotations into each bone's channels, as make_demo's
//! `solve` does: each bone's rotation is its parent's composed with its own
//! delta, laid over its rest orientation at its posed head; its channels
//! (the basis) are what take its rest frame there under its posed parent.
//! Then the body is lifted so its lowest foot point — a toe's tip, the ball
//! of a foot, a heel — stands on the sole, `air` higher when it jumps.

use super::{BDeltas, BSkeleton};
use glam::{Mat4, Quat, Vec3};

/// The sole under the foot's lowest point, metres.
pub const SOLE: f32 = 0.022;

/// A bone's channels: its rotation from rest and, for the root, the lift.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BChannel {
    pub rotation: Quat,
    pub location: Vec3,
}

/// The solved pose: every bone's channels, and every bone's posed
/// armature-space matrix (for where a hand is).
#[derive(Clone, Debug, PartialEq)]
pub struct BSolved {
    pub channels: Vec<BChannel>,
    pub posed: Vec<Mat4>,
}

/// The points that stand on the floor at rest: (bone, at its tail?).
const CONTACT: [(&str, bool); 6] = [("DEF-toe.L", true), ("DEF-toe.R", true), ("DEF-toe.L", false), ("DEF-toe.R", false), ("DEF-foot.L", false), ("DEF-foot.R", false)];

pub fn solve(skeleton: &BSkeleton, deltas: &BDeltas, air: f32) -> BSolved {
    let n = skeleton.bones.len();
    let mut absolute = vec![Quat::IDENTITY; n];
    let mut posed = vec![Mat4::IDENTITY; n];
    let mut basis = vec![Mat4::IDENTITY; n];
    for (i, bone) in skeleton.bones.iter().enumerate() {
        let delta = deltas.get(&bone.name);
        absolute[i] = match (bone.parent, delta) {
            (Some(p), Some(d)) => absolute[p] * d,
            (Some(p), None) => absolute[p],
            (None, d) => d.unwrap_or(Quat::IDENTITY),
        };
        let rest_rot = Mat4::from_quat(Quat::from_mat4(&bone.rest));
        let head = match bone.parent {
            Some(p) => posed[p].transform_point3(skeleton.bones[p].rest.inverse().transform_point3(bone.rest.w_axis.truncate())),
            None => bone.rest.w_axis.truncate(),
        };
        let target = Mat4::from_translation(head) * Mat4::from_quat(absolute[i]) * rest_rot;
        posed[i] = target;
        let rel = match bone.parent {
            Some(p) => skeleton.bones[p].rest.inverse() * bone.rest,
            None => bone.rest,
        };
        let parent = bone.parent.map_or(Mat4::IDENTITY, |p| posed[p]);
        basis[i] = rel.inverse() * parent.inverse() * target;
    }
    // Stand the lowest contact point on the sole.
    let lowest = CONTACT
        .iter()
        .filter_map(|(name, tip)| {
            let i = skeleton.index(name)?;
            let rest = &skeleton.bones[i];
            let at_rest = if *tip { rest.rest.transform_point3(Vec3::Y * rest.length).z } else { rest.rest.w_axis.z };
            let now = if *tip { posed[i].transform_point3(Vec3::Y * rest.length).z } else { posed[i].w_axis.z };
            Some(now - at_rest)
        })
        .fold(f32::MAX, f32::min);
    // A skeleton without feet stands as it is.
    let lowest = if lowest == f32::MAX { 0.0 } else { lowest };
    let lift = air - lowest + SOLE;
    let shift = Mat4::from_translation(Vec3::Z * lift);
    if let Some(root) = skeleton.bones.iter().position(|b| b.parent.is_none()) {
        let rel = skeleton.bones[root].rest;
        basis[root] = rel.inverse() * shift * posed[root];
    }
    for p in &mut posed {
        *p = shift * *p;
    }
    let channels = basis
        .iter()
        .map(|m| {
            let (_, rotation, location) = m.to_scale_rotation_translation();
            BChannel { rotation: rotation.normalize(), location }
        })
        .collect();
    BSolved { channels, posed }
}

#[cfg(test)]
mod tests {
    use super::super::BSkelBone;
    use super::*;

    /// A leg: hip at 1 m, knee at 0.5, ankle on the floor, a toe forward.
    fn leg() -> BSkeleton {
        // A bone points along its local +y: each is built from its head and
        // the direction it points.
        let bone = |name: &str, parent: Option<usize>, head: Vec3, dir: Vec3, length: f32| {
            let rot = Quat::from_rotation_arc(Vec3::Y, dir);
            BSkelBone { name: name.into(), parent, rest: Mat4::from_rotation_translation(rot, head), length }
        };
        BSkeleton {
            bones: vec![
                bone("DEF-spine", None, Vec3::new(0.0, 0.0, 1.0), Vec3::Z, 0.3),
                bone("DEF-thigh.L", Some(0), Vec3::new(0.1, 0.0, 1.0), -Vec3::Z, 0.5),
                bone("DEF-shin.L", Some(1), Vec3::new(0.1, 0.0, 0.5), -Vec3::Z, 0.45),
                bone("DEF-foot.L", Some(2), Vec3::new(0.1, 0.0, 0.05), -Vec3::Y, 0.12),
                bone("DEF-toe.L", Some(3), Vec3::new(0.1, -0.12, 0.05), -Vec3::Y, 0.05),
            ],
        }
    }

    #[test]
    fn at_rest_the_channels_are_identity_and_the_feet_stand_on_the_sole() {
        let s = leg();
        let solved = solve(&s, &BDeltas::default(), 0.0);
        for (i, c) in solved.channels.iter().enumerate().skip(1) {
            assert!(c.rotation.angle_between(Quat::IDENTITY) < 1e-4, "bone {i} turned at rest");
        }
        assert!((solved.channels[0].location.length() - SOLE).abs() < 1e-4, "the root lifted by the sole: {:?}", solved.channels[0].location);
    }

    #[test]
    fn a_bent_knee_drops_the_body_onto_the_bent_leg() {
        let s = leg();
        let mut d = BDeltas::default();
        super::super::pieces::leg(&mut d, super::super::pieces::Side::L, 0.6, 1.2, 0.0, 0.0);
        let solved = solve(&s, &d, 0.0);
        // The bent leg is shorter, so the whole body comes down until its
        // lowest point — here the toe's tip, the foot tipped with the shin —
        // stands on the sole.
        let toe = solved.posed[4].transform_point3(Vec3::Y * 0.05);
        let points = [toe.z - 0.05, solved.posed[4].w_axis.z - 0.05, solved.posed[3].w_axis.z - 0.05];
        let lowest = points.iter().copied().fold(f32::MAX, f32::min);
        assert!((lowest - SOLE).abs() < 1e-4, "the lowest point stands on the sole: {points:?}");
        assert!(solved.posed[0].w_axis.z < 1.0, "the hip came down: {}", solved.posed[0].w_axis.z);
    }
}
