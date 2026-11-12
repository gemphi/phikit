// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/mesh.h` and `mesh.cpp`
//! (`Mesh::Triangle`): one triangle, as the three vertices it joins.

use crate::scene::Attribute;
use crate::util::BoundBox;
use crate::util::vector::{cross, interp, is_finite, len, scale, sub};

/// `Mesh::Triangle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle {
    /// The three vertices, by their place in the mesh.
    pub v: [i32; 3],
}

impl Triangle {
    /// Its corners out of `point`. A vertex the mesh does not have is at
    /// the origin.
    pub fn corner(&self, point: &[[f32; 3]]) -> [[f32; 3]; 3] {
        self.v.map(|vert| usize::try_from(vert).ok().and_then(|vert| point.get(vert)).copied().unwrap_or_default())
    }

    /// `bounds_grow`.
    pub fn bound_grow(&self, point: &[[f32; 3]], bound: &mut BoundBox) {
        self.corner(point).into_iter().for_each(|point| bound.grow(point));
    }

    /// `compute_normal`: the way it faces. A triangle with no area faces
    /// along x.
    pub fn compute_normal(&self, point: &[[f32; 3]]) -> [f32; 3] {
        let [v0, v1, v2] = self.corner(point);
        let normal = cross(sub(v1, v0), sub(v2, v0));
        let length = len(normal);
        match length == 0.0 {
            true => [1.0, 0.0, 0.0],
            false => scale(normal, 1.0 / length),
        }
    }

    /// `valid`: no corner is infinite or not a number.
    pub fn valid(&self, point: &[[f32; 3]]) -> bool {
        self.corner(point).into_iter().all(is_finite)
    }

    /// `verts_for_step`: its corners at a motion step, counted in the
    /// order of time.
    pub fn vert_for_step(&self, position: &Attribute, step: usize) -> [[f32; 3]; 3] {
        self.corner(position.data_at_time_step(step, position.motion_step_count()).float3())
    }

    /// `motion_verts`: its corners at `time`, from 0 at the first of
    /// `step_count` steps to 1 at the last.
    pub fn motion_vert(&self, position: &Attribute, step_count: usize, time: f32) -> [[f32; 3]; 3] {
        let last = step_count.saturating_sub(1);
        let step = ((time * last as f32) as usize).min(last.saturating_sub(1));
        let t = time * last as f32 - step as f32;
        let (current, next) = (self.vert_for_step(position, step), self.vert_for_step(position, step + 1));
        [0, 1, 2].map(|corner| interp(current[corner], next[corner], t))
    }
}

#[cfg(test)]
mod test;
