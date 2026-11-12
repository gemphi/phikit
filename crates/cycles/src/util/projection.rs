// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/projection.h` (`ProjectionTransform`,
//! `projection_identity`, `operator*`, `projection_perspective`,
//! `projection_orthographic`, `projection_inverse`, `transform_perspective`,
//! `transform_perspective_direction`, `polar_to_cartesian`) and `util/projection_inverse.h`
//! (`projection_inverse_impl`, adapted there from Industrial Light & Magic's
//! Imath under the BSD 3-clause licence).

use std::ops::Mul;

use super::Transform;

/// `ProjectionTransform`: a full 4 by 4 matrix, which a perspective needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectionTransform {
    pub x: [f32; 4],
    pub y: [f32; 4],
    pub z: [f32; 4],
    pub w: [f32; 4],
}

impl Default for ProjectionTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// `ProjectionTransform(Transform)`.
impl From<Transform> for ProjectionTransform {
    fn from(transform: Transform) -> Self {
        Self { x: transform.x, y: transform.y, z: transform.z, w: [0.0, 0.0, 0.0, 1.0] }
    }
}

impl ProjectionTransform {
    /// `projection_identity`.
    pub const IDENTITY: ProjectionTransform = ProjectionTransform { x: [1.0, 0.0, 0.0, 0.0], y: [0.0, 1.0, 0.0, 0.0], z: [0.0, 0.0, 1.0, 0.0], w: [0.0, 0.0, 0.0, 1.0] };

    fn row(&self) -> [[f32; 4]; 4] {
        [self.x, self.y, self.z, self.w]
    }

    fn of_row(row: [[f32; 4]; 4]) -> ProjectionTransform {
        ProjectionTransform { x: row[0], y: row[1], z: row[2], w: row[3] }
    }

    /// `projection_perspective`: camera space to the screen, for a field of
    /// view in radians and the near and far clip distances.
    pub fn perspective(fov: f32, near: f32, far: f32) -> ProjectionTransform {
        let depth = [0.0, 0.0, far / (far - near), -far * near / (far - near)];
        let perspective = ProjectionTransform { z: depth, w: [0.0, 0.0, 1.0, 0.0], ..ProjectionTransform::IDENTITY };
        let inv_angle = 1.0 / (0.5 * fov).tan();
        Transform::scale(inv_angle, inv_angle, 1.0) * perspective
    }

    /// `projection_orthographic`.
    pub fn orthographic(near: f32, far: f32) -> ProjectionTransform {
        Transform::scale(1.0, 1.0, 1.0 / (far - near)).into()
    }

    /// `projection_inverse`: by elimination with the largest pivot. The
    /// identity for a matrix that has no inverse.
    pub fn inverse(&self) -> ProjectionTransform {
        let (mut m, mut r) = (self.row(), ProjectionTransform::IDENTITY.row());
        for i in 0..4 {
            // The row with the largest entry in this column; the first of
            // equals, as in Cycles.
            let pivot = (i..4).fold(i, |best, row| if m[row][i].abs() > m[best][i].abs() { row } else { best });
            if m[pivot][i] == 0.0 {
                return ProjectionTransform::IDENTITY;
            }
            m.swap(i, pivot);
            r.swap(i, pivot);
            for j in i + 1..4 {
                let factor = m[j][i] / m[i][i];
                for k in 0..4 {
                    m[j][k] -= factor * m[i][k];
                    r[j][k] -= factor * r[i][k];
                }
            }
        }
        for i in (0..4).rev() {
            let scale = m[i][i];
            if scale == 0.0 {
                return ProjectionTransform::IDENTITY;
            }
            for k in 0..4 {
                m[i][k] /= scale;
                r[i][k] /= scale;
            }
            for j in 0..i {
                let factor = m[j][i];
                for k in 0..4 {
                    m[j][k] -= factor * m[i][k];
                    r[j][k] -= factor * r[i][k];
                }
            }
        }
        ProjectionTransform::of_row(r)
    }

    /// `transform_perspective`: the point, divided by its depth. Zero for a
    /// point the projection sends to infinity.
    pub fn point(&self, point: [f32; 3]) -> [f32; 3] {
        let along = |row: [f32; 4]| row[0] * point[0] + row[1] * point[1] + row[2] * point[2] + row[3];
        let w = along(self.w);
        match w != 0.0 {
            true => [along(self.x) / w, along(self.y) / w, along(self.z) / w],
            false => [0.0; 3],
        }
    }

    /// `transform_perspective_direction`: by the upper 3 by 3 alone.
    pub fn direction(&self, direction: [f32; 3]) -> [f32; 3] {
        let along = |row: [f32; 4]| row[0] * direction[0] + row[1] * direction[1] + row[2] * direction[2];
        [along(self.x), along(self.y), along(self.z)]
    }

    /// The sixteen floats, row by row, as a device takes them.
    pub fn word(&self) -> [f32; 16] {
        let mut word = [0.0; 16];
        self.row().iter().enumerate().for_each(|(index, row)| word[index * 4..index * 4 + 4].copy_from_slice(row));
        word
    }
}

/// `operator*`: `a * b` does `b` first, then `a`.
impl Mul for ProjectionTransform {
    type Output = ProjectionTransform;

    fn mul(self, b: ProjectionTransform) -> ProjectionTransform {
        let (a, b) = (self.row(), b.row());
        let cell = |i: usize, k: usize| (0..4).map(|j| a[i][j] * b[j][k]).sum::<f32>();
        ProjectionTransform::of_row([0, 1, 2, 3].map(|i| [0, 1, 2, 3].map(|k| cell(i, k))))
    }
}

impl Mul<Transform> for ProjectionTransform {
    type Output = ProjectionTransform;

    fn mul(self, b: Transform) -> ProjectionTransform {
        self * ProjectionTransform::from(b)
    }
}

impl Mul<ProjectionTransform> for Transform {
    type Output = ProjectionTransform;

    fn mul(self, b: ProjectionTransform) -> ProjectionTransform {
        ProjectionTransform::from(self) * b
    }
}

/// `polar_to_cartesian`: the point at `r` from the middle, turned by `phi`.
pub fn polar_to_cartesian(r: f32, phi: f32) -> [f32; 2] {
    [r * phi.cos(), r * phi.sin()]
}
