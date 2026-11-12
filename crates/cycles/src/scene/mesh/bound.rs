// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/mesh.cpp` (`Mesh::compute_bounds`,
//! `Mesh::apply_transform`): the box a mesh fits in, and an object's
//! transform put into the mesh itself.

use super::Mesh;
use crate::scene::{Attribute, AttributeStandard};
use crate::util::vector::normalize;
use crate::util::{BoundBox, PackedNormal, Transform};

/// The steps of a moving attribute after the centre.
fn motion_step(attribute: &Attribute) -> std::ops::Range<usize> {
    1..attribute.motion_step_count()
}

impl Mesh {
    /// `compute_bounds`: the box around every vertex, at every motion
    /// step when the mesh is blurred. Vertices that are infinite or not a
    /// number are left out only when they spoil the box; an empty mesh is
    /// a point at the origin.
    pub fn compute_bound(&mut self) {
        let blurred = self.geometry.use_motion_blur();
        let grown = |safe: bool| {
            let mut bound = BoundBox::EMPTY;
            let position = self.geometry.position_attribute();
            let step = position.map(|attribute| match blurred && attribute.has_motion() {
                true => 0..attribute.motion_step_count(),
                false => 0..1,
            });
            for step in step.unwrap_or(0..0) {
                let point = position.map(|attribute| attribute.data(step).float3()).unwrap_or(&[]);
                match safe {
                    true => point.iter().for_each(|point| bound.grow_safe(*point)),
                    false => point.iter().for_each(|point| bound.grow(*point)),
                }
            }
            bound
        };
        let mut bound = grown(false);
        if !bound.valid() && self.vert_count() > 0 {
            bound = grown(true);
        }
        if !bound.valid() {
            bound.grow([0.0; 3]);
        }
        self.geometry.bound = bound;
    }

    /// `apply_transform`: move the vertices by `transform` and turn the
    /// normals to match; the motion steps too, with `apply_to_motion`.
    pub fn apply_transform(&mut self, transform: &Transform, apply_to_motion: bool) {
        let turn = transform.transposed_inverse();
        self.geometry.transform_normal = turn;
        let turned = |normal: &mut PackedNormal| *normal = PackedNormal::encode(normalize(turn.direction(normal.decode())));
        let moved = |point: &mut [f32; 3]| *point = transform.point(*point);

        self.geometry.position_mut().iter_mut().for_each(moved);
        let attribute = &mut self.geometry.attribute;
        for std in ["VERTEX_NORMAL", "CORNER_NORMAL", "NORMAL_UNDISPLACED"].map(AttributeStandard::of) {
            let normal = attribute.find_standard_mut(std);
            normal.into_iter().for_each(|normal| normal.center.normal_mut().iter_mut().for_each(turned));
        }
        if !apply_to_motion {
            return;
        }
        if let Some(position) = attribute.find_standard_mut(AttributeStandard::of("POSITION")) {
            motion_step(position).for_each(|step| position.data_mut(step).float3_mut().iter_mut().for_each(moved));
        }
        for std in ["VERTEX_NORMAL", "CORNER_NORMAL"].map(AttributeStandard::of) {
            if let Some(normal) = attribute.find_standard_mut(std) {
                motion_step(normal).for_each(|step| normal.data_mut(step).normal_mut().iter_mut().for_each(turned));
            }
        }
    }
}

#[cfg(test)]
mod test;
