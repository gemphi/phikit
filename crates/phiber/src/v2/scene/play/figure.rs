//! The figure every player is made of: the demo's first rigged player
//! (`assets/source/make_demo.py`'s Epic01 build), read back from the
//! embedded demo — her rig, her body as the kit is cut from it (phiplay's
//! `BBody`: the mesh the Armature deforms, its groups and weights, the
//! bones' rest heads and tails) and her skeleton as the poses are solved on
//! it (`BSkeleton`).
//!
//! The demo parents each body to its rig with nothing between them, so the
//! body's own space is the rig's: the vertices and the bones' armature-space
//! rest matrices are in one frame, and a garment cut from one fits the other.

use crate::phiphx::scene::SceneModel;
use phiplay::build::body::{BBody, BBone};
use phiplay::pose::{BSkelBone, BSkeleton};
use std::sync::Arc;

/// The meshes adopted with the figure: the body and its face. Her hair,
/// kit and shoes are the match's own (`phiplay::build::dress`).
pub const KEEP: [&str; 6] = ["Body", "Eye.L", "Eye.R", "Eyebrows", "Eyelashes", "Nose"];

pub struct BFigure {
    /// The rig's index in the demo's `motion.rigs.rigs`.
    pub rig: usize,
    /// The rig's name less " Rig" — the prefix its meshes carry.
    pub name: String,
    pub body: BBody,
    pub skeleton: Arc<BSkeleton>,
}

impl BFigure {
    /// The first rig in `demo` that deforms a body.
    pub fn of(demo: &SceneModel) -> Option<BFigure> {
        let rigs = &demo.motion.rigs;
        let (rig, skin) = rigs.skins.iter().find_map(|s| demo.objects.get(s.object).filter(|o| o.name.ends_with(" Body")).map(|_| (s.rig, s.clone())))?;
        let source = rigs.rigs.get(rig)?;
        let name = demo.motion.motion.list[source.object].name.strip_suffix(" Rig")?.to_string();
        let base = &skin.source.base;
        let bones: Vec<BBone> = source
            .rig
            .bones
            .iter()
            .map(|b| {
                let head = b.arm.w_axis.truncate();
                let tail = b.arm.transform_point3(glam::Vec3::Y * b.length);
                BBone { name: b.name.clone(), head: head.into(), tail: tail.into() }
            })
            .collect();
        let body = BBody {
            positions: base.vertices.clone(),
            faces: base.faces.iter().map(<[u32]>::to_vec).collect(),
            groups: skin.source.groups.clone(),
            weights: base.dverts.clone(),
            bones,
        };
        let skeleton = BSkeleton { bones: source.rig.bones.iter().map(|b| BSkelBone { name: b.name.clone(), parent: b.parent, rest: b.arm, length: b.length }).collect() };
        Some(BFigure { rig, name, body, skeleton: Arc::new(skeleton) })
    }

    /// Whether a mesh of the figure's (by its name in the demo) comes with
    /// her into the match.
    pub fn keeps(&self, mesh: &str) -> bool {
        mesh.strip_prefix(&self.name).is_some_and(|rest| KEEP.contains(&rest.trim_start()))
    }
}
