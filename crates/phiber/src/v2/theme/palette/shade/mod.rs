//! The shade ladder — every surface, line and letter of the interface is
//! one of eighteen shades of one seed. The seed of the default theme is
//! the sphere's mid-tone, `#3B4657`. Shades 1–10 are the seed at 10 %,
//! 20 %, … 100 % over black; shades 11–18 are white at 10 %, 20 %, … 80 %
//! over the seed. Nothing is nudged by a lightness delta: a colour moves
//! along the ladder in whole steps (`BShadeLadder::step`), or it is not a
//! ladder colour at all (the accent, the axes, the status green).
//!
//! Structure here; the reference table and its wording are `set.rs`.

pub mod seed;
pub mod set;

pub use seed::*;

use super::BColor;

/// Shades are numbered 1 ..= 18; the seed itself is shade 10.
pub const STEPS: u8 = 18;
pub const SEED_STEP: u8 = 10;
/// How near a colour must sit to a shade to count as that shade.
const HUE_TOLERANCE: f32 = 2.5;
const SATURATION_TOLERANCE: f32 = 3.0;
const LIGHTNESS_TOLERANCE: f32 = 1.5;

/// Eighteen shades of one seed. A dark theme reads the ladder up from the
/// seed's shadow to white text; a light theme reads the same eighteen
/// shades the other way — step 1 is the whitest, step 18 the darkest — so
/// every slot (`SHADE_BODY`, `SHADE_TEXT`, …) keeps its number and its
/// meaning, and "one step inward" is a step darker instead of lighter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BShadeLadder {
    pub seed: [u8; 3],
    pub light: bool,
}

/// `[r, g, b]` in 0 ..= 1 to `(hue°, saturation %, lightness %)`.
pub const fn hsl_of(rgb: [f32; 3]) -> (f32, f32, f32) {
    let [red, green, blue] = rgb;
    let max = if red > green { if red > blue { red } else { blue } } else if green > blue { green } else { blue };
    let min = if red < green { if red < blue { red } else { blue } } else if green < blue { green } else { blue };
    let lightness = (max + min) / 2.0;
    let chroma = max - min;
    if chroma < 1e-6 {
        return (0.0, 0.0, lightness * 100.0);
    }
    let spread = 1.0 - if 2.0 * lightness - 1.0 < 0.0 { 1.0 - 2.0 * lightness } else { 2.0 * lightness - 1.0 };
    let saturation = chroma / spread;
    let mut hue = if max == red {
        (green - blue) / chroma
    } else if max == green {
        (blue - red) / chroma + 2.0
    } else {
        (red - green) / chroma + 4.0
    };
    if hue < 0.0 {
        hue += 6.0;
    }
    (hue * 60.0, saturation * 100.0, lightness * 100.0)
}

impl BShadeLadder {
    /// Shade `step` (1 ..= 18) as `[r, g, b]` in 0 ..= 1, made as the table
    /// makes it: the seed over black up to 10, white over the seed past it —
    /// read from the white end for a light ladder.
    pub const fn rgb(&self, step: u8) -> [f32; 3] {
        let step = if step < 1 { 1 } else if step > STEPS { STEPS } else { step };
        let step = if self.light { STEPS + 1 - step } else { step };
        let seed = [self.seed[0] as f32 / 255.0, self.seed[1] as f32 / 255.0, self.seed[2] as f32 / 255.0];
        if step <= SEED_STEP {
            let opacity = step as f32 / SEED_STEP as f32;
            [seed[0] * opacity, seed[1] * opacity, seed[2] * opacity]
        } else {
            let white = (step - SEED_STEP) as f32 / SEED_STEP as f32;
            [seed[0] + (1.0 - seed[0]) * white, seed[1] + (1.0 - seed[1]) * white, seed[2] + (1.0 - seed[2]) * white]
        }
    }

    /// Shade `step` as the theme's `BColor` (h, s, l, 1).
    pub const fn color(&self, step: u8) -> BColor {
        let (hue, saturation, lightness) = hsl_of(self.rgb(step));
        (hue, saturation, lightness, 1.0)
    }

    /// Shade `step` at `alpha`.
    pub const fn at(&self, step: u8, alpha: f32) -> BColor {
        let (hue, saturation, lightness, _) = self.color(step);
        (hue, saturation, lightness, alpha)
    }

