//! The hex spelling of a colour — `#RRGGBB` / `#RRGGBBAA` — beside `parse_hsla`,
//! which is the other spelling a theme colour has.
//!
//! Ported from Blender 5.2.0 in `gemphi/ext/phiber-refs/blender`, not invented:
//! `blenlib/intern/math_color.cc:202` `hex_to_rgba` is the parser, and
//! `editors/interface/regions/interface_region_color_picker.cc:260-283` is the
//! formatter — `#%02X%02X%02X%02X` when the colour has an alpha channel,
//! `#%02X%02X%02X` when it does not.
//!
//! What transfers literally is the whole of the parser, which is more generous
//! than it looks: eight accepted lengths, a leading `#` optional, and a string it
//! cannot read leaves the colour alone rather than resetting it. Shorthands expand
//! by nibble-doubling (`#abc` → `#aabbcc`), which is why `#f` is white and not
//! `#0f0f0f`. Those are the cases this file tests, because a hex parser is exactly
//! the kind of thing that looks right and is wrong at the third digit.
//!
//! Two things do not transfer, both recorded rather than silently dropped:
//!
//! 1. Blender converts sRGB ↔ scene-linear around the hex (`is_color_gamma_picker`,
//!    `IMB_colormanagement_*`), because a Blender colour is scene-linear while a hex
//!    code pasted from a browser or another app is sRGB. phiber's theme colours have
//!    no colour management and no linear space — a `BColor` is already the sRGB
//!    colour the screen shows — so there is nothing to convert, and converting
//!    anyway would make a pasted `#3D3D3D` come out as a different grey than the
//!    one it names.
//! 2. Blender stores RGB and phiber stores HSL, so the round trip goes through
//!    `gpui`'s conversions rather than straight into the channels. That is lossy in
//!    the last decimal, which is why `format` re-rounds through bytes and why the
//!    tests below assert bytes rather than HSL floats.

use super::BColor;
use gpui::{Hsla, Rgba};

