// SPDX-License-Identifier: Apache-2.0
use crate::session::*;

#[test]
fn the_defaults_are_those_of_cycles() {
    let param = SessionParam::default();
    assert_eq!((param.sample_count, param.tile_size, param.pixel_size), (1024, 2048, 1));
    assert!(param.use_auto_tile && param.use_resolution_divider && !param.background);
}

#[test]
fn more_samples_keep_the_session_and_a_new_tile_size_does_not() {
    let base = SessionParam::default();
    assert!(!base.modified(&SessionParam { sample_count: 64, time_limit: 30.0, ..base.clone() }));
    assert!(base.modified(&SessionParam { tile_size: 512, ..base.clone() }));
}
