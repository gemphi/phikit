// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::bvh::*;
use crate::util::BoundBox;

#[test]
fn a_reference_holds_for_the_whole_shutter_unless_told() {
    let reference = BvhReference::new(BoundBox { min: [0.0; 3], max: [1.0; 3] }, 7, 2, 1);
    assert_eq!((reference.prim_index, reference.prim_object, reference.prim_type), (7, 2, 1));
    assert_eq!((reference.time_from, reference.time_to, reference.object()), (0.0, 1.0, 2));
}

#[test]
fn a_range_ends_where_the_next_starts() {
    let range = BvhRange::new(BoundBox { min: [0.0; 3], max: [1.0; 3] }, 3, 4);
    assert_eq!((range.start, range.size, range.end()), (3, 4, 7));
    assert!(!range.cent_bound.valid());
    assert_eq!((BvhRange::default().size, BvhRange::default().bound), (0, BoundBox::EMPTY));
}

#[test]
fn the_storage_has_a_row_of_bins_for_each_axis() {
    let storage = BvhSpatialStorage::new();
    assert!(storage.bin.iter().all(|bin| bin.len() == BvhLimit::get().spatial_bin_count));
    assert!(storage.right_bound.is_empty() && storage.new_reference.is_empty());
    assert_eq!(storage.bin[0][0], BvhSpatialBin { bound: BoundBox::EMPTY, enter: 0, exit: 0 });
    assert_eq!(storage, BvhSpatialStorage::default());
}
