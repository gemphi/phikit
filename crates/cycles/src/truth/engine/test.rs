// SPDX-FileCopyrightText: 2015-2023 Blender Authors
// SPDX-License-Identifier: Apache-2.0
//! Tests of `truth/engine.rs`: a test scene of Blender's rendered by the
//! kernels and judged against the image Blender made of it.
use crate::truth::verdict::test::root;
use crate::truth::*;

fn scratch() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("cycles-engine-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn a_test_scene_is_rendered_at_the_size_of_its_reference_and_reports_what_it_used() {
    let Some(root) = root() else { return };
    let scene = TestScene::new(&root, "shader", "material_output_selection");
    let mut renderer = SceneRenderer::with_samples(2);
    let image = renderer.render(&scene).expect("the kernels render it");
    let reference = Image::read(&scene.reference).unwrap();
    assert_eq!((image.width, image.height, image.channel_count), (reference.width, reference.height, reference.channel_count));
    let report = renderer.last.clone().expect("a report");
    assert_eq!((report.sample_count, report.scene_sample_count, report.size), (2, 25, (128, 128)));
    assert!(report.missing.is_empty() && !report.view.approximated());
    // The world fills the corner, the emission the middle: both flat.
    let pixel = |x: usize, y: usize| (0..3).map(|c| image.channel(y * image.width + x, c)).collect::<Vec<_>>();
    let ref_pixel = |x: usize, y: usize| (0..3).map(|c| reference.channel(y * reference.width + x, c)).collect::<Vec<_>>();
    for (x, y) in [(0, 0), (127, 127), (64, 64), (100, 20)] {
        for (ours, theirs) in pixel(x, y).iter().zip(ref_pixel(x, y)) {
            assert!((ours - theirs).abs() < 0.01, "pixel {x},{y}: {:?} against {:?}", pixel(x, y), ref_pixel(x, y));
        }
    }
}

#[test]
fn the_first_scene_of_gate_one_passes_the_judge_at_its_own_sample_count() {
    // Two Material Outputs, a world and a Gaussian filter, drawn with the
    // Classic sampler: the same samples as Blender took, so the same picture.
    let Some(root) = root() else { return };
    let scene = TestScene::new(&root, "shader", "material_output_selection");
    let mut renderer = SceneRenderer::new();
    let verdict = judge(&scene, &mut renderer, &scratch());
    let comparison = verdict.comparison.as_ref().expect("compared");
    assert!(verdict.passed(), "{:?} {:?}", verdict.outcome, comparison);
    assert!(comparison.failed_percent < 0.1 && comparison.max_error < 0.01, "{comparison:?}");
}

#[test]
fn a_scene_with_lights_says_what_it_could_not_draw() {
    let Some(root) = root() else { return };
    let scene = TestScene::new(&root, "bsdf", "diffuse");
    let mut renderer = SceneRenderer::with_samples(1);
    renderer.render(&scene).expect("it still renders");
    let report = renderer.last.expect("a report");
    assert_eq!(report.missing, ["1 lights"]);
}

#[test]
fn a_scene_that_is_not_there_is_an_error() {
    let mut renderer = SceneRenderer::new();
    let scene = TestScene::new(std::path::Path::new("nowhere"), "a", "b");
    assert!(matches!(renderer.render(&scene), Err(TruthError::Image { .. })));
}
