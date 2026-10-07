//! A frame's driver state, kept rather than dropped: what the objects'
//! own driver pass leaves behind, so a driver on something that is *not* an
//! object — a material, and in time a modifier — reads the same scene the
//! object drivers read, and reads it after them.
//!
//! `BlendMotionList::raw_at` runs that pass and keeps only the transforms.
//! A material driver needs the rest of it: the world matrices a Transform
//! Channel variable reads, and the custom properties the object pass drove,
//! which is the ordering Blender's depsgraph gives — custom property first,
//! then whatever reads it.

use super::pass::{BDriverProps, BDriverRun};
use super::space::BDriverWorld;
use super::var::BlendDriverScene;
use crate::blend::anim::motion::BlendMotionList;
use crate::blend::object::transform::ObjectTransformRaw;
use std::collections::HashMap;

/// One frame's scene, as the drivers left it.
#[derive(Clone, Debug, Default)]
pub struct BDriverState {
    pub frame: f32,
    /// Each object's transform after its actions, NLA and channel drivers.
    pub raw: Vec<ObjectTransformRaw>,
    /// Each object's world matrix over those transforms.
    pub world: Vec<[[f32; 4]; 4]>,
    /// Every object's custom properties as the pass drove them.
    pub props: BDriverProps,
    /// Block address → index, which a driver variable's target names.
    pub index: HashMap<u64, usize>,
}

impl BDriverState {
    /// What a driver reads. Borrows the state, so one frame's state serves
    /// every driver on it.
    pub fn scene(&self) -> BlendDriverScene<'_> {
        BlendDriverScene { raw: &self.raw, world: &self.world, prop: &self.props, index: &self.index }
    }
}

impl BlendMotionList {
    /// The driver state at `frame` — the objects' pass run and kept.
    pub fn driver_state_at(&self, frame: f32) -> BDriverState {
        let mut raw: Vec<ObjectTransformRaw> = self.list.iter().map(|m| m.raw_at(frame)).collect();
        let index: HashMap<u64, usize> =
            self.list.iter().enumerate().filter(|(_, m)| m.address != 0).map(|(i, m)| (m.address, i)).collect();
        let sets: Vec<_> = self.list.iter().map(|m| m.drivers.clone()).collect();
        let props = sets.run(&mut raw, &index, frame);
        let world = raw.driver_worlds(&index);
        BDriverState { frame, raw, world, props, index }
    }
}
