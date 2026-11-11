// SPDX-License-Identifier: Apache-2.0
use std::rc::Rc;

use crate::util::*;

#[test]
fn a_manual_clock_reads_what_it_was_set_to() {
    let clock = ManualClock::at(100.0);
    assert_eq!(clock.now(), 100.0);
    clock.advance(2.5);
    assert_eq!(clock.now(), 102.5);
    clock.set(7.0);
    assert_eq!(clock.now(), 7.0);
}

#[test]
fn a_shared_clock_is_one_clock() {
    let clock = Rc::new(ManualClock::at(1.0));
    let held = clock.clone();
    clock.advance(1.0);
    assert_eq!(held.now(), 2.0);
    assert_eq!((&*held).now(), 2.0);
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn the_system_clock_is_never_zero_and_never_goes_back_far() {
    let first = SystemClock.now();
    let second = SystemClock.now();
    assert!(first > 0.0);
    assert!(second >= first - 1.0);
}
