// SPDX-FileCopyrightText: 2011-2013 Intel Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::util::vector::*;

#[test]
fn vectors_add_subtract_scale_and_blend_part_by_part() {
    assert_eq!(add([1.0, 2.0, 3.0], [10.0, 20.0, 30.0]), [11.0, 22.0, 33.0]);
    assert_eq!(sub([1.0, 2.0, 3.0, 4.0], [1.0, 1.0, 1.0, 1.0]), [0.0, 1.0, 2.0, 3.0]);
    assert_eq!(scale([1.0, -2.0, 3.0], 2.0), [2.0, -4.0, 6.0]);
    assert_eq!(interp([0.0, 10.0, 0.0], [4.0, 20.0, 0.0], 0.25), [1.0, 12.5, 0.0]);
}

#[test]
fn the_cross_of_x_and_y_is_z() {
    assert_eq!(cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
    assert_eq!(dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), 32.0);
    assert_eq!(len([3.0, 4.0, 0.0]), 5.0);
}

#[test]
fn a_vector_of_no_length_normalizes_to_not_a_number() {
    assert_eq!(normalize([0.0, 0.0, 2.0]), [0.0, 0.0, 1.0]);
    assert!(normalize([0.0f32; 3]).iter().all(|part| part.is_nan()));
    assert_eq!(safe_divide([1.0, 2.0, 3.0], 0.0), [0.0; 3]);
    assert_eq!(safe_divide([1.0, 2.0, 3.0], 2.0), [0.5, 1.0, 1.5]);
}

#[test]
fn a_part_that_is_not_a_number_loses_to_the_other_vector_s() {
    let odd = [f32::NAN, 5.0, -5.0];
    assert_eq!(min(odd, [1.0; 3]), [1.0, 1.0, -5.0]);
    assert_eq!(max(odd, [1.0; 3]), [1.0, 5.0, 1.0]);
    assert!(!is_finite(odd) && !is_finite([f32::INFINITY, 0.0, 0.0]) && is_finite([1.0; 3]));
}

#[test]
fn mix_steps_from_the_first_towards_the_second() {
    assert_eq!(mix([0.0, 4.0, 1.0], [4.0, 0.0, 1.0], 0.25), [1.0, 3.0, 1.0]);
    assert_eq!((mix([2.0], [6.0], 0.0), mix([2.0], [6.0], 1.0)), ([2.0], [6.0]));
}