/// `color` as `#RRGGBBAA`, or `#RRGGBB` when it is fully opaque.
///
/// Blender decides the two forms by `cpicker->has_alpha` — whether the property it
/// is editing carries an alpha channel at all. Every theme colour here does, so
/// that question does not exist; what a reader actually wants to know is whether
/// THIS colour is transparent, and an opaque one spelled `#3D3D3DFF` makes the
/// common case the noisy one. So opacity decides it, and a round trip through
/// `parse` is unaffected either way: the parser preserves the alpha of a colour
/// whose hex does not state one.
pub fn format_hex(color: BColor) -> String {
    let [r, g, b, a] = bytes_of(color);
    if a == u8::MAX {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

/// `raw` read as a hex colour, keeping `current`'s alpha when the text does not
/// state one — `None` when it is not a hex colour at all, so the caller leaves the
/// colour as it was.
///
/// Every length is Blender's `hex_to_rgba`, case for case — the match below is its
/// switch, arm for arm, and `every_length_blenders_parser_accepts` is the same list
/// as data.
///
/// The odd ones are Blender's and are kept deliberately. Seven digits shifts the
/// last nibble UP (`a <<= 4`), so `#112233f` is alpha `0xF0`, not `0x0F`. Two
/// digits does NOT double the nibbles — it repeats the whole byte, so `#1A` is
/// `#1A1A1A` while one digit doubles the nibble and `#1` is `#111111`.
pub fn parse_hex(raw: &str, current: BColor) -> Option<BColor> {
    let text = raw.trim();
    let digits = text.strip_prefix('#').unwrap_or(text);
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    // Each nibble on its own, so the groupings below are just slicing.
    let n: Vec<u32> = digits.chars().map(|c| c.to_digit(16).unwrap_or(0)).collect();
    let byte = |hi: u32, lo: u32| (hi << 4) | lo;
    let twice = |v: u32| (v << 4) | v;
    let (r, g, b, a) = match n.len() {
        8 => (byte(n[0], n[1]), byte(n[2], n[3]), byte(n[4], n[5]), Some(byte(n[6], n[7]))),
        // `a <<= 4` in Blender: the single alpha nibble is the HIGH one.
        7 => (byte(n[0], n[1]), byte(n[2], n[3]), byte(n[4], n[5]), Some(n[6] << 4)),
        6 => (byte(n[0], n[1]), byte(n[2], n[3]), byte(n[4], n[5]), None),
        5 => (twice(n[0]), twice(n[1]), twice(n[2]), Some(byte(n[3], n[4]))),
        4 => (twice(n[0]), twice(n[1]), twice(n[2]), Some(twice(n[3]))),
        3 => (twice(n[0]), twice(n[1]), twice(n[2]), None),
        // Blender repeats the whole BYTE here, not the nibble.
        2 => {
            let v = byte(n[0], n[1]);
            (v, v, v, None)
        }
        1 => {
            let v = twice(n[0]);
            (v, v, v, None)
        }
        // "Invalid hex color length - leave color unchanged."
        _ => return None,
    };
    let alpha = a.map_or(current.3, |a| a as f32 / 255.0);
    Some(color_of([r as u8, g as u8, b as u8], alpha))
}

/// A colour's channels as bytes, the way the formatter needs them.
fn bytes_of(color: BColor) -> [u8; 4] {
    let rgba: Rgba = Hsla {
        h: color.0 / 360.0,
        s: color.1 / 100.0,
        l: color.2 / 100.0,
        a: color.3,
    }
    .into();
    // `round` rather than truncate: 0.5 of a step is nearer the next byte, and
    // truncating made every channel of a round trip drift one byte darker.
    [rgba.r, rgba.g, rgba.b, rgba.a].map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// Bytes back to a `BColor`, in the degrees-and-percent the descriptor parses.
fn color_of([r, g, b]: [u8; 3], alpha: f32) -> BColor {
    let hsla: Hsla = Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: alpha,
    }
    .into();
    (hsla.h * 360.0, hsla.s * 100.0, hsla.l * 100.0, alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What `parse_hex` makes of `raw`, as bytes — HSL floats round-trip with noise
    /// in the last decimal and the claim here is about the digits, not the float.
    fn parsed(raw: &str) -> Option<[u8; 4]> {
        parse_hex(raw, (0.0, 0.0, 0.0, 1.0)).map(bytes_of)
    }

    #[test]
    fn every_length_blenders_parser_accepts() {
        // Each case is one arm of `hex_to_rgba`'s switch, with the expansion that
        // arm performs. Alpha is 255 where the form does not state one, because
        // `current` above is opaque.
        for (raw, want) in [
            ("#11223344", [0x11, 0x22, 0x33, 0x44]),
            // Seven digits: the lone alpha nibble shifts UP, so `f` is 0xF0.
            ("#112233f", [0x11, 0x22, 0x33, 0xF0]),
            ("#112233", [0x11, 0x22, 0x33, 0xFF]),
            // Five: RGB doubles per nibble, the last two are one alpha byte.
            ("#12345", [0x11, 0x22, 0x33, 0x45]),
            ("#1234", [0x11, 0x22, 0x33, 0x44]),
            ("#123", [0x11, 0x22, 0x33, 0xFF]),
            // Two repeats the BYTE; one doubles the NIBBLE. Not the same rule.
            ("#1A", [0x1A, 0x1A, 0x1A, 0xFF]),
            ("#1", [0x11, 0x11, 0x11, 0xFF]),
        ] {
            assert_eq!(parsed(raw), Some(want), "{raw}");
        }
    }

    #[test]
    fn the_hash_is_optional_and_case_does_not_matter() {
        assert_eq!(parsed("3d3d3d"), parsed("#3D3D3D"));
        assert_eq!(parsed("#aabbcc"), parsed("#AABBCC"));
    }

    #[test]
    fn a_string_that_is_not_a_hex_colour_leaves_the_colour_alone() {
        let current = (210.0, 40.0, 60.0, 0.5);
        // Blender's default arm: "Invalid hex color length - leave color unchanged."
        for raw in ["", "#", "#123456789", "#xyzxyz", "not a colour", "#12 34 56"] {
            assert_eq!(parse_hex(raw, current), None, "{raw:?} is not a hex colour");
        }
    }

    #[test]
    fn a_form_without_an_alpha_keeps_the_one_it_was_given() {
        // The reason Blender pre-fills `rgba` from the property before parsing:
        // typing `#FF0000` into a half-transparent colour should change the hue,
        // not silently make it opaque.
        let half = (0.0, 0.0, 0.0, 0.5);
        for raw in ["#112233", "#123", "#1A", "#1"] {
            let (.., a) = parse_hex(raw, half).expect(raw);
            assert_eq!(a, 0.5, "{raw} states no alpha, so the colour keeps its own");
        }
        // And a form that DOES stay its alpha overrides it.
        let (.., a) = parse_hex("#11223380", half).expect("eight digits");
        assert!((a - 0x80 as f32 / 255.0).abs() < 1e-6, "{a}");
    }

    #[test]
    fn opacity_decides_which_form_is_written() {
        assert_eq!(format_hex((0.0, 0.0, 0.0, 1.0)), "#000000");
        assert_eq!(format_hex((0.0, 0.0, 100.0, 1.0)), "#FFFFFF");
        // Transparent says so rather than hiding it.
        assert_eq!(format_hex((0.0, 0.0, 0.0, 0.0)), "#00000000");
    }

    #[test]
    fn a_colour_survives_the_round_trip_through_its_hex() {
        // The trip is HSL → RGB → bytes → RGB → HSL, so the claim has to be about
        // the bytes: two HSL triples can name the same byte and the hex is the
        // byte. What must not happen is drift — the same colour spelled, read and
        // spelled again landing somewhere else.
        for color in [
            (0.0, 0.0, 15.7, 1.0),
            (210.0, 40.0, 60.0, 1.0),
            (33.0, 10.0, 20.0, 0.25),
            (359.0, 100.0, 50.0, 1.0),
            (228.94, 24.36, 6.12, 1.0),
        ] {
            let once = format_hex(color);
            let back = parse_hex(&once, color).expect(&once);
            assert_eq!(format_hex(back), once, "{color:?} drifted through {once}");
            // The alpha comes back exactly for an opaque colour, because its hex
            // omits the alpha and the parser keeps the one it was given. A
            // transparent colour spells its alpha out, so that one is a byte and
            // comes back quantised — 0.25 → `40` → 0.25098. Blender's eight-digit
            // arm does the same; it is the cost of the hex form, not a drift, and
            // spelling it again gives the same digits.
            if color.3 == 1.0 {
                assert_eq!(back.3, color.3, "{color:?} lost its alpha");
            } else {
                assert!((back.3 - color.3).abs() <= 1.0 / 255.0, "{color:?} → {} → {}", once, back.3);
            }
        }
    }

    #[test]
    fn what_it_writes_is_what_the_descriptor_accepts() {
        // The one rule every picker write obeys: a colour it produces must be a
        // colour the field beside it would take back.
        for raw in ["#3D3D3D", "#1", "#12345678", "#abc"] {
            let color = parse_hex(raw, (0.0, 0.0, 0.0, 1.0)).expect(raw);
            let text = format!("{}, {}, {}, {}", color.0, color.1, color.2, color.3);
            assert!(super::super::parse_hsla(&text).is_ok(), "{raw} became {text:?}, which the field would reject");
        }
    }
}
