// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/bvh/sort.h` and `sort.cpp`
//! (`BVHReferenceCompare`, `bvh_reference_sort`), adapted by Cycles from
//! code of NVIDIA Corporation: references in order along an axis.
//!
//! Cycles sorts a long run on several threads with its own quicksort;
//! here every run goes through the standard sort, which gives the same
//! order. Not ported: sorting in the aligned space of a curve.

use std::cmp::Ordering;

use super::BvhReference;

/// `BVHReferenceCompare::compare`: by the centre along `dim`, then by
/// object, primitive and type, so that the order is the same on every run.
pub fn bvh_reference_compare(dim: usize, a: &BvhReference, b: &BvhReference) -> Ordering {
    let center = |reference: &BvhReference| reference.bound.min[dim] + reference.bound.max[dim];
    let by_center = center(a).partial_cmp(&center(b)).unwrap_or(Ordering::Equal);
    by_center.then(a.prim_object.cmp(&b.prim_object)).then(a.prim_index.cmp(&b.prim_index)).then(a.prim_type.cmp(&b.prim_type))
}

/// `bvh_reference_sort`: these references in order along `dim`.
pub fn bvh_reference_sort(reference: &mut [BvhReference], dim: usize) {
    reference.sort_by(|a, b| bvh_reference_compare(dim, a, b));
}

#[cfg(test)]
mod test;
