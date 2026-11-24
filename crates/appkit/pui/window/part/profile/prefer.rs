//! What a person prefers of how the app appears, under the looks in the
//! profile's Appearance: a row of pills for each group of `asset/prefer.tsv`
//! (which way the light runs, how strong it is, how far it reaches), the
//! one in force lit, and the wheel the light's colour is chosen on. A
//! press puts the value over the look (`PShade::put`) and tells the app
//! `prefer` and all that is preferred, so it can keep it.

use crate::pui::block::wheel::{self, read, written};
use crate::pui::core::light::label;
use crate::pui::core::{PLight, PPaint, PPoint, PRect};
use crate::theme::{PShade, PTheme};

use super::{DIM, FILL, GAP, INK, LIT, PProfile, ROUND, ROW};

/// The kit's table of what can be preferred.
pub const PREFER: &str = include_str!("../../../../asset/prefer.tsv");
/// What a preference's words start with, in the user's table.
pub const WORD: &str = "prefer.";
/// The word the app is told, before all that is preferred.
pub const TOLD: &str = "prefer";
/// What a pill's place keeps, before the row of the table it chooses;
/// and before the colour a dot of the wheel chooses.
pub const ROW_OF: &str = "prefer\t";
pub const WHEEL: &str = "wheel\t";

/// A row of the table: its group, kind, tokens, value and word.
pub fn rows() -> Vec<[String; 5]> {
    let cell = |line: &str| -> Option<[String; 5]> {
        let c: Vec<&str> = line.split('\t').collect();
        Some([c.first()?.to_string(), c.get(1)?.to_string(), c.get(2)?.to_string(), c.get(3)?.to_string(), c.get(4)?.to_string()])
    };
    PREFER.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()).filter_map(cell).collect()
}

/// Whether a row is what is in force now.
fn now(shade: &PShade, row: &[String; 5]) -> bool {
    let token = row[2].split_whitespace().next().unwrap_or_default();
    match row[1].as_str() {
        "color" => shade.color(token).map(written) == Some(row[3].clone()),
        _ => shade.size(token).zip(row[3].parse::<f32>().ok()).is_some_and(|(now, value)| (now - value).abs() < 0.001),
    }
}

/// Put a row's value over the look, and what is kept of it now.
pub fn put(shade: &PShade, row: &[String; 5]) -> String {
    row[2].split_whitespace().for_each(|token| shade.put(&row[1], token, &row[3]));
    shade.kept()
}

impl PProfile {
    /// Draw the preferences from a place across a width: where they end,
    /// and what each place a press does keeps.
    pub(super) fn prefer(&self, shade: &PShade, paint: &mut dyn PPaint, left: f32, top: f32, wide: f32) -> (f32, Vec<(PRect, String)>) {
        let (gap, line, round, ink, dim) = (paint.size(GAP), paint.size(ROW), paint.size(ROUND), paint.color(INK), paint.color(DIM));
        let word = |name: &str, or: &str| Some(self.user.get(name)).filter(|word| !word.is_empty()).map(str::to_string).unwrap_or_else(|| or.to_string());
        let mut spot = Vec::new();
        let mut down = top;
        let all = rows();
        let mut group: Vec<&str> = all.iter().map(|row| row[0].as_str()).collect();
        group.dedup();
        for name in group {
            let row: Vec<(usize, &[String; 5])> = all.iter().enumerate().filter(|(_, row)| row[0] == name).collect();
            paint.text(PPoint::new(left, down), &word(&format!("{WORD}{name}"), name), dim);
            down += line + gap / 2.0;
            match row.first().map(|(_, row)| row[1].as_str()) {
                // A colour: its wheel, as wide as half the width.
                Some("color") => {
                    let token = row[0].1[2].split_whitespace().next().unwrap_or_default().to_string();
                    let side = (wide / 2.0).min(paint.size(ROW) * 8.0);
                    let chosen = shade.color(&token).map(written).unwrap_or_default();
                    let dots = wheel::wheel(paint, PRect::new(left, down, side, side), &chosen, ink);
                    spot.extend(dots.into_iter().map(|(rect, colour)| (rect, format!("{WHEEL}{}", row[0].0) + "\t" + &colour)));
                    down += side + gap;
                }
                _ => {
                    let mut x = left;
                    for (index, one) in &row {
                        let said = word(&one[4], &one[3]);
                        let long = paint.measure(&said) + gap * 2.0;
                        let rect = PRect::new(x, down, long, line + gap);
                        let (light, token) = match now(shade, one) {
                            true => (PLight::Strong, LIT),
                            false => (PLight::Soft, FILL),
                        };
                        label(paint, rect, round, light, false, token);
                        paint.text(PPoint::new(rect.origin.x + gap, rect.center().y - line / 2.0), &said, ink);
                        spot.push((rect, format!("{ROW_OF}{index}")));
                        x += long + gap / 2.0;
                    }
                    down += line + gap * 1.75;
                }
            }
        }
        (down, spot)
    }

    /// Do what a press on a preference's place does: what is kept of
    /// all that is preferred now, if it was one.
    pub(super) fn chose(shade: &PShade, kept: &str) -> Option<String> {
        let all = rows();
        match (kept.strip_prefix(ROW_OF), kept.strip_prefix(WHEEL)) {
            (Some(index), _) => all.get(index.parse::<usize>().ok()?).map(|row| put(shade, row)),
            (_, Some(rest)) => {
                let (index, colour) = rest.split_once('\t')?;
                read(colour)?;
                let mut row = all.get(index.parse::<usize>().ok()?)?.clone();
                row[3] = colour.to_string();
                Some(put(shade, &row))
            }
            _ => None,
        }
    }
}
