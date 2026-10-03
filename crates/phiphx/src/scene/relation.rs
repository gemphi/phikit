//! The viewport's relationship lines (`overlay_relation.hh`): a dashed line
//! from each child to where it hangs from its parent, and one from each open
//! constraint's target to the object it constrains — by motion index, at a
//! frame, so a line follows what moves.

use crate::blend::BMotionAt;

/// One relationship line, in world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneRelation {
    /// The object it belongs to, by index into `SceneMotion::motion.list`.
    pub child: usize,
    /// The parent it runs to, for a parent's line; `None` for a
    /// constraint's, which Blender draws in another colour.
    pub parent: Option<usize>,
    pub from: [f32; 3],
    pub to: [f32; 3],
}

impl SceneRelation {
    /// Every line of a solved frame: child to parent (`parent_display_origin`),
    /// then target to owner, as Blender appends them.
    pub fn list(at: &BMotionAt, parent: impl Fn(usize) -> Option<usize>) -> Vec<Self> {
        let place = |i: usize| at.world.get(i).map(|m| [m[3][0], m[3][1], m[3][2]]);
        let mut out = Vec::new();
        for i in 0..at.world.len() {
            let Some(from) = place(i) else { continue };
            if let (Some(p), Some(Some(to))) = (parent(i), at.parent_origin.get(i)) {
                out.push(Self { child: i, parent: Some(p), from, to: *to });
            }
            for target in at.targets.get(i).into_iter().flatten() {
                out.push(Self { child: i, parent: None, from: *target, to: from });
            }
        }
        out
    }
}
