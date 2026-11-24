//! A colour wheel: every hue round a ring and, from its rim to its
//! middle, each paler, as round dots on the font's basis (each a pill
//! as wide as it is high), the way a system's colour picker offers any
//! colour. A press on a dot chooses its colour; the one chosen is
//! ringed. A colour is said as `#rrggbb`, as a theme's table writes it.

use super::super::core::{PColor, PPaint, PRect};

/// How many hues round the wheel, and how many rings from its rim in.
pub const HUE: usize = 24;
pub const RING: usize = 5;

/// A colour of a hue (0 to 1 round the wheel), as full as `full` (0 is
/// white, 1 the hue itself).
pub fn hue(turn: f32, full: f32) -> PColor {
    let h = (turn.rem_euclid(1.0)) * 6.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    let pale = |c: f32| 1.0 - full * (1.0 - c);
    PColor::new(pale(r), pale(g), pale(b), 1.0)
}

/// A colour as a theme's table writes it.
pub fn written(colour: PColor) -> String {
    let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", byte(colour.r), byte(colour.g), byte(colour.b))
}

/// A colour as a theme's table writes it, read.
pub fn read(text: &str) -> Option<PColor> {
    let hex = text.strip_prefix('#').filter(|hex| hex.len() == 6)?;
    let byte = |at: usize| u8::from_str_radix(hex.get(at..at + 2)?, 16).ok().map(|b| b as f32 / 255.0);
    Some(PColor::new(byte(0)?, byte(2)?, byte(4)?, 1.0))
}

/// The dots of a wheel in a square: where each stands, and its colour,
/// from the rim's top round, then each ring further in.
pub fn dots(square: PRect) -> Vec<(PRect, PColor)> {
    let middle = square.center();
    let reach = square.size.width.min(square.size.height) / 2.0;
    let mut out = Vec::new();
    for ring in 0..RING {
        let away = reach * (1.0 - ring as f32 / RING as f32) - reach / RING as f32 / 2.0;
        let count = (HUE * (RING - ring) / RING).max(6);
        let dot = (std::f32::consts::TAU * away / count as f32 * 0.8).min(reach / RING as f32 * 0.9);
        for step in 0..count {
            let turn = step as f32 / count as f32;
            let angle = turn * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            let colour = hue(turn, 1.0 - ring as f32 / RING as f32 * 0.85);
            out.push((PRect::new(middle.x + away * angle.cos() - dot / 2.0, middle.y + away * angle.sin() - dot / 2.0, dot, dot), colour));
        }
    }
    out
}

/// Every colour a wheel offers, as written, in the order of its dots.
pub fn colours() -> Vec<String> {
    dots(PRect::new(0.0, 0.0, 100.0, 100.0)).into_iter().map(|(_, colour)| written(colour)).collect()
}

/// Draw a wheel in a square, the colour chosen ringed in the ink: where
/// each dot stands, and its colour as written.
pub fn wheel(paint: &mut dyn PPaint, square: PRect, chosen: &str, ink: PColor) -> Vec<(PRect, String)> {
    dots(square).into_iter().map(|(rect, colour)| {
        let word = written(colour);
        if word == chosen {
            paint.round(rect.inset(-2.0), rect.size.width, ink);
        }
        paint.round(rect, rect.size.width / 2.0, colour);
        paint.press(rect);
        (rect, word)
    }).collect()
}
