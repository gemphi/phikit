// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::*;

#[test]
fn the_sequences_of_base_two_and_three() {
    let mut state = HaltonSequence::default();
    let jitter: Vec<[f32; 2]> = (0..6).map(|_| state.next()).collect();
    let two = [1.0 / 2.0, 1.0 / 4.0, 3.0 / 4.0, 1.0 / 8.0, 5.0 / 8.0, 3.0 / 8.0];
    let three = [1.0 / 3.0, 2.0 / 3.0, 1.0 / 9.0, 4.0 / 9.0, 7.0 / 9.0, 2.0 / 9.0];
    let expect: Vec<[f32; 2]> = two.iter().zip(three).map(|(two, three): (&f32, f32)| [two - 0.5, three - 0.5]).collect();
    assert_eq!(jitter, expect);
}

#[test]
fn a_jitter_is_within_half_a_pixel() {
    let mut state = HaltonSequence::default();
    assert!((0..1000).map(|_| state.next()).all(|jitter| jitter.iter().all(|part| (-0.5..0.5).contains(part))));
}

#[test]
fn a_reset_starts_the_sequence_over() {
    let mut state = HaltonSequence::default();
    let first = state.next();
    state.next();
    state.reset();
    assert_eq!(state, HaltonSequence::default());
    assert_eq!(state.next(), first);
}
