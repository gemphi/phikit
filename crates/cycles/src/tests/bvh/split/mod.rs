// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/bvh/split`, one file for each file there. This file
//! tests `mod.rs`.

mod object;
mod primitive;
mod spatial;

use super::*;

/// Two triangles a unit apart at each end of x from 0 to 4, each in a
/// box of side 1, and a thin one that runs under both.
pub(crate) fn under_two() -> Mesh {
    let point = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 1.0], [3.0, 0.0, 0.0], [4.0, 0.0, 0.0], [3.0, 1.0, 1.0], [4.0, 0.5, 0.0]];
    mesh_of(&point, &[[0, 1, 2], [3, 4, 5], [0, 4, 6]])
}

/// Run `run` on a builder over one object that places `mesh`.
pub(crate) fn build_over<R>(mesh: &Mesh, run: impl FnOnce(&mut BvhBuild<'_>) -> R) -> R {
    let (mut pack, object) = (PackedBvh::default(), [Object::new(0)]);
    run(&mut BvhBuild::new(&object, std::slice::from_ref(mesh), &mut pack, BvhParam::default()))
}

#[test]
fn a_range_that_costs_less_cut_is_cut_by_sorting_when_its_halves_do_not_overlap() {
    let (range, mut reference) = row(&[0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0]);
    let mesh = square();
    build_over(&mesh, |build| {
        build.spatial_min_overlap = 1.0;
        let mut storage = BvhSpatialStorage::new();
        let split = BvhMixedSplit::new(build, &mut storage, &range, &mut reference, 0);
        // The row's area is 62: a leaf of eight, a node of two.
        assert_eq!((split.leaf_sah, split.node_sah, split.bound), (496.0, 124.0, range.bound));
        assert_eq!((split.spatial.sah, split.min_sah, split.no_split), (f32::MAX, split.object.sah, false));
        let (left, right) = split.split(build, &mut storage, &mut reference, &range);
        assert_eq!((left.size, right.start, right.size, reference.len()), (4, 4, 4, 8));
    });
}

#[test]
fn a_range_that_costs_least_whole_and_fits_a_leaf_is_not_split() {
    let (range, mut reference) = row(&[0.0, 0.5]);
    build_over(&square(), |build| {
        let split = BvhMixedSplit::new(build, &mut BvhSpatialStorage::new(), &range, &mut reference, 0);
        assert!(split.no_split && split.min_sah == split.leaf_sah);
        // Too many for a leaf: split, whatever it costs.
        build.param.max_triangle_leaf_size = 1;
        assert!(!BvhMixedSplit::new(build, &mut BvhSpatialStorage::new(), &range, &mut reference, 0).no_split);
    });
}

#[test]
fn a_cut_is_tried_only_above_the_depth_cycles_stops_at() {
    let mesh = under_two();
    let (range, mut reference) = reference_of(&mesh);
    build_over(&mesh, |build| {
        let mut storage = BvhSpatialStorage::new();
        assert!(BvhMixedSplit::new(build, &mut storage, &range, &mut reference, 0).spatial.sah < f32::MAX);
        assert_eq!(BvhMixedSplit::new(build, &mut storage, &range, &mut reference, BvhLimit::get().max_spatial_depth).spatial.sah, f32::MAX);
        build.param.use_spatial_split = false;
        assert_eq!(BvhMixedSplit::new(build, &mut storage, &range, &mut reference, 0).spatial.sah, f32::MAX);
    });
}
