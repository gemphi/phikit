//! The relationship lines (`overlay_relation.hh`): from each child to where
//! it hangs from its parent — the parent's origin, a bone's tail, a point on
//! a curve's path — and from each open constraint's target to the object it
//! constrains, where the motion puts them at the frame in scope
//! (phiphx `SceneMotion::relations_at`).
//!
//! An object the session moved (G / R / S, the Transform panel) carries its
//! end of a line with it: each end moves as its object's world did
//! (`transform::BTransforms::worlds`). A deleted object has no lines; a
//! hidden one is left out as the lines are drawn, since whether it shows
//! is not part of what these are kept for.

use super::{BExtraKind, BExtraLine};
use crate::phiphx::scene::SceneModel;
use crate::v2::scene::edit::BSceneEdit;
use crate::v2::scene::transform::BTransforms;
use crate::v2::state::BState;
use glam::{Mat4, Vec3};
use std::collections::HashMap;

/// Every relationship line at `frame`, by the object it belongs to.
pub(super) fn push(out: &mut Vec<BExtraLine>, s: &BState, model: &SceneModel, frame: Option<f32>) {
    let motion = &model.motion;
    if !motion.motion.list.iter().any(|m| m.parent.is_some() || !m.constraints.is_empty()) {
        return;
    }
    let edit = BSceneEdit::now(s);
    let moved = BRelationMove::of(s, model, frame, &edit);
    let at = frame.or(super::super::motion::scoped_frame()).unwrap_or(model.timeline.current_frame as f32);
    for line in motion.relations_at(at) {
        let name = |i: usize| motion.motion.list.get(i).map(|m| m.name.as_str());
        if [Some(line.child), line.parent].into_iter().flatten().any(|i| name(i).is_some_and(|n| edit.deleted.iter().any(|d| d == n))) {
            continue;
        }
        let (a, b) = match line.parent {
            Some(p) => (moved.carry(name(line.child), line.from), moved.carry(name(p), line.to)),
            None => (line.from, moved.carry(name(line.child), line.to)),
        };
        let kind = line.parent.map_or(BExtraKind::Constraint, |of| BExtraKind::Parent { of });
        let origin = if line.parent.is_some() { a } else { b };
        out.push(BExtraLine { kind, a, b, name_index: line.child, origin });
    }
}

/// How far each edited object moved, by its name in the file.
struct BRelationMove(HashMap<String, Mat4>);

impl BRelationMove {
    fn of(s: &BState, model: &SceneModel, frame: Option<f32>, edit: &BSceneEdit) -> Self {
        let edits = BTransforms::now(s);
        if edits.list.is_empty() {
            return Self(HashMap::new());
        }
        let before = BTransforms::posed(s, model, frame);
        let after = edits.worlds(s, model, &before);
        let file = |now: &str| edit.renamed.iter().find(|(new, _)| new == now).map_or(now.to_string(), |(_, old)| old.clone());
        Self(before.iter().filter_map(|(name, old)| after.get(name).map(|new| (file(name), *new * old.inverse()))).collect())
    }

    /// `point`, moved as the object `name` was.
    fn carry(&self, name: Option<&str>, point: [f32; 3]) -> [f32; 3] {
        match name.and_then(|n| self.0.get(n)) {
            Some(m) => m.transform_point3(Vec3::from(point)).to_array(),
            None => point,
        }
    }
}
