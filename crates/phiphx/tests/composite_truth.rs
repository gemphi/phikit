//! The compositor's per-pixel nodes against Blender's own compositor.
//!
//! `truth/blender_composite.py` renders a 4 × 4 image of known pixels
//! through one node at a time — Blender's real compositor, through
//! `bpy.ops.render.render` — and records the output in
//! `truth/composite.txt`. Here the same operation runs through
//! `phiphx::comp::pixel` and the two are compared.
//!
//! The *input* every case is compared against is the `passthrough` line,
//! not the `input` line: Blender writes its EXR in half precision whatever
//! `color_depth` says, so the pixels the nodes actually saw are a few parts
//! in 10⁵ off the ones asked for. Taking the passthrough as the input keeps
//! the file format out of the measurement.

use phiphx::comp::pixel::{self, BCompPixel};

const TRUTH: &str = include_str!("truth/composite.txt");
/// Half-precision in, so a few parts in 10⁵, plus room for the operations.
const TOL: f32 = 2e-4;

fn case(name: &str) -> Vec<BCompPixel> {
    let line = TRUTH
        .lines()
        .find(|l| l.split_whitespace().next() == Some(name))
        .unwrap_or_else(|| panic!("truth/composite.txt has no case {name}; rerun truth/blender_composite.py"));
    let v: Vec<f32> = line.split_whitespace().skip(1).map(|t| t.parse().expect("number")).collect();
    assert_eq!(v.len(), 64, "{name}: a 4 x 4 of RGBA");
    v.chunks_exact(4).map(|c| [c[0], c[1], c[2], c[3]]).collect()
}

/// `op` over the input, against the case Blender rendered.
fn same(name: &str, op: impl Fn(BCompPixel) -> BCompPixel) {
    let (input, want) = (case("passthrough"), case(name));
    let mut worst = (0.0f32, 0usize);
    for (i, (&was, &expect)) in input.iter().zip(&want).enumerate() {
        let got = op(was);
        for k in 0..4 {
            let off = (got[k] - expect[k]).abs() / expect[k].abs().max(1.0);
            if off > worst.0 {
                worst = (off, i);
            }
        }
        assert!(
            (0..4).all(|k| (got[k] - expect[k]).abs() <= TOL * expect[k].abs().max(1.0)),
            "{name} pixel {i}: {got:?} vs Blender's {expect:?} (from {was:?})"
        );
    }
    println!("{name}: worst {:.2e} at pixel {}", worst.0, worst.1);
}

#[test]
fn brightness_and_contrast_match_blender() {
    same("bright_contrast", |c| pixel::bright_contrast(c, 15.0, 40.0));
    // The negative branch is not the positive one mirrored: it multiplies
    // by a multiplier clamped at zero where the positive divides by one
    // clamped away from it.
    same("bright_contrast_down", |c| pixel::bright_contrast(c, -10.0, -60.0));
}

#[test]
fn exposure_gamma_and_posterize_match_blender() {
    same("exposure", |c| pixel::exposure(c, -1.5));
    same("gamma", |c| pixel::gamma(c, 2.2));
    same("posterize", |c| pixel::posterize(c, 5.0));
}

#[test]
fn invert_and_hue_saturation_match_blender() {
    // Invert's default is the COLOUR alone: `node->custom1 =
    // CMP_CHAN_RGB`, with `CMP_CHAN_A` clear. Asking for both here gave an
    // alpha of 0.25 against Blender's 1.0 — the node never touched it.
    same("invert", |c| pixel::invert(c, 0.75, true, false));
    same("hue_sat", |c| pixel::hue_sat(c, 0.65, 1.4, 0.8, 1.0));
}

#[test]
fn the_alpha_nodes_match_blender() {
    same("rgb_to_bw", |c| {
        let v = pixel::rgb_to_bw(c);
        [v, v, v, 1.0]
    });
    same("set_alpha_apply", |c| pixel::set_alpha(c, 0.4, pixel::SET_ALPHA_APPLY));
    same("premultiply", |c| pixel::alpha_convert(c, pixel::ALPHA_PREMULTIPLY));
    // Straight Alpha off is the default, and it means the foreground is
    // taken as ALREADY premultiplied — so it goes over untouched. With it
    // on the foreground would be multiplied by its own alpha first, which
    // over black gave 0.12 where Blender gives 0.2.
    same("alpha_over", |c| pixel::alpha_over(c, [0.2, 0.9, 0.4, 0.6], 1.0, false, pixel::OVER_PLAIN));
}

/// Every `MA_RAMP_*` blend mode, over the same second colour Blender used.
#[test]
fn every_mix_mode_matches_blender() {
    let b: BCompPixel = [0.3, 0.7, 0.45, 1.0];
    for (name, mode) in [
        ("mix", 0),
        ("add", 1),
        ("multiply", 2),
        ("subtract", 3),
        ("screen", 4),
        ("divide", 5),
        ("difference", 6),
        ("darken", 7),
        ("lighten", 8),
        ("overlay", 9),
        ("dodge", 10),
        ("burn", 11),
        ("hue", 12),
        ("saturation", 13),
        ("value", 14),
        ("color", 15),
        ("soft_light", 16),
        ("linear_light", 17),
        ("exclusion", 18),
    ] {
        same(&format!("mix_{name}"), |c| pixel::mix(mode, 0.65, c, b, false));
    }
}
