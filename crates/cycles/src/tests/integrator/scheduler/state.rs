// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::integrator::*;

#[test]
fn a_fresh_state_is_at_full_resolution_with_no_display_update_yet() {
    let state = SchedulerState::default();
    assert_eq!((state.resolution_divider, state.rendered_sample_count, state.last_display_update_sample), (1, 0, -1));
    assert!(!state.user_is_navigating && !state.path_trace_finished && !state.time_limit_reached);
}

#[test]
fn a_fresh_state_takes_the_device_as_fully_occupied() {
    let state = SchedulerState::default();
    assert_eq!((state.occupancy_sample_count, state.occupancy), (0, 1.0));
}

#[test]
fn no_first_render_was_timed_yet() {
    assert_eq!(FirstRenderTime::default().path_trace_per_sample, 0.0);
}
