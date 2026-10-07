//! A frame's driver pass over every object: the custom-property drivers
//! first, then the transform-channel drivers over what they wrote.
//!
//! Blender's depsgraph puts a driven property before the driver that reads
//! it; this is that order for the one chain a file leans on in practice —
//! a custom property on the rig object (or any object), read by a driver
//! on something else. Within each step the values come from a snapshot, so
//! the order objects are visited in cannot change the answer.

use super::var::BlendDriverScene;
use super::space::BDriverWorld;
use super::{BlendDriver, BlendObjectDriver};
use crate::blend::idprop::{BIdGroup, BIdValue};
use crate::blend::object::transform::ObjectTransformRaw;
use crate::blend::rig::solve::BChannelValue;
use std::collections::HashMap;

/// One driver on an object's own custom property (`["name"]`).
#[derive(Clone, Debug, PartialEq)]
pub struct BlendPropDriver {
    pub name: String,
    pub driver: BlendDriver,
}

/// Everything one object brings to the pass.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendDriverSet {
    /// Its block address, which a variable's target names.
    pub address: u64,
    /// Drivers on its own transform channels.
    pub channels: Vec<BlendObjectDriver>,
    /// Drivers on its own custom properties.
    pub props: Vec<BlendPropDriver>,
    /// Its custom properties as saved — what a property with no driver
    /// reads, and what a driven one starts from.
    pub saved: BIdGroup,
}

impl BlendDriverSet {
    /// Has this object anything for the pass to do?
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty() && self.props.is_empty()
    }
}

/// What a frame's pass leaves behind: every object's custom properties as
/// the drivers left them, by block address, for a caller that shows or
/// writes them.
pub type BDriverProps = HashMap<u64, BIdGroup>;

/// A frame's drivers written over a snapshot of the scene.
pub trait BDriverRun {
    /// Run every object's drivers at `frame`, writing the channel drivers
    /// into `raw` and returning the custom properties as driven.
    fn run(&self, raw: &mut [ObjectTransformRaw], index: &HashMap<u64, usize>, frame: f32) -> BDriverProps;
}

impl BDriverRun for [BlendDriverSet] {
    fn run(&self, raw: &mut [ObjectTransformRaw], index: &HashMap<u64, usize>, frame: f32) -> BDriverProps {
        let mut props: BDriverProps = self.iter().filter(|s| s.address != 0).map(|s| (s.address, s.saved.clone())).collect();
        if self.iter().all(BlendDriverSet::is_empty) {
            return props;
        }
        // The values are read from a snapshot of the actions' own result,
        // so the order objects are driven in cannot change the answer.
        let before = raw.to_vec();
        let world = before.driver_worlds(index);
        let driven: Vec<(u64, Vec<(String, f32)>)> =
            self.iter().map(|s| (s.address, s.driven_props(&before, &world, &props, index, frame))).collect();
        for (address, values) in driven {
            if let Some(group) = props.get_mut(&address) {
                for (name, value) in values {
                    group.set(&name, BIdValue::Float(value as f64));
                }
            }
        }
        let scene = BlendDriverScene { raw: &before, world: &world, prop: &props, index };
        for (i, set) in self.iter().enumerate() {
            for d in set.channels.iter().filter(|d| !d.driver.muted()) {
                let v = d.driver.value(&scene, frame);
                if let Some(slot) = raw.get_mut(i) {
                    slot.set(&d.property, d.index, v);
                }
            }
        }
        props
    }
}

impl BlendDriverSet {
    /// This object's custom-property drivers at `frame`, read against the
    /// properties as saved.
    fn driven_props(
        &self,
        before: &[ObjectTransformRaw],
        world: &[[[f32; 4]; 4]],
        props: &BDriverProps,
        index: &HashMap<u64, usize>,
        frame: f32,
    ) -> Vec<(String, f32)> {
        if self.props.is_empty() {
            return Vec::new();
        }
        let scene = BlendDriverScene { raw: before, world, prop: props, index };
        self.props.iter().filter(|d| !d.driver.muted()).map(|d| (d.name.clone(), d.driver.value(&scene, frame))).collect()
    }
}
