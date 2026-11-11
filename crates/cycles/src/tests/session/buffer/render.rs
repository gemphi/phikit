// SPDX-License-Identifier: Apache-2.0
use crate::scene::{Pass, PassMode, PassType};
use crate::session::*;

fn pass(name: &str, mode: PassMode) -> Pass {
    Pass::new(PassType::named(name).expect("in data/pass.tsv"), mode)
}

fn buffer(width: i32, height: i32, scene_pass: &[Pass]) -> RenderBuffer {
    let mut param = BufferParam::full(width, height);
    param.update_pass_from(scene_pass);
    let mut made = RenderBuffer::default();
    made.reset(&param);
    made
}

#[test]
fn a_buffer_holds_pass_stride_floats_a_pixel_zeroed() {
    let made = buffer(8, 4, &[pass("PASS_COMBINED", PassMode::Noisy), pass("PASS_DEPTH", PassMode::Noisy)]);
    assert_eq!(made.buffer.len(), 8 * 4 * 5);
    assert!(made.buffer.iter().all(|float| *float == 0.0));
    assert_eq!(made.pixel(7, 3).len(), 5);
}

#[test]
fn a_pixel_written_is_the_pixel_read_and_zero_clears_it() {
    let mut made = buffer(8, 4, &[pass("PASS_COMBINED", PassMode::Noisy)]);
    made.pixel_mut(3, 2).copy_from_slice(&[0.1, 0.2, 0.3, 1.0]);
    assert_eq!(made.pixel(3, 2), [0.1, 0.2, 0.3, 1.0]);
    assert_eq!(made.pixel(2, 2), [0.0; 4]);
    made.zero();
    assert_eq!(made.pixel(3, 2), [0.0; 4]);
}

#[test]
fn denoised_passes_copy_between_buffers_laid_out_differently() {
    let noisy = pass("PASS_COMBINED", PassMode::Noisy);
    let denoised = pass("PASS_COMBINED", PassMode::Denoised);
    let mut source = buffer(4, 2, &[noisy, denoised.clone()]);
    let mut target = buffer(4, 2, &[denoised]);
    for (index, pixel) in source.buffer.chunks_exact_mut(8).enumerate() {
        pixel[4..8].copy_from_slice(&[index as f32, 0.5, 0.25, 1.0]);
    }
    target.copy_denoised(&source, 0);
    assert_eq!(target.pixel(3, 1), [7.0, 0.5, 0.25, 1.0]);
    assert_eq!(target.pixel(0, 0), [0.0, 0.5, 0.25, 1.0]);
}
