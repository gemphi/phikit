// SPDX-FileCopyrightText: 2018-2023 Blender Authors
// SPDX-License-Identifier: Apache-2.0
//! Tests of `truth/scene.rs`, `truth/image.rs`, `truth/verdict.rs` and
//! `truth/render.rs`, over Blender's own test scenes when they are there.
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::device::{CpuDevice, CpuKernel, DeviceError, KernelArgument};
use crate::kernel::DeviceKernel;
use crate::truth::*;

/// The test scenes, or `None` where the workspace has no copy of them.
pub(crate) fn root() -> Option<PathBuf> {
    find_root(Path::new(env!("CARGO_MANIFEST_DIR"))).ok()
}

/// A scratch folder for one test, empty.
fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("cycles-truth-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

/// A renderer that returns a fixed image.
struct Fixed(Image);

impl Renderer for Fixed {
    fn render(&mut self, _scene: &TestScene) -> Result<Image, TruthError> {
        Ok(self.0.clone())
    }
}

#[test]
fn a_scene_is_a_blend_and_the_png_beside_it_in_cycles_renders() {
    let scene = TestScene::new(Path::new("root"), "bsdf", "diffuse");
    assert_eq!(scene.blend, Path::new("root/bsdf/diffuse.blend"));
    assert_eq!(scene.reference, Path::new("root/bsdf/cycles_renders/diffuse.png"));
    assert!(!scene.exists());
}

#[test]
fn the_test_scenes_are_found_and_listed_by_name() {
    let Some(root) = root() else { return };
    let list = TestScene::list(&root, "bsdf");
    assert!(list.len() > 20, "bsdf has {} scenes", list.len());
    assert!(list.windows(2).all(|pair| pair[0].name < pair[1].name));
    let diffuse = list.iter().find(|scene| scene.name == "diffuse").unwrap();
    assert!(diffuse.exists());
    assert!(TestScene::list(&root, "no_such_category").is_empty());
}

#[test]
fn a_reference_png_reads_as_bytes_over_255() {
    let Some(root) = root() else { return };
    let scene = TestScene::new(&root, "bsdf", "diffuse");
    let image = Image::read(&scene.reference).unwrap();
    assert!(image.width > 0 && image.height > 0 && image.channel_count == 3, "the references are opaque RGB");
    assert_eq!(image.pixel.len(), image.width * image.height * image.channel_count);
    // Every value is a byte over 255.
    assert!(image.pixel.iter().all(|value| (value * 255.0 - (value * 255.0).round()).abs() < 1e-3 && (0.0..=1.0).contains(value)));
}

#[test]
fn a_real_reference_passes_against_itself_and_writes_its_diff() {
    let Some(root) = root() else { return };
    let scene = TestScene::new(&root, "bsdf", "diffuse");
    let reference = Image::read(&scene.reference).unwrap();
    let out = scratch("self");
    let verdict = judge(&scene, &mut Fixed(reference.clone()), &out);
    assert_eq!(verdict.outcome, Outcome::Pass);
    let comparison = verdict.comparison.unwrap();
    assert_eq!((comparison.failed_pixel, comparison.max_error), (0, 0.0));
    let diff = Image::read(verdict.diff_color.as_ref().unwrap()).unwrap();
    assert_eq!((diff.width, diff.height, diff.channel_count), (reference.width, reference.height, 3));
    assert!(diff.pixel.iter().all(|value| *value == 0.0), "no difference is a black diff");
    assert!(verdict.diff_alpha.is_none(), "an RGB reference has no alpha to diff");
    std::fs::remove_dir_all(out).unwrap();
}

#[test]
fn a_real_reference_perturbed_fails_at_the_documented_threshold() {
    let Some(root) = root() else { return };
    let scene = TestScene::new(&root, "bsdf", "diffuse");
    let reference = Image::read(&scene.reference).unwrap();
    let out = scratch("perturbed");
    
    let pixel = reference.width * reference.height;
    let one_percent = pixel / 100;
    let raise = |count: usize, by: f32| {
        let mut image = reference.clone();
        let stride = reference.channel_count;
        for index in 0..count {
            let red = reference.pixel[index * stride];
            image.pixel[index * stride] = if red < 0.5 { red + by } else { red - by };
        }
        image
    };
    let at_threshold = judge(&scene, &mut Fixed(raise(pixel, 0.0159)), &out);
    assert_eq!(at_threshold.outcome, Outcome::Pass, "every pixel just under the threshold: none is over");
    let over_by_a_hair = judge(&scene, &mut Fixed(raise(one_percent + 1, 0.0161)), &out);
    assert_eq!(over_by_a_hair.outcome, Outcome::Fail, "more than 1 per cent of the pixels past it");
    let within = judge(&scene, &mut Fixed(raise(one_percent, 0.0161)), &out);
    assert_eq!(within.outcome, Outcome::Pass, "1 per cent past it is allowed");
    std::fs::remove_dir_all(out).unwrap();
}

#[test]
fn a_scene_without_a_reference_has_no_verdict_to_give() {
    let scene = TestScene::new(Path::new("nowhere"), "bsdf", "nothing");
    let verdict = judge(&scene, &mut Fixed(Image::new(1, 1, 4)), Path::new("nowhere"));
    assert_eq!(verdict.outcome, Outcome::NoReference);
    assert!(verdict.comparison.is_none() && !verdict.passed());
}

#[test]
fn a_render_of_the_wrong_size_fails_the_judge_with_a_message() {
    let Some(root) = root() else { return };
    let scene = TestScene::new(&root, "bsdf", "diffuse");
    let verdict = judge(&scene, &mut Fixed(Image::new(2, 2, 4)), &scratch("size"));
    assert!(matches!(&verdict.outcome, Outcome::RenderFailed(message) if message.contains("render 2x2")));
}

#[test]
fn the_category_sets_the_allowance_in_the_verdict() {
    let Some(root) = root() else { return };
    let scene = TestScene::new(&root, "light", "area_light");
    if !scene.exists() {
        return;
    }
    let verdict = judge(&scene, &mut Fixed(Image::new(1, 1, 4)), &scratch("light"));
    assert_eq!(verdict.allowance, Allowance { threshold: 0.02, percent: 4.0 });
}

#[test]
fn the_session_renderer_stops_with_not_ported_until_the_kernel_exists() {
    let mut renderer = SessionRenderer::new(Arc::new(CpuDevice::new()), (16, 8), 2);
    let scene = TestScene::new(Path::new("root"), "bsdf", "diffuse");
    let error = renderer.render(&scene).unwrap_err();
    assert_eq!(error, TruthError::Device(DeviceError::NotPorted(DeviceKernel::IntegratorMegakernel)));
}

#[test]
fn a_render_error_is_the_verdict_and_not_a_black_picture() {
    let Some(root) = root() else { return };
    let scene = TestScene::new(&root, "bsdf", "diffuse");
    let mut renderer = SessionRenderer::new(Arc::new(CpuDevice::new()), (16, 8), 2);
    let verdict = judge(&scene, &mut renderer, &scratch("notported"));
    assert!(matches!(&verdict.outcome, Outcome::RenderFailed(message) if message.contains("not ported")), "{:?}", verdict.outcome);
}

/// A megakernel that writes `sample_count` into red and alpha of each pixel.
fn white_kernel() -> CpuDevice {
    let mut kernels = CpuKernel::new();
    kernels.set(
        DeviceKernel::IntegratorMegakernel,
        Arc::new(|memory, _work_size, args| {
            let bad = |index| DeviceError::BadArgument { kernel: DeviceKernel::IntegratorMegakernel, index };
            let buffer = args.first().and_then(KernelArgument::as_ptr).ok_or(bad(0))?;
            let int = |index: usize| args.get(index).and_then(KernelArgument::as_i32).ok_or(bad(index));
            let (x, y, w, h, samples, offset, stride) = (int(1)?, int(2)?, int(3)?, int(4)?, int(6)?, int(8)?, int(9)?);
            let floats = memory.slice_mut::<f32>(buffer)?;
            for row in y..y + h {
                for column in x..x + w {
                    let pixel = (offset + row * stride + column) as usize * 4;
                    floats[pixel..pixel + 4].iter_mut().for_each(|value| *value += samples as f32);
                }
            }
            Ok(())
        }),
    );
    CpuDevice::with_kernels(kernels)
}

#[test]
fn the_session_renderer_turns_a_ported_kernel_into_a_picture() {
    let mut renderer = SessionRenderer::new(Arc::new(white_kernel()), (16, 8), 4);
    let image = renderer.render(&TestScene::new(Path::new("root"), "bsdf", "diffuse")).unwrap();
    assert_eq!((image.width, image.height, image.channel_count), (16, 8, 3));
    // The kernel wrote the sample count of its round; the rounds add up to
    // the whole count and the picture divides it out: white.
    assert!(image.pixel.iter().all(|value| (*value - 1.0).abs() < 1e-6), "{:?}", &image.pixel[..3]);
    let mut transparent = SessionRenderer::new(Arc::new(white_kernel()), (4, 4), 1);
    transparent.transparent = true;
    assert_eq!(transparent.render(&TestScene::new(Path::new("root"), "bsdf", "diffuse")).unwrap().channel_count, 4);
}

#[test]
fn the_view_is_the_srgb_curve() {
    assert_eq!(srgb_encode(0.0), 0.0);
    assert_eq!(srgb_encode(1.0), 1.0);
    assert!((srgb_encode(0.5) - 0.7354).abs() < 1e-3);
    assert!((srgb_encode(0.002) - 0.02584).abs() < 1e-4, "the linear toe");
    assert_eq!(srgb_encode(7.0), 1.0);
}
