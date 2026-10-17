//! The scene in motion: every object's transform over time and which
//! entry of `objects`, `cameras`, `lights` and `empties` moves with which
//! motion. Read by a player that wants the world matrices at a frame — the
//! 3D viewport during playback — so the still model stays what it is and a
//! frame is a lookup, never a second import.

use crate::blend::{BMotionAt, BlendMotion, BlendMotionList};

/// What moves, and who moves with it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneMotion {
    /// Every blend object's motion, in file order.
    pub motion: BlendMotionList,
    /// `objects[i]` moves with `motion.list[object[i]]`.
    pub object: Vec<usize>,
    /// `cameras[i]` moves with `motion.list[camera[i]]`.
    pub camera: Vec<usize>,
    /// `lights[i]` moves with `motion.list[light[i]]`.
    pub light: Vec<usize>,
    /// `empties[i]` moves with `motion.list[empty[i]]`.
    pub empty: Vec<usize>,
    /// The rigs that move and the meshes they deform.
    pub rigs: super::SceneRigs,
}

/// Every world matrix at one frame, ready to be read by object, camera,
/// light or empty index.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneMotionFrame<'a> {
    at: &'a SceneMotion,
    world: Vec<[[f32; 4]; 4]>,
}

impl SceneMotion {
    /// Does anything in the scene move — an object, a mesh its rig
    /// deforms, or an object following a moving rig's bone?
    pub fn animated(&self) -> bool {
        self.motion.animated() || self.rigs.animated() || (self.motion.follows() && self.rigs.moves())
    }

    /// Every object at `frame`, every rig posed.
    pub fn solved_at(&self, frame: f32) -> BMotionAt {
        self.motion.solve(frame, &self.rigs.rigs, true)
    }

    /// Each mesh a rig deforms, posed at `frame`, by object index.
    pub fn meshes_at(&self, frame: f32) -> Vec<(usize, super::SceneMesh)> {
        if !self.rigs.animated() {
            return Vec::new();
        }
        self.rigs.meshes_at(&self.solved_at(frame), &self.rigs.driven_at(&self.motion, frame))
    }

    /// Each mesh a rig deforms and each rig's bones, posed at `frame` —
    /// one solve for both.
    pub fn posed_at(&self, frame: f32) -> (Vec<(usize, super::SceneMesh)>, Vec<std::sync::Arc<Vec<glam::Mat4>>>) {
        let at = self.solved_at(frame);
        let meshes = if self.rigs.animated() {
            self.rigs.meshes_at(&at, &self.rigs.driven_at(&self.motion, frame))
        } else {
            Vec::new()
        };
        (meshes, self.rigs.bones_at(&at))
    }

    /// Each mesh a rig deforms as its Armature modifier leaves it at
    /// `frame`, by object index.
    pub fn skinned_at(&self, frame: f32) -> Vec<(usize, Vec<[f32; 3]>)> {
        self.rigs.skinned_at(&self.solved_at(frame))
    }

    /// The scene's matrices at `frame` (a rig posed only when an object
    /// follows one of its bones).
    pub fn at(&self, frame: f32) -> SceneMotionFrame<'_> {
        SceneMotionFrame { at: self, world: self.motion.solve(frame, &self.rigs.rigs, false).world }
    }

    /// The relationship lines at `frame` (`relation.rs`), a rig posed only
    /// when a line or an object needs one of its bones.
    pub fn relations_at(&self, frame: f32) -> Vec<super::SceneRelation> {
        let at = self.motion.solve(frame, &self.rigs.rigs, false);
        super::SceneRelation::list(&at, |i| self.motion.list.get(i).and_then(|m| m.parent))
    }

    /// The motion of the object named `name`.
    pub fn named(&self, name: &str) -> Option<&BlendMotion> {
        self.motion.named(name)
    }

    /// The frames anything is keyed at, first and last.
    pub fn range(&self) -> Option<(f32, f32)> {
        self.motion.list.iter().filter_map(BlendMotion::range).reduce(|(lo, hi), (a, b)| (lo.min(a), hi.max(b)))
    }
}

impl SceneMotionFrame<'_> {
    fn pick(&self, index: Option<&usize>) -> Option<[[f32; 4]; 4]> {
        index.and_then(|&i| self.world.get(i)).copied()
    }
    /// `objects[i]`'s world matrix at this frame.
    pub fn object(&self, i: usize) -> Option<[[f32; 4]; 4]> {
        self.pick(self.at.object.get(i))
    }
    /// `cameras[i]`'s world matrix at this frame.
    pub fn camera(&self, i: usize) -> Option<[[f32; 4]; 4]> {
        self.pick(self.at.camera.get(i))
    }
    /// `lights[i]`'s world matrix at this frame.
    pub fn light(&self, i: usize) -> Option<[[f32; 4]; 4]> {
        self.pick(self.at.light.get(i))
    }
    /// `empties[i]`'s world matrix at this frame.
    pub fn empty(&self, i: usize) -> Option<[[f32; 4]; 4]> {
        self.pick(self.at.empty.get(i))
    }
}
