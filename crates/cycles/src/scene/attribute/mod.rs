// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/attribute.h` and `attribute.cpp`
//! (`Attribute`): a layer of values on a geometry, one for each vertex,
//! corner or face, such as its positions, its normals or a UV map. An
//! attribute may hold its values again for each step of a motion.
//!
//! Not ported: a voxel attribute (its value is an image, which waits for
//! the image manager), and values shared with the host application
//! (`add_shared`, `set_motion_step_shared`): an attribute owns its values.

mod data;
mod request;
mod set;
mod standard;

pub use data::AttributeData;
pub use request::{AttributeDescriptor, AttributeRequest, AttributeRequestSet};
pub use set::AttributeSet;
pub use standard::{AttributeElement, AttributePrimitive, AttributeStandard, AttributeType, KernelDataType};

use std::collections::BTreeSet;

/// `Attribute`.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub std: AttributeStandard,
    /// `type`.
    pub kind: AttributeType,
    pub element: AttributeElement,
    /// The values at the centre of the motion, or the only ones.
    pub center: AttributeData,
    /// The other motion steps; empty when the attribute does not move.
    pub motion: Vec<AttributeData>,
    pub flag: u32,
    /// Its values changed since the device took them.
    pub modified: bool,
}

impl Attribute {
    /// `Attribute(name, type, element, geom, prim)`, with the number of
    /// elements the geometry has worked out by the caller.
    pub fn new(name: &str, kind: AttributeType, element: AttributeElement, size: usize) -> Attribute {
        let mut center = AttributeData::of(kind, element);
        center.resize(size);
        Attribute { name: name.to_string(), std: AttributeStandard::NONE, kind, element, center, motion: Vec::new(), flag: 0, modified: true }
    }

    /// `size`: how many elements it has a value for.
    pub fn size(&self) -> usize {
        self.center.len()
    }

    /// `resize`: every step to `count` values.
    pub fn resize(&mut self, count: usize) {
        self.center.resize(count);
        self.motion.iter_mut().for_each(|step| step.resize(count));
    }

    /// `add_motion`: room for the steps of a geometry with `motion_step_count`
    /// in all, the centre included. Zero steps leaves it as it is.
    pub fn add_motion(&mut self, motion_step_count: u32) {
        let wanted = (motion_step_count as usize).saturating_sub(1);
        match motion_step_count == 0 || wanted == self.motion.len() {
            true => {}
            false => {
                let mut step = AttributeData::of(self.kind, self.element);
                step.resize(self.size());
                self.motion.resize(wanted, step);
                self.modified = true;
            }
        }
    }

    /// `remove_motion`.
    pub fn remove_motion(&mut self) {
        self.modified |= self.has_motion();
        self.motion.clear();
    }

    pub fn has_motion(&self) -> bool {
        !self.motion.is_empty()
    }

    /// `num_motion_steps`: the steps stored, the centre included.
    pub fn motion_step_count(&self) -> usize {
        self.motion.len() + 1
    }

    /// `take_motion_from`: the motion steps of `other` become this one's.
    pub fn take_motion_from(&mut self, other: &mut Attribute) {
        debug_assert!(other.kind == self.kind && other.element == self.element && other.size() == self.size());
        self.motion = std::mem::take(&mut other.motion);
        (self.modified, other.modified) = (true, true);
    }

    /// `data(step)`: step 0 is the centre, and so is any step of an
    /// attribute that does not move.
    pub fn data(&self, step: usize) -> &AttributeData {
        debug_assert!(step <= self.motion.len() || !self.has_motion());
        match step == 0 || step > self.motion.len() {
            true => &self.center,
            false => &self.motion[step - 1],
        }
    }

    /// `data_for_write(step)`.
    pub fn data_mut(&mut self, step: usize) -> &mut AttributeData {
        debug_assert!(step <= self.motion.len());
        match step == 0 || step > self.motion.len() {
            true => &mut self.center,
            false => &mut self.motion[step - 1],
        }
    }

    /// `time_step_to_attr_step`: steps in the order of time have the
    /// centre in the middle; stored, it comes first.
    pub fn time_step_to_attr_step(step: usize, time_step_count: usize) -> usize {
        let center = time_step_count.saturating_sub(1) / 2;
        match step.cmp(&center) {
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Less => step + 1,
            std::cmp::Ordering::Greater => step,
        }
    }

    /// `data_at_time_step`.
    pub fn data_at_time_step(&self, step: usize, time_step_count: usize) -> &AttributeData {
        self.data(Attribute::time_step_to_attr_step(step, time_step_count))
    }

    /// `set_data_from`: take the values of `other`, and count as modified
    /// only when a step of them differs.
    pub fn set_data_from(&mut self, other: Attribute) {
        debug_assert!(other.std == self.std && other.kind == self.kind && other.element == self.element);
        self.flag = other.flag;
        self.modified |= self.center != other.center || self.motion != other.motion;
        self.center = other.center;
        self.motion = other.motion;
    }

    /// `Attribute::kernel_type`: the device array its values go to.
    pub fn kernel_type(&self) -> KernelDataType {
        match (self.element.has("IS_BYTE"), self.element.has("IS_NORMAL"), self.kind) {
            (true, _, _) => KernelDataType::Uchar4,
            (_, true, _) => KernelDataType::Normal,
            (_, _, AttributeType::Float) => KernelDataType::Float,
            (_, _, AttributeType::Float2) => KernelDataType::Float2,
            (_, _, AttributeType::Float4 | AttributeType::Rgba | AttributeType::Matrix) => KernelDataType::Float4,
            _ => KernelDataType::Float3,
        }
    }

    /// `get_uv_tiles`: the UDIM tiles a UV map reaches into, numbered
    /// from 1001.
    pub fn uv_tile_list(&self) -> BTreeSet<i32> {
        let tile = |uv: &[f32; 2]| {
            let (mut x, mut y) = (uv[0] as i32, uv[1] as i32);
            // A point on a tile's right or upper edge is in that tile, not the next.
            x -= i32::from(x > 0 && uv[0] < x as f32 + 1e-6);
            y -= i32::from(y > 0 && uv[1] < y as f32 + 1e-6);
            ((uv[0] as i32) >= 0 && (uv[1] as i32) >= 0 && (uv[0] as i32) < 10).then_some(1001 + 10 * y + x)
        };
        self.center.float2().iter().filter_map(tile).collect()
    }
}

#[cfg(test)]
mod test;
