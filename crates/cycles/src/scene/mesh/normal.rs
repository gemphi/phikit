// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/mesh.cpp` (`Mesh::add_vertex_normals`,
//! `Mesh::SubdFace::normal`): a normal for each vertex, the sum of the
//! normals of the triangles around it, when the mesh came without.

use super::{Mesh, SubdFace};
use crate::scene::{Attribute, AttributeData, AttributeStandard};
use crate::util::PackedNormal;
use crate::util::vector::{add, cross, len, normalize, scale, sub};

impl Mesh {
    /// The sums as packed normals, turned round on a mesh that was
    /// mirrored. A vertex no triangle uses has a normal that is not a
    /// number, which packs as zero, as in Cycles.
    fn packed(&self, sum: Vec<[f32; 3]>) -> Vec<PackedNormal> {
        let sign = match self.geometry.transform_negative_scaled {
            true => -1.0,
            false => 1.0,
        };
        sum.into_iter().map(|normal| PackedNormal::encode(scale(normalize(normal), sign))).collect()
    }

    /// A normal for each vertex at the positions `point`.
    fn vertex_normal(&self, point: &[[f32; 3]]) -> Vec<PackedNormal> {
        let mut sum = vec![[0.0f32; 3]; self.vert_count()];
        for index in 0..self.triangle_count() {
            let triangle = self.triangle(index);
            let normal = triangle.compute_normal(point);
            for vert in triangle.v {
                let slot = usize::try_from(vert).ok().and_then(|vert| sum.get_mut(vert));
                slot.into_iter().for_each(|slot| *slot = add(*slot, normal));
            }
        }
        self.packed(sum)
    }

    /// `SubdFace::normal`: the way a face to subdivide faces, from its
    /// first three corners.
    pub fn subd_face_normal(&self, face: &SubdFace) -> [f32; 3] {
        let position = self.subd_attribute.find_standard(AttributeStandard::of("POSITION"));
        let point = position.map(|attribute| attribute.center.float3()).unwrap_or(&[]);
        let corner = self.geometry.node.int_array("subd_face_corners");
        let at = |index: usize| {
            let vert = corner.get(face.start_corner as usize + index).copied().unwrap_or(0);
            point.get(vert as usize).copied().unwrap_or_default()
        };
        let normal = cross(sub(at(1), at(0)), sub(at(2), at(0)));
        match len(normal) != 0.0 {
            true => normalize(normal),
            false => normal,
        }
    }

    /// A normal for each vertex of the faces to subdivide.
    fn subd_vertex_normal(&self) -> Vec<PackedNormal> {
        let mut sum = vec![[0.0f32; 3]; self.size().subd_base_vert];
        let corner = self.geometry.node.int_array("subd_face_corners");
        for index in 0..self.subd_face_count() {
            let face = self.subd_face(index);
            let normal = self.subd_face_normal(&face);
            for vert in (0..face.corner_count.max(0) as usize).filter_map(|at| corner.get(face.start_corner as usize + at)) {
                let slot = usize::try_from(*vert).ok().and_then(|vert| sum.get_mut(vert));
                slot.into_iter().for_each(|slot| *slot = add(*slot, normal));
            }
        }
        self.packed(sum)
    }

    /// `add_vertex_normals`. Corner normals take their place, unless the
    /// mesh is blurred and they do not move: those cannot be drawn right
    /// and are dropped.
    pub fn add_vertex_normal(&mut self) {
        let (corner, vertex) = (AttributeStandard::of("CORNER_NORMAL"), AttributeStandard::of("VERTEX_NORMAL"));
        let blurred = self.has_motion_blur();
        match self.geometry.attribute.find_standard(corner).map(Attribute::has_motion) {
            Some(false) if blurred => self.geometry.attribute.remove_standard(corner),
            Some(_) => return,
            None => {}
        }
        let has_triangle = self.triangle_count() > 0;
        if self.geometry.attribute.find_standard(vertex).is_none() && has_triangle {
            let normal = self.vertex_normal(self.geometry.position());
            self.add_attribute(vertex).center = AttributeData::Normal(normal);
        }

        // The motion steps, when the positions move and the normals do not yet.
        let position = self.geometry.position_attribute().filter(|attribute| attribute.has_motion());
        let moving = self.geometry.attribute.find_standard(vertex).is_some_and(Attribute::has_motion);
        if let Some(position) = position.filter(|_| blurred && !moving && has_triangle) {
            let step: Vec<Vec<PackedNormal>> = (1..position.motion_step_count()).map(|step| self.vertex_normal(position.data(step).float3())).collect();
            let motion_step_count = self.geometry.motion_step_count();
            let attribute = self.add_attribute(vertex);
            attribute.add_motion(motion_step_count);
            step.into_iter().enumerate().for_each(|(index, normal)| *attribute.data_mut(index + 1) = AttributeData::Normal(normal));
        }

        if self.subd_attribute.find_standard(vertex).is_none() && self.subd_face_count() > 0 {
            let normal = self.subd_vertex_normal();
            self.add_subd_attribute(vertex).center = AttributeData::Normal(normal);
        }
    }
}

#[cfg(test)]
mod test;
