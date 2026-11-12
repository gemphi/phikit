// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/transform.h` and `transform.cpp`
//! (`Transform`, `transform_identity`, `transform_scale`,
//! `transform_translate`, `operator*`, `transform_inverse_impl`,
//! `transform_point`, `transform_direction`, `transform_from_viewplane`,
//! `transform_empty`, `transform_get_column`, `transform_set_column`,
//! `transform_negative_scale`, `transform_uniform_scale`,
//! `transform_transposed_inverse`), and `util/boundbox.h` (`BoundBox2D`).

use std::ops::Mul;

use super::vector::{cross, dot};

/// `BoundBox2D`: a rectangle, such as the part of the screen a camera sees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundBox2D {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl Default for BoundBox2D {
    fn default() -> Self {
        Self { left: 0.0, right: 1.0, bottom: 0.0, top: 1.0 }
    }
}

impl BoundBox2D {
    /// `BoundBox2D(radius)`: the rectangle that reaches `radius` each way
    /// from the origin.
    pub fn of_radius(radius: [f32; 2]) -> BoundBox2D {
        BoundBox2D { left: -radius[0], right: radius[0], bottom: -radius[1], top: radius[1] }
    }
}

/// `Transform`: three rows of a 4 by 4 matrix whose last row is 0 0 0 1.
/// A point is turned and scaled by the first three columns and moved by
/// the fourth.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub x: [f32; 4],
    pub y: [f32; 4],
    pub z: [f32; 4],
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

fn dot3(row: [f32; 4], point: [f32; 3]) -> f32 {
    row[0] * point[0] + row[1] * point[1] + row[2] * point[2]
}

impl Transform {
    /// `transform_identity`.
    pub const IDENTITY: Transform = Transform { x: [1.0, 0.0, 0.0, 0.0], y: [0.0, 1.0, 0.0, 0.0], z: [0.0, 0.0, 1.0, 0.0] };

    /// `transform_empty`: every float zero, which an object with no
    /// transform at a motion step is given.
    pub const EMPTY: Transform = Transform { x: [0.0; 4], y: [0.0; 4], z: [0.0; 4] };

    /// `transform_scale`.
    pub fn scale(x: f32, y: f32, z: f32) -> Transform {
        Transform { x: [x, 0.0, 0.0, 0.0], y: [0.0, y, 0.0, 0.0], z: [0.0, 0.0, z, 0.0] }
    }

    /// `transform_translate`.
    pub fn translate(x: f32, y: f32, z: f32) -> Transform {
        Transform { x: [1.0, 0.0, 0.0, x], y: [0.0, 1.0, 0.0, y], z: [0.0, 0.0, 1.0, z] }
    }

    /// `transform_from_viewplane`: the rectangle to the square from 0 to 1.
    pub fn from_viewplane(viewplane: &BoundBox2D) -> Transform {
        let scale = Transform::scale(1.0 / (viewplane.right - viewplane.left), 1.0 / (viewplane.top - viewplane.bottom), 1.0);
        scale * Transform::translate(-viewplane.left, -viewplane.bottom, 0.0)
    }

    /// `transform_point`.
    pub fn point(&self, point: [f32; 3]) -> [f32; 3] {
        [dot3(self.x, point) + self.x[3], dot3(self.y, point) + self.y[3], dot3(self.z, point) + self.z[3]]
    }

    /// `transform_direction`: turned and scaled, not moved.
    pub fn direction(&self, direction: [f32; 3]) -> [f32; 3] {
        [dot3(self.x, direction), dot3(self.y, direction), dot3(self.z, direction)]
    }

    /// `transform_inverse_impl`: the same steps as Embree's, so instances
    /// are intersected consistently. A matrix that cannot be inverted (a
    /// scale of zero on an axis) is nudged first, as in Cycles.
    pub fn inverse(&self) -> Transform {
        let mut x = [self.x[0], self.y[0], self.z[0]];
        let mut y = [self.x[1], self.y[1], self.z[1]];
        let mut z = [self.x[2], self.y[2], self.z[2]];
        let w = [self.x[3], self.y[3], self.z[3]];
        let mut det = dot(x, cross(y, z));
        if det == 0.0 {
            x[0] += 1e-8;
            y[1] += 1e-8;
            z[2] += 1e-8;
            det = dot(x, cross(y, z));
        }
        if det == 0.0 {
            det = f32::MAX;
        }
        let row = |a: [f32; 3], b: [f32; 3]| {
            let inverse = cross(a, b).map(|part| part / det);
            [inverse[0], inverse[1], inverse[2], -dot(inverse, w)]
        };
        Transform { x: row(y, z), y: row(z, x), z: row(x, y) }
    }

    /// `transform_get_column`.
    pub fn column(&self, column: usize) -> [f32; 3] {
        [self.x[column], self.y[column], self.z[column]]
    }

    /// `transform_set_column`.
    pub fn set_column(&mut self, column: usize, value: [f32; 3]) {
        (self.x[column], self.y[column], self.z[column]) = (value[0], value[1], value[2]);
    }

    /// `transform_negative_scale`: it mirrors, so a triangle's corners
    /// come out the other way round.
    pub fn negative_scale(&self) -> bool {
        dot(cross(self.column(0), self.column(1)), self.column(2)) < 0.0
    }

    /// `transform_uniform_scale`: the square of the scale, when it is
    /// the same along every axis.
    pub fn uniform_scale(&self) -> Option<f32> {
        let square = |part: [f32; 3]| dot(part, part);
        let sx = square([self.x[0], self.x[1], self.x[2]]);
        let other = [square([self.y[0], self.y[1], self.y[2]]), square([self.z[0], self.z[1], self.z[2]]), square(self.column(0)), square(self.column(1)), square(self.column(2))];
        other.iter().all(|part| (sx - part).abs() < 1e-6).then_some(sx)
    }

    /// `transform_transposed_inverse`: what turns a normal when the
    /// transform itself turns the surface.
    pub fn transposed_inverse(&self) -> Transform {
        let inverse = self.inverse();
        let row = |column: usize| [inverse.x[column], inverse.y[column], inverse.z[column], 0.0];
        Transform { x: row(0), y: row(1), z: row(2) }
    }

    /// The twelve floats, row by row, as a device takes them.
    pub fn word(&self) -> [f32; 12] {
        let mut word = [0.0; 12];
        word[..4].copy_from_slice(&self.x);
        word[4..8].copy_from_slice(&self.y);
        word[8..].copy_from_slice(&self.z);
        word
    }
}

/// `operator*`: `a * b` does `b` first, then `a`.
impl Mul for Transform {
    type Output = Transform;

    fn mul(self, b: Transform) -> Transform {
        let row = |a: [f32; 4]| {
            let column = |index: usize| a[0] * b.x[index] + a[1] * b.y[index] + a[2] * b.z[index];
            [column(0), column(1), column(2), column(3) + a[3]]
        };
        Transform { x: row(self.x), y: row(self.y), z: row(self.z) }
    }
}

impl Transform {
    /// `transform_clear_scale`: each of the three axes made unit length.
    pub fn clear_scale(&self) -> Transform {
        let mut result = *self;
        for column in 0..3 {
            result.set_column(column, super::vector::normalize(self.column(column)));
        }
        result
    }
}

#[cfg(test)]
mod test;
