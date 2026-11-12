// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::*;

fn near(left: f32, right: f32) -> bool {
    (left - right).abs() < 1e-4
}

#[test]
fn the_box_filter_spreads_samples_evenly_over_the_pixel() {
    let table = filter_table("box", 1.0);
    assert_eq!(table.len(), 1024);
    assert!(near(table[0], 0.0) && near(table[511], 0.5) && near(table[1022], 1.0), "{} {} {}", table[0], table[511], table[1022]);
    // 256 steps of 511 from the centre, each way.
    assert!(near(table[255], 0.5 - 0.5 * 256.0 / 511.0) && near(table[767], 0.5 + 0.5 * 256.0 / 511.0));
    // The table has an even size, and Cycles leaves its last entry alone.
    assert_eq!(table[1023], 0.0);
}

#[test]
fn a_table_rises_from_one_edge_of_the_filter_to_the_other() {
    for filter in ["box", "gaussian", "blackman_harris"] {
        let table = filter_table(filter, 1.5);
        assert!(table[..1023].windows(2).all(|pair| pair[0] <= pair[1]), "{filter}");
        assert!(near(table[511], 0.5), "{filter}");
    }
}

#[test]
fn the_gaussian_reaches_three_pixels_wide_and_gathers_samples_at_the_centre() {
    let table = filter_table("gaussian", 1.0);
    assert!(near(table[0], -1.0) && near(table[1022], 2.0), "{} {}", table[0], table[1022]);
    // Three quarters of the samples are closer to the centre than the box
    // would put them.
    assert!((table[767] - 0.5).abs() < 0.25 && (table[255] - 0.5).abs() < 0.25);
}

#[test]
fn blackman_harris_reaches_two_pixels_wide() {
    let table = filter_table("blackman_harris", 1.0);
    assert!(near(table[0], -0.5) && near(table[1022], 1.5), "{} {}", table[0], table[1022]);
}

#[test]
fn an_unknown_filter_is_the_box() {
    assert_eq!(filter_table("no_such_filter", 1.0), filter_table("box", 1.0));
}
