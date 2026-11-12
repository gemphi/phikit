// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Cycles' own tests, ported from
//! `intern/cycles/test/integrator_render_scheduler_test.cpp`.
use crate::integrator::*;

#[test]
fn calculate_resolution_divider_for_resolution() {
    assert_eq!(divider_for_resolution(1920, 1080, 1920), 1);
    assert_eq!(divider_for_resolution(1920, 1080, 960), 2);
    assert_eq!(divider_for_resolution(1920, 1080, 480), 4);
    assert_eq!(divider_for_resolution(1920, 1080, i32::MAX), 1);
}

#[test]
fn calculate_resolution_for_divider() {
    assert_eq!(resolution_for_divider(1920, 1080, 1), 1440);
    assert_eq!(resolution_for_divider(1920, 1080, 2), 720);
    assert_eq!(resolution_for_divider(1920, 1080, 4), 360);
}
