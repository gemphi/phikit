use super::*;

#[test]
fn a_pattern_held_in_the_alpha_reads_black_in_its_holes() {
    // White everywhere, the shape in the alpha: half covered, half hole.
    let t = SceneTexture { width: 2, height: 1, rgba: Arc::new(vec![255, 255, 255, 255, 255, 255, 255, 0]) };
    let done = t.associated();
    assert_eq!(*done.rgba, vec![255, 255, 255, 255, 0, 0, 0, 0]);
    let opaque = SceneTexture { width: 1, height: 1, rgba: Arc::new(vec![9, 8, 7, 255]) };
    assert!(Arc::ptr_eq(&opaque.associated().rgba, &opaque.rgba), "an opaque image keeps its pixels");
}

#[test]
fn relative_paths_resolve_next_to_the_blend() {
    let c = SceneTexture::candidates("//tex\\a.png", Some(Path::new("/w/scene.blend")));
    assert_eq!(c[0], PathBuf::from("/w/tex/a.png"));
    assert_eq!(c[1], PathBuf::from("/w/a.png"));
}

#[test]
fn sample_is_bottom_up_and_wraps() {
    // 2×2, rows top-down: TL blue, TR yellow / BL red, BR green.
    let px = [0, 0, 255, 255, 255, 255, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255];
    let t = SceneTexture { width: 2, height: 2, rgba: Arc::new(px.to_vec()) };
    assert_eq!(t.sample([0.25, 0.25]), [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(t.sample([0.75, 0.75]), [1.0, 1.0, 0.0, 1.0]);
    assert_eq!(t.sample([1.25, 0.25]), [1.0, 0.0, 0.0, 1.0]);
}
#[test]
fn a_smooth_sample_at_a_texel_centre_is_that_texel() {
    // 2×2, rows top-down: TL blue, TR yellow / BL red, BR green.
    let px = [0, 0, 255, 255, 255, 255, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255];
    let tex = SceneTexture { width: 2, height: 2, rgba: std::sync::Arc::new(px.to_vec()) };
    // Texel centres in UV space (origin bottom-left) for the bottom row.
    let left = tex.sample_smooth([0.25, 0.25]);
    assert!(left[0] > 0.9 && left[1] < 0.1, "bottom-left is red: {left:?}");
    let right = tex.sample_smooth([0.75, 0.25]);
    assert!(right[1] > 0.9 && right[0] < 0.1, "bottom-right is green: {right:?}");
}

#[test]
fn a_smooth_sample_between_texels_blends_them() {
    let px = [0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255];
    let tex = SceneTexture { width: 2, height: 2, rgba: std::sync::Arc::new(px.to_vec()) };
    // Halfway between a black and a white texel on the bottom row.
    let mid = tex.sample_smooth([0.5, 0.25]);
    let near = tex.sample([0.5, 0.25]);
    assert!(mid[0] > 0.05 && mid[0] < 0.95, "the blend is neither texel: {mid:?}");
    assert!(
        (mid[0] - near[0]).abs() > 0.05,
        "and differs from the nearest-texel answer"
    );
}
