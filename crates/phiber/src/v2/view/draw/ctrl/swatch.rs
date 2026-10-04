//! A colour control's swatch: the square that shows the colour it names.
//!
//! All 29 colours on the Preferences Appearance page used to be black squares.
//! `ctrl/mod.rs` matched `BValue::Color(rgba)` and fell back to opaque black for
//! anything else — and a theme preference does not carry RGBA. It carries the
//! `"h, s, l, a"` text its `BStyleDescriptor` formats and parses back, because
//! that is the one spelling the descriptor will accept. So the number beside each
//! swatch was right and the swatch was black, in the one screen whose whole job
//! is choosing colours.
//!
//! Two spellings reach a swatch and both are real; the black was the third thing,
//! and it was a guess.

use crate::v2::state::val::BValue;
use crate::v2::theme::BColor;

/// A colour control's value as the swatch takes it: RGBA from a scene property,
/// HSL from a theme preference.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BCtrlSwatch {
    Rgba([f32; 4]),
    Hsl(BColor),
}

/// What "I cannot read this colour" looks like: black at half alpha.
///
/// Opaque black is a colour a theme could legitimately name, so it cannot double
/// as the absence of one — that ambiguity is exactly how 29 wrong swatches went
/// unnoticed. Half alpha is not a value any descriptor produces.
const UNREADABLE: BCtrlSwatch = BCtrlSwatch::Rgba([0.0, 0.0, 0.0, 0.5]);

impl BCtrlSwatch {
    /// `value` as a swatch, in whichever spelling it holds.
    ///
    /// The text form goes through the descriptor's own `parse_color`, not a second
    /// parser living here, so what the swatch shows cannot disagree with what the
    /// field beside it would accept.
    ///
    /// `parse_color` and not `parse_hsla`, because a theme colour has two
    /// spellings: `"h, s, l, a"` and a hex code, which the field accepts since
    /// Blender's picker has a Hex field and a hex is how a colour arrives from
    /// anywhere else. The narrower parser would have shown `#3D3D3D` — a colour the
    /// field takes and the resolver applies — as the half-black "unreadable"
    /// swatch, which is the same class of bug as the 29 black squares.
    pub fn of(value: &BValue) -> Self {
        match value {
            BValue::Color(rgba) => Self::Rgba(*rgba),
            BValue::Text(text) => crate::v2::theme::parse_color(text).map_or(UNREADABLE, Self::Hsl),
            _ => UNREADABLE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_preference_colour_reaches_the_swatch_as_its_colour() {
        // The form `BStyleDescriptor::format_value` writes for a colour.
        let value = BValue::Text("228, 24.36, 6.12, 1".to_string());
        let BCtrlSwatch::Hsl((h, s, l, a)) = BCtrlSwatch::of(&value) else {
            panic!("a formatted colour is read as a colour, not as `unreadable`")
        };
        assert!((h - 228.0).abs() < 0.01 && (s - 24.36).abs() < 0.01);
        assert!((l - 6.12).abs() < 0.01 && (a - 1.0).abs() < 0.01);
    }

    #[test]
    fn a_scene_colour_still_arrives_as_rgba() {
        let value = BValue::Color([0.25, 0.5, 0.75, 1.0]);
        assert_eq!(BCtrlSwatch::of(&value), BCtrlSwatch::Rgba([0.25, 0.5, 0.75, 1.0]));
    }

    #[test]
    fn what_cannot_be_read_does_not_look_like_a_colour() {
        // Not a colour at all, and text that is not one: both are the absence of
        // a colour, and neither may paint as opaque black — a theme can name that.
        for value in [BValue::Text("not a colour".to_string()), BValue::Bool(true), BValue::Text(String::new())] {
            let got = BCtrlSwatch::of(&value);
            assert_eq!(got, UNREADABLE, "{value:?}");
            let BCtrlSwatch::Rgba([.., alpha]) = got else { panic!("the fallback is plain rgba") };
            assert!(alpha < 1.0, "and it is not opaque, so it cannot pass for a real colour");
        }
    }
}
