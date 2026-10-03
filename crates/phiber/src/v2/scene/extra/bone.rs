//! The rigs, as the viewport draws them in Object mode: each shown bone's
//! octahedron, stick or custom shape (phiphx `rig::display`), as overlay
//! lines in the armature object's wire colour — the object's selection
//! colour when it is selected, as Blender's `const_color` has it
//! (`overlay_armature.hh`).
//!
//! The bones stand where the poser has them at the shown frame
//! (`scene::pose`), else as the file was saved; the armature object where
//! the viewport draws it, the session's edits in (`source::scene_objects`).

use super::{BExtraKind, BExtraLine};
use crate::phiphx::scene::SceneModel;
use crate::v2::scene::edit::BSceneEdit;
use crate::v2::scene::BObject3D;
use crate::v2::scene::pose::BPosed;
use crate::v2::state::BState;
use glam::Mat4;

/// Every shown bone of every rig as lines, by the armature object.
pub(super) fn push(out: &mut Vec<BExtraLine>, s: &BState, model: &SceneModel) {
    let rigs = &model.motion.rigs;
    if rigs.rigs.iter().all(|r| r.look.is_none()) {
        return;
    }
    let posed = BPosed::shown(s);
    let objects = crate::v2::scene::source::scene_objects(s);
    let edit = BSceneEdit::now(s);
    for (i, rig) in rigs.rigs.iter().enumerate() {
        let (Some(look), Some(k)) = (rig.look.as_ref(), model.motion.object.iter().position(|&b| b == rig.object)) else { continue };
        let Some(bones) = posed.bones_of(model, i) else { continue };
        let file = &model.objects[k].name;
        let name = edit.renamed.iter().find(|(_, old)| old == file).map_or(file.as_str(), |(new, _)| new.as_str());
        let world = objects.iter().find(|o| o.name() == name).and_then(|o| match o {
            BObject3D::Loaded { matrix_world, .. } => Some(*matrix_world),
            BObject3D::Primitive { .. } => None,
        });
        let Some(world) = world else { continue };
        let world = Mat4::from_cols_array_2d(&world);
        let origin = world.w_axis.truncate().to_array();
        out.extend(look.lines(&rig.rig, bones, world).into_iter().map(|[a, b]| BExtraLine { kind: BExtraKind::Bone, a: a.to_array(), b: b.to_array(), name_index: k, origin }));
    }
}
