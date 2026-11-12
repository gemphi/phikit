// SPDX-License-Identifier: Apache-2.0
use crate::kernel::util::*;

#[test]
fn a_compact_differential_is_the_mean_length_of_the_two_changes() {
    assert_eq!(differential_zero_compact(), 0.0);
    let change = Differential3 { dx: [3.0, 4.0, 0.0], dy: [0.0, 0.0, 1.0] };
    assert_eq!(differential_make_compact(&change), 3.0);
    assert_eq!(differential_make_compact(&Differential3::default()), 0.0);
}
