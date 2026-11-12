// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::integrator::*;

fn spent(time: &[f64]) -> Vec<WorkBalanceInfo> {
    let mut list: Vec<WorkBalanceInfo> = time.iter().map(|time_spent| WorkBalanceInfo { time_spent: *time_spent, ..WorkBalanceInfo::default() }).collect();
    WorkBalanceInfo::share_evenly(&mut list);
    list
}

#[test]
fn before_anything_is_timed_the_shares_are_equal() {
    let weight: Vec<f64> = spent(&[0.0; 4]).iter().map(|info| info.weight).collect();
    assert_eq!(weight, [0.25; 4]);
    assert_eq!(spent(&[0.0])[0].weight, 1.0);
}

#[test]
fn a_fresh_share_takes_its_device_as_fully_occupied() {
    assert_eq!(WorkBalanceInfo::default().occupancy, 1.0);
}

#[test]
fn devices_that_took_the_same_time_keep_their_shares() {
    let mut list = spent(&[2.0, 2.0]);
    assert!(!WorkBalanceInfo::rebalance(&mut list));
    assert_eq!(list, spent(&[2.0, 2.0]));
}

#[test]
fn times_within_two_percent_are_left_alone() {
    let mut list = spent(&[1.0, 1.03]);
    assert!(!WorkBalanceInfo::rebalance(&mut list));
    assert_eq!(list[1].time_spent, 1.03);
}

#[test]
fn the_faster_device_gets_more_and_the_timing_starts_over() {
    // One second against three: each moves half of the way to the average.
    let mut list = spent(&[1.0, 3.0]);
    assert!(WorkBalanceInfo::rebalance(&mut list));
    assert!((list[0].weight - 9.0 / 14.0).abs() < 1e-12, "{list:?}");
    assert!((list[1].weight - 5.0 / 14.0).abs() < 1e-12, "{list:?}");
    assert_eq!((list[0].time_spent, list[1].time_spent), (0.0, 0.0));
}

#[test]
fn the_shares_always_add_up_to_the_frame() {
    let mut list = spent(&[1.0, 2.0, 7.0, 0.5]);
    assert!(WorkBalanceInfo::rebalance(&mut list));
    assert!((list.iter().map(|info| info.weight).sum::<f64>() - 1.0).abs() < 1e-12);
}