    /// The shade a colour is, when it is one — the nearest step within the
    /// tolerances; `None` for a colour off the ladder (an accent). Nearest,
    /// not first: a near-black ladder's steps sit closer together than the
    /// lightness tolerance, so the first step within it is not always the
    /// right one.
    pub fn step_of(&self, color: BColor) -> Option<u8> {
        (1..=STEPS)
            .filter_map(|step| {
                let (hue, saturation, lightness, _) = self.color(step);
                let hue_gap = (color.0 - hue).abs().min(360.0 - (color.0 - hue).abs());
                // Under 1 % saturation a hue means nothing.
                let hue_fits = hue_gap <= HUE_TOLERANCE || color.1 < 1.0 && saturation < 1.0;
                let saturation_gap = (color.1 - saturation).abs();
                let lightness_gap = (color.2 - lightness).abs();
                (hue_fits && saturation_gap <= SATURATION_TOLERANCE && lightness_gap <= LIGHTNESS_TOLERANCE)
                    .then_some((step, lightness_gap + saturation_gap))
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(step, _)| step)
    }

    /// `color` moved `by` steps along the ladder, keeping its alpha; `None`
    /// when it is not a ladder colour.
    pub fn step(&self, color: BColor, by: i32) -> Option<BColor> {
        let from = self.step_of(color)?;
        let to = (i32::from(from) + by).clamp(1, i32::from(STEPS)) as u8;
        Some(self.at(to, color.3))
    }

    /// `#rrggbb` of shade `step`, rounding ties to even as the table does.
    pub fn hex(&self, step: u8) -> String {
        let [red, green, blue] = self.rgb(step).map(|channel| (channel * 255.0).round_ties_even() as u8);
        format!("#{red:02X}{green:02X}{blue:02X}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sphere_ladder_is_the_table() {
        for (index, (hex, _)) in set::SPHERE_TABLE.iter().enumerate() {
            let step = index as u8 + 1;
            let ours = SPHERE.hex(step);
            let close = |a: &str, b: &str| {
                (0..3).all(|channel| {
                    let parse = |text: &str| u8::from_str_radix(&text[1 + channel * 2..3 + channel * 2], 16).unwrap();
                    (i16::from(parse(a)) - i16::from(parse(b))).abs() <= 1
                })
            };
            assert!(close(&ours, hex), "shade {step}: {ours} is not {hex}");
        }
        assert_eq!(SPHERE.hex(SEED_STEP), "#3B4657", "the seed is shade 10");
        assert_eq!(SPHERE.hex(0), SPHERE.hex(1), "steps clamp to the ladder");
        assert_eq!(SPHERE.hex(19), SPHERE.hex(18));
    }

    #[test]
    fn colours_on_the_ladder_are_found_and_stepped_and_accents_are_not() {
        let five = SPHERE.color(5);
        assert_eq!(SPHERE.step_of(five), Some(5));
        assert_eq!(SPHERE.step(five, 2), Some(SPHERE.color(7)));
        assert_eq!(SPHERE.step(five, -10), Some(SPHERE.color(1)), "steps clamp at the ends");
        assert_eq!(SPHERE.step(SPHERE.at(12, 0.5), 1), Some(SPHERE.at(13, 0.5)), "alpha rides along");
        assert_eq!(SPHERE.step_of((211.0, 60.0, 55.0, 1.0)), None, "the accent is not a shade");
        assert_eq!(SPHERE.step_of((0.0, 0.0, 90.0, 1.0)), None, "a plain grey is not a shade of a blue seed");
        let (hue, saturation, lightness) = hsl_of([0x3B as f32 / 255.0, 0x46 as f32 / 255.0, 0x57 as f32 / 255.0]);
        assert!((hue - 216.4).abs() < 0.5 && (saturation - 19.2).abs() < 0.5 && (lightness - 28.6).abs() < 0.2, "{hue} {saturation} {lightness}");
        assert_eq!(hsl_of([0.5, 0.5, 0.5]), (0.0, 0.0, 50.0));
        assert_eq!(GRAPHITE.color(SEED_STEP).1, 0.0, "Graphite's seed is Blender's neutral grey");
        for (ladder, hue) in [(TOKYO, 228.9), (SWIFT, 13.8), (CYBERPUNK, 265.3)] {
            assert!((ladder.color(SEED_STEP).0 - hue).abs() < 0.5, "{ladder:?} keeps its hue at the seed: {}", ladder.color(SEED_STEP).0);
        }
        // A light ladder is the same eighteen shades the other way up: its
        // step 2 (the window) is near white, its step 16 (text) dark.
        assert!(GREY_LIGHT.color(2).2 > 90.0 && GREY_LIGHT.color(16).2 < 30.0, "{:?} {:?}", GREY_LIGHT.color(2), GREY_LIGHT.color(16));
        assert_eq!(GREY_LIGHT.rgb(5), BShadeLadder::dark(GREY_LIGHT_SEED).rgb(14), "step 5 of a light ladder is shade 14 of the dark one");
        assert_eq!(GREY_LIGHT.step_of(GREY_LIGHT.color(7)), Some(7), "steps are found in step space, whichever way the ladder reads");
        assert_eq!(GREY_LIGHT.step(GREY_LIGHT.color(5), 1), Some(GREY_LIGHT.color(6)), "and a step inward is a step darker");
        // A dark seed lifted to light keeps its hue and lands near the silver
        // Grey's light mode uses.
        assert_eq!(lifted(SPHERE_SEED), [0xCE, 0xD1, 0xD5]);
        // The hue survives the lift within the rounding of an 8-bit seed
        // whose saturation is now a tenth of what it was.
        assert!((SPHERE_LIGHT.color(SEED_STEP).0 - SPHERE.color(SEED_STEP).0).abs() < 8.0, "the hue survives the lift: {} vs {}", SPHERE_LIGHT.color(SEED_STEP).0, SPHERE.color(SEED_STEP).0);
        assert!(SPHERE_LIGHT.color(2).2 > 90.0 && SPHERE_LIGHT.color(16).2 < 30.0);
    }
}
