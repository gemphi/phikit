// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::kernel::util::*;

/// Two entries of another table, then one of three entries.
const TABLE: [f32; 5] = [7.0, 7.0, 0.0, 10.0, 20.0];

#[test]
fn a_table_is_read_between_its_two_nearest_entries() {
    let read = |x: f32| lookup_table_read(&TABLE, x, 2, 3);
    assert_eq!([read(0.0), read(0.5), read(1.0)], [0.0, 10.0, 20.0]);
    assert_eq!([read(0.25), read(0.75)], [5.0, 15.0]);
}

#[test]
fn outside_zero_to_one_the_nearer_end_is_read() {
    let read = |x: f32| lookup_table_read(&TABLE, x, 2, 3);
    assert_eq!([read(-1.0), read(2.0)], [0.0, 20.0]);
    // A table of one entry is that entry, and none at all is zero.
    assert_eq!((lookup_table_read(&TABLE, 0.6, 4, 1), lookup_table_read(&TABLE, 0.6, 5, 3)), (20.0, 0.0));
}
