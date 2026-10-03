//! The Welcome backdrop is a surface, so it reads the theme. The mark on it is
//! a mark, so it does not.
//!
//! The backdrop was five frozen hexes — a ground and four SVG layers — and what
//! they were frozen to was Sphere's dark ladder. So the Welcome screen was a
//! dark slab under every other palette, and under a light one; and a light one
//! is now the mode a fresh window opens in (`BThemeMode::DEFAULT`), which made
//! it the first thing anybody would see. `brand.rs:jobs` asked whether a brand
//! mark should follow the theme. This is the answer, and it is two answers,
//! because there are two things there:
//!
//! - the MARK keeps its own colours in every palette, which is what
//!   `brand_warm` and `brand_cool` are for. A logo is a logo.
//! - the BACKDROP is the wall the mark hangs on, and a wall is a surface.
//!
//! Nothing about the artwork changed. The layers were already ladder rungs —
//! the doc on them said so in words, "cool 40%, warm 60%, join 40%, line 30%
//! — on a 30% ground" — they just could not move off the one theme. They are
//! those rungs by number now, and `shades_at_sphere_dark_are_the_old_hexes`
//! below is what proves the picture is unchanged where it used to be drawn.

use phiber::v2::theme::{BPaletteName, BThemeMode, palette::shade::STEPS, ui_style_defaults_in};

/// The rungs `brand.rs` names, kept here so the test states the contract
/// rather than reading it back out of the file it is checking.
const GROUND: u8 = 3;
const LAYERS: [u8; 4] = [6, 8, 6, 5];

fn rgb_of(color: phiber::v2::theme::BColor) -> u32 {
    let (h, s, l) = (color.0 / 360.0, color.1 / 100.0, color.2 / 100.0);
    // HSL → RGB, the same conversion the ladder inverts.
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h * 6.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let byte = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u32;
    (byte(r) << 16) | (byte(g) << 8) | byte(b)
}

/// The picture is the same picture, where it used to be drawn: Sphere, dark.
#[test]
fn shades_at_sphere_dark_are_the_old_hexes() {
    let ladder = ui_style_defaults_in(BPaletteName::Sphere, BThemeMode::Dark).colors.ladder;
    let near = |step: u8, want: u32| {
        let got = rgb_of(ladder.color(step));
        let channel = |v: u32, n: u32| ((v >> (8 * n)) & 0xff) as i32;
        for n in 0..3 {
            assert!(
                (channel(got, n) - channel(want, n)).abs() <= 1,
                "step {step} is #{got:06X}, and the frozen value was #{want:06X}"
            );
        }
    };
    near(GROUND, 0x12151A);
    near(LAYERS[0], 0x232A34);
    near(LAYERS[1], 0x2F3846);
    near(LAYERS[2], 0x232A34);
    near(LAYERS[3], 0x1E232C);
}

/// And it is a picture in every theme and both modes, not a dark slab in all
/// but one of them.
#[test]
fn the_backdrop_moves_with_the_theme_and_its_mode() {
    let mut seen = std::collections::BTreeSet::new();
    for name in BPaletteName::ALL.iter().copied() {
        for mode in BThemeMode::ALL {
            let ladder = ui_style_defaults_in(name, mode).colors.ladder;
            let ground = ladder.color(GROUND);
            let layers: Vec<_> = LAYERS.iter().map(|s| ladder.color(*s)).collect();
            seen.insert(format!("{:?}", rgb_of(ground)));

            // Every rung is in range, so none of them is silently clamped.
            for step in [GROUND].iter().chain(LAYERS.iter()) {
                assert!(*step >= 1 && *step <= STEPS, "{name:?}/{mode:?}: step {step} is on the ladder");
            }
            // The layers stand clear of the ground they sit on — the texture
            // has to be visible, which is the whole point of it.
            for (n, layer) in layers.iter().enumerate() {
                assert!(
                    (layer.2 - ground.2).abs() >= 1.0,
                    "{name:?}/{mode:?}: layer {n} at L={} is lost in a ground at L={}",
                    layer.2,
                    ground.2
                );
            }
            // Light mode reads the same rungs from the other end, so a light
            // theme's backdrop is light. That is the bug this fixes.
            let light = mode == BThemeMode::Light;
            assert_eq!(
                ground.2 > 50.0,
                light,
                "{name:?}/{mode:?}: the ground is {} at L={}",
                if light { "meant to be light" } else { "meant to be dark" },
                ground.2
            );
        }
    }
    // Twenty-two theme-and-mode pairs, and not one ground shared by all of
    // them — which is what a frozen hex would have given.
    assert!(seen.len() > 10, "the ground really does move: {} distinct values", seen.len());
}

/// The mark itself is untouched: its two colours are fixed on purpose.
#[test]
fn the_mark_keeps_its_own_colours() {
    let first = ui_style_defaults_in(BPaletteName::Sphere, BThemeMode::Dark).colors;
    for name in BPaletteName::ALL.iter().copied() {
        for mode in BThemeMode::ALL {
            let c = ui_style_defaults_in(name, mode).colors;
            assert_eq!(c.brand_warm, first.brand_warm, "{name:?}/{mode:?}: a logo keeps its colours");
            assert_eq!(c.brand_cool, first.brand_cool, "{name:?}/{mode:?}: both of them");
        }
    }
}

/// And the source says it in rungs, so the next person cannot freeze it again
/// without noticing.
#[test]
fn the_backdrop_is_written_as_rungs_not_as_hexes() {
    let src = include_str!("../src/v2/view/draw/brand.rs");
    assert!(src.contains("const BACKDROP_GROUND: u8"), "the ground is a step");
    assert!(src.contains("const BACKDROP_LAYERS: [(&str, u8); 4]"), "and so are the layers");
    assert!(!src.contains("BACKDROP_BG"), "the frozen ground is gone");
    for frozen in ["0x12151A", "0x232A34", "0x2F3846", "0x1E232C"] {
        assert!(!src.contains(frozen), "{frozen} is gone from the source");
    }
    assert!(src.contains("ladder.color(BACKDROP_GROUND)"), "the ground is painted from the ladder");
}
