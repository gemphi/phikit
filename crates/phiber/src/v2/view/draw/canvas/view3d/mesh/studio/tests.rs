//! The rig has to behave like Blender's, not merely be four numbers.

use super::{RIG, lighting, wrapped};

/// A plain grey dielectric, so the test reads the rig rather than a material.
fn grey(normal: (f32, f32, f32)) -> [f32; 3] {
    lighting([0.5, 0.5, 0.5], 0.5, 0.0, normal)
}

#[test]
fn wrapping_softens_the_terminator() {
    // Straight at the light is full either way.
    assert!((wrapped(1.0, 0.0) - 1.0).abs() < 1e-6);
    // At ninety degrees a hard light is black; a wrapped one is not. That
    // gradual falloff is what Solid mode looks like.
    assert_eq!(wrapped(0.0, 0.0), 0.0);
    assert!(wrapped(0.0, 0.5) > 0.1, "a soft light wraps past the terminator");
    // And it never exceeds full.
    assert!(wrapped(1.0, 0.9) <= 1.0);
}

#[test]
fn every_light_in_the_rig_points_somewhere() {
    for (n, l) in RIG.iter().enumerate() {
        let length = (l.vec[0] * l.vec[0] + l.vec[1] * l.vec[1] + l.vec[2] * l.vec[2]).sqrt();
        assert!((length - 1.0).abs() < 0.01, "light {n} is not a unit direction");
    }
}

#[test]
fn the_key_light_is_the_bright_one_and_it_faces_the_viewer() {
    // Blender's Default rig has one dominant light (index 1) pointing out of
    // the screen; the other three are fill and rim at a fraction of it.
    let key = &RIG[1];
    assert!(key.vec[2] > 0.8, "the key light comes from behind the viewer");
    for (n, l) in RIG.iter().enumerate() {
        if n == 1 {
            continue;
        }
        assert!(l.col[0] < key.col[0], "light {n} must be dimmer than the key");
    }
}

#[test]
fn a_surface_facing_the_viewer_is_brighter_than_one_facing_away() {
    let toward = grey((0.0, 0.0, 1.0));
    let across = grey((1.0, 0.0, 0.0));
    let sum = |v: [f32; 3]| v[0] + v[1] + v[2];
    assert!(sum(toward) > sum(across), "{toward:?} vs {across:?}");
}

#[test]
fn lighting_is_two_sided() {
    // With backface culling off, Blender lights a surface pointing away from
    // the camera the same as one pointing at it.
    assert_eq!(grey((0.0, 0.0, 1.0)), grey((0.0, 0.0, -1.0)));
}

/// The normal that mirrors the key light straight back at the viewer: the
/// half vector between the key's direction and the view axis. A highlight is
/// at its strongest here, which is where roughness is worth measuring.
fn mirror_of_the_key() -> (f32, f32, f32) {
    let key = RIG[1].vec;
    let h = (key[0], key[1], key[2] + 1.0);
    let len = (h.0 * h.0 + h.1 * h.1 + h.2 * h.2).sqrt();
    (h.0 / len, h.1 / len, h.2 / len)
}

#[test]
fn a_metal_has_no_diffuse() {
    // `diffuse_color = mix(base, 0, metallic)`: at metallic 1 the surface's
    // own colour stops contributing diffusely and only the highlight is left,
    // so a metal away from a highlight is far darker than a dielectric.
    let n = (0.0, 0.0, 1.0);
    let red = [0.8, 0.05, 0.05];
    let sum = |v: [f32; 3]| v[0] + v[1] + v[2];
    let dielectric = sum(lighting(red, 0.3, 0.0, n));
    let metal = sum(lighting(red, 0.3, 1.0, n));
    assert!(metal < dielectric * 0.5, "metal {metal} vs dielectric {dielectric}");
}

#[test]
fn a_polished_surface_has_a_tighter_hotter_highlight_than_a_rough_one() {
    // Blinn's normalization factor concentrates a polished surface's light
    // into the mirror direction and spreads a rough one's out, so the two
    // swap places either side of that direction.
    let sum = |v: [f32; 3]| v[0] + v[1] + v[2];
    let on = mirror_of_the_key();
    assert!(
        sum(lighting([0.5; 3], 0.05, 0.0, on)) > sum(lighting([0.5; 3], 0.9, 0.0, on)),
        "the polished surface is brighter in the mirror direction"
    );
    // A normal well away from it: now the rough surface keeps more.
    let off = (0.8, 0.0, 0.6);
    assert!(
        sum(lighting([0.5; 3], 0.9, 0.0, off)) > sum(lighting([0.5; 3], 0.05, 0.0, off)),
        "and the rough one is brighter away from it"
    );
}

#[test]
fn specular_adds_light_rather_than_replacing_it() {
    // In the mirror direction the highlight is on top of the diffuse, so a
    // shiny surface is brighter there than the same surface made matte.
    let n = mirror_of_the_key();
    let sum = |v: [f32; 3]| v[0] + v[1] + v[2];
    let matte = sum(lighting([0.5; 3], 1.0, 0.0, n));
    let shiny = sum(lighting([0.5; 3], 0.1, 0.0, n));
    assert!(shiny > matte, "shiny {shiny} vs matte {matte}");
}
