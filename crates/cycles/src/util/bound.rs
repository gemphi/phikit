// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/boundbox.h` (`BoundBox`): the box, with
//! its sides along the axes, that a mesh or an object fits in.

use super::Transform;
use super::vector::{add, is_finite, max, min, scale, sub};

/// `BoundBox`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Default for BoundBox {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl BoundBox {
    /// `BoundBox::empty`: a box nothing is in, which the first point grown
    /// into becomes.
    pub const EMPTY: BoundBox = BoundBox { min: [f32::MAX; 3], max: [-f32::MAX; 3] };

    /// `grow` by a point. A point that is not a number leaves the box.
    pub fn grow(&mut self, point: [f32; 3]) {
        self.min = min(point, self.min);
        self.max = max(point, self.max);
    }

    /// `grow_safe`: a point with a part that is infinite or not a number
    /// is left out.
    pub fn grow_safe(&mut self, point: [f32; 3]) {
        match is_finite(point) {
            true => self.grow(point),
            false => {}
        }
    }

    /// `grow` by another box.
    pub fn grow_box(&mut self, other: &BoundBox) {
        self.min = min(other.min, self.min);
        self.max = max(other.max, self.max);
    }

    /// `valid`: it holds something, and every side is a finite number.
    pub fn valid(&self) -> bool {
        (0..3).all(|axis| self.min[axis] <= self.max[axis]) && is_finite(self.min) && is_finite(self.max)
    }

    /// `intersect`: what is left of the box inside `other`. Boxes that do
    /// not meet leave one with a side that ends before it starts.
    pub fn intersect(&mut self, other: &BoundBox) {
        self.min = max(self.min, other.min);
        self.max = min(self.max, other.max);
    }

    /// `merge`: the box around both.
    pub fn merge(&self, other: &BoundBox) -> BoundBox {
        BoundBox { min: min(self.min, other.min), max: max(self.max, other.max) }
    }

    /// `half_area`: half the area of its six faces. An empty box has an
    /// infinite one.
    pub fn half_area(&self) -> f32 {
        let side = self.size();
        side[0] * side[2] + side[1] * side[2] + side[0] * side[1]
    }

    /// `area`.
    pub fn area(&self) -> f32 {
        self.half_area() * 2.0
    }

    /// `safe_area`: the area, zero for a box with a side that ends before
    /// it starts.
    pub fn safe_area(&self) -> f32 {
        match (0..3).all(|axis| self.min[axis] <= self.max[axis]) {
            true => self.area(),
            false => 0.0,
        }
    }

    /// `center2`: twice the centre, which sorts the same and costs less.
    pub fn center2(&self) -> [f32; 3] {
        add(self.min, self.max)
    }

    /// `size`.
    pub fn size(&self) -> [f32; 3] {
        sub(self.max, self.min)
    }

    /// `center`.
    pub fn center(&self) -> [f32; 3] {
        scale(add(self.min, self.max), 0.5)
    }

    /// `transformed`: the box its eight corners fit in after `transform`.
    /// A box that is not valid stays empty.
    pub fn transformed(&self, transform: &Transform) -> BoundBox {
        let mut result = BoundBox::EMPTY;
        let corner = |index: usize, axis: usize| match index & (1 << axis) != 0 {
            true => self.min[axis],
            false => self.max[axis],
        };
        let count = match self.valid() {
            true => 8,
            false => 0,
        };
        for index in 0..count {
            result.grow(transform.point([corner(index, 0), corner(index, 1), corner(index, 2)]));
        }
        result
    }

    /// `intersects`: the two boxes touch or overlap.
    pub fn intersects(&self, other: &BoundBox) -> bool {
        let between = sub(self.center(), other.center());
        let reach = scale(add(self.size(), other.size()), 0.5);
        (0..3).all(|axis| between[axis].abs() <= reach[axis])
    }
}
