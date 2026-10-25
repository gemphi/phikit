//! The scoreboard: a rounded slab hung over a court on four cables, its
//! four faces screens — or, over a pitch, one big screen above the east
//! stand — reading each side's code and score and the clock, in the
//! stroke face (`font`).
//!
//! The screen is geometry, not a texture, so it changes the way the rest
//! of the match does: every digit a slot can show is its own part, all
//! faces' copies of it in one mesh, and the match shows the one the score
//! (or the clock) names and hides the other nine (`BSlot::shows`).

use super::font::{self, TALL};
use super::mesh::BMesh;
use super::part::{BMat, BPart};
use super::ring::BRing;
use crate::choice::BChoice;
use crate::play::FPS;
use glam::{Mat4, Vec3};
use std::f32::consts::FRAC_PI_2;

/// A place on the screen a digit is shown in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSlot {
    HomeTens,
    HomeUnits,
    AwayTens,
    AwayUnits,
    Minutes,
    SecondsTens,
    SecondsUnits,
}

impl BSlot {
    pub const ALL: [BSlot; 7] = [BSlot::HomeTens, BSlot::HomeUnits, BSlot::AwayTens, BSlot::AwayUnits, BSlot::Minutes, BSlot::SecondsTens, BSlot::SecondsUnits];

    /// The digit the slot shows with `score` on the board at `frame`, or
    /// none (a score's tens below ten are blank). Scores past 99 show
    /// their last two digits; the clock runs up from the tip-off.
    pub fn shows(self, score: [u16; 2], frame: usize) -> Option<u8> {
        let seconds = (frame as f32 / FPS) as u32;
        let tens = |s: u16| (s >= 10).then_some(((s / 10) % 10) as u8);
        match self {
            BSlot::HomeTens => tens(score[0]),
            BSlot::HomeUnits => Some((score[0] % 10) as u8),
            BSlot::AwayTens => tens(score[1]),
            BSlot::AwayUnits => Some((score[1] % 10) as u8),
            BSlot::Minutes => Some(((seconds / 60) % 10) as u8),
            BSlot::SecondsTens => Some(((seconds % 60) / 10) as u8),
            BSlot::SecondsUnits => Some((seconds % 10) as u8),
        }
    }
}

/// One digit a slot can show: `value` drawn on every face.
#[derive(Clone, Debug)]
pub struct BDigit {
    pub slot: BSlot,
    pub value: u8,
    pub part: BPart,
}

/// The built board: the slab, its screens, the side codes and the colon
/// (`parts`), and every slot's ten digits (`digits`).
#[derive(Clone, Debug, Default)]
pub struct BBoard {
    pub parts: Vec<BPart>,
    pub digits: Vec<BDigit>,
}

/// A face of the board: where its screen's centre is, which way it looks
/// (the turn about z that brings −y round to its normal), and its size as
/// a multiple of the court board's screen.
#[derive(Clone, Copy)]
struct Face {
    centre: Vec3,
    heading: f32,
    scale: f32,
}

impl Face {
    /// Where something `out` off the screen at (`u`, `v`) metres of the
    /// court board's layout (across, up from the centre) sits on this face,
    /// scaled to it.
    fn at(&self, u: f32, v: f32, out: f32) -> Mat4 {
        Mat4::from_translation(self.centre) * Mat4::from_rotation_z(self.heading) * Mat4::from_scale(Vec3::splat(self.scale)) * Mat4::from_translation(Vec3::new(u, -out, v))
    }
}

/// The court board's screen, metres.
const SCREEN: [f32; 2] = [4.2, 2.2];
/// The top row — the codes and the clock — and the bottom row, the scores:
/// their glyphs' heights and their centres' heights.
const SMALL: f32 = 0.45;
const LARGE: f32 = 0.8;
const TOP: f32 = 0.6;
const LOW: f32 = -0.3;
/// Where each side's code and score are centred across the screen.
const SIDE: f32 = 1.3;
const WEIGHT: f32 = 0.2;
const GAP: f32 = 0.2;

/// How far apart glyphs `height` tall are set.
fn advance(height: f32) -> f32 {
    height / TALL * (1.0 + GAP)
}

pub fn board(choice: &BChoice) -> BBoard {
    let sport = choice.sport.spec();
    let theme = choice.theme.spec();
    let (home, away) = (choice.home.spec(), choice.away.spec());
    let [hx, _] = sport.field.floor_half();
    let mut out = BBoard::default();
    let frame = BMat::metal("Scoreboard", theme.apron.mix(crate::colour::INK, 0.7), 0.35);
    let screen = BMat::glowing("Scoreboard Screen", crate::colour::Rgb::hex(0x0b0d12), 0.3);
    let faces: Vec<Face> = if hx < 20.0 {
        // Hung over the court: a square slab rounded at its corners, a
        // screen on each side, on four cables to the roof.
        let r = 0.6;
        let side = SCREEN[0] + 2.0 * r;
        let (h, z) = (SCREEN[1] + 0.5, 10.5);
        let outline: Vec<[f32; 2]> = BRing::rounded([SCREEN[0] / 2.0; 2], r, 6).points().iter().map(|p| [p.x, p.y]).collect();
        // Extruded along +y from z − h/2; turned so that runs up +z.
        let slab = BMesh::extruded(&outline, h);
        out.parts.push(BPart::new("Scoreboard", slab, frame.clone(), Mat4::from_translation(Vec3::new(0.0, 0.0, z - h / 2.0)) * Mat4::from_rotation_x(FRAC_PI_2)));
        let mut cables = BMesh::default();
        for (x, y) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let foot = Vec3::new(x * SCREEN[0] / 2.0, y * SCREEN[0] / 2.0, z + h / 2.0);
            cables.merge(&BMesh::rod(foot, foot + Vec3::Z * 8.0, 0.025), Mat4::IDENTITY);
        }
        out.parts.push(BPart::new("Scoreboard Cables", cables, frame.clone(), Mat4::IDENTITY));
        let c = Vec3::new(0.0, 0.0, z);
        (0..4)
            .map(|k| {
                let heading = FRAC_PI_2 * k as f32;
                let normal = Vec3::new(heading.sin(), -heading.cos(), 0.0);
                Face { centre: c + normal * (side / 2.0), heading, scale: 1.0 }
            })
            .collect()
    } else {
        // Over a pitch: one big screen above the east stand, facing west,
        // on two legs.
        let scale = 2.6;
        let (x, z) = (hx + 6.0, 16.0);
        let [w, h] = SCREEN.map(|d| d * scale);
        out.parts.push(BPart::new("Scoreboard", BMesh::cuboid(Vec3::new(0.6, w + 0.6, h + 0.6)), frame.clone(), Mat4::from_translation(Vec3::new(x + 0.3, 0.0, z))));
        let mut legs = BMesh::default();
        for y in [-1.0, 1.0] {
            legs.merge(&BMesh::rod(Vec3::new(x + 0.3, y * w * 0.35, 0.0), Vec3::new(x + 0.3, y * w * 0.35, z - h / 2.0), 0.18), Mat4::IDENTITY);
        }
        out.parts.push(BPart::new("Scoreboard Legs", legs, frame.clone(), Mat4::IDENTITY));
        vec![Face { centre: Vec3::new(x, 0.0, z), heading: -FRAC_PI_2, scale }]
    };
    // Everything on a face, merged across the faces.
    let across = |m: &BMesh, u: f32, v: f32, out: f32| {
        let mut all = BMesh::default();
        for f in &faces {
            all.merge(m, f.at(u, v, out));
        }
        all
    };
    let pane = BMesh::cuboid(Vec3::new(SCREEN[0] * 0.98, 0.02, SCREEN[1]));
    out.parts.push(BPart::new("Scoreboard Screen", across(&pane, 0.0, 0.0, 0.01), screen, Mat4::IDENTITY));
    // The side codes in their colours over their scores.
    for (code, u, colour) in [(home.code, -SIDE, home.home), (away.code, SIDE, away.shirt(&home))] {
        let m = across(&font::mesh(code, SMALL, WEIGHT, GAP), u, TOP, 0.03);
        out.parts.push(BPart::new(format!("Scoreboard {code}"), m, BMat::glowing(format!("Scoreboard {code}"), colour, 4.0), Mat4::IDENTITY));
    }
    // The clock between them: m:ss, the colon two squares.
    let lit = BMat::glowing("Scoreboard Digits", theme.glow.mix(crate::colour::WHITE, 0.6), 5.0);
    let dot = BMesh::cuboid(Vec3::new(SMALL * 0.12, 0.01, SMALL * 0.12));
    let mut colon = BMesh::default();
    for v in [TOP - SMALL * 0.2, TOP + SMALL * 0.2] {
        colon.merge(&across(&dot, -0.62 * advance(SMALL), v, 0.03), Mat4::IDENTITY);
    }
    out.parts.push(BPart::new("Scoreboard Colon", colon, lit.clone(), Mat4::IDENTITY));
    for slot in BSlot::ALL {
        let (u, v, height) = match slot {
            BSlot::HomeTens => (-SIDE - 0.5 * advance(LARGE), LOW, LARGE),
            BSlot::HomeUnits => (-SIDE + 0.5 * advance(LARGE), LOW, LARGE),
            BSlot::AwayTens => (SIDE - 0.5 * advance(LARGE), LOW, LARGE),
            BSlot::AwayUnits => (SIDE + 0.5 * advance(LARGE), LOW, LARGE),
            BSlot::Minutes => (-1.3 * advance(SMALL), TOP, SMALL),
            BSlot::SecondsTens => (0.0, TOP, SMALL),
            BSlot::SecondsUnits => (advance(SMALL), TOP, SMALL),
        };
        for value in 0..10u8 {
            let m = across(&font::mesh(&value.to_string(), height, WEIGHT, GAP), u, v, 0.03);
            out.digits.push(BDigit { slot, value, part: BPart::new(format!("Scoreboard {slot:?} {value}"), m, lit.clone(), Mat4::IDENTITY) });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sport::BSport;

    #[test]
    fn the_board_shows_the_score_and_the_clock() {
        // 23–7 at 1:05: the home tens and units, a blank away tens, the
        // clock's three digits.
        let shown: Vec<Option<u8>> = BSlot::ALL.iter().map(|s| s.shows([23, 7], (65.0 * FPS) as usize)).collect();
        assert_eq!(shown, vec![Some(2), Some(3), None, Some(7), Some(1), Some(0), Some(5)]);
        for sport in BSport::ALL {
            let b = board(&BChoice { sport, ..BChoice::default() });
            assert_eq!(b.digits.len(), 70, "{sport:?}");
            // High enough to play under, and every digit drawn on every face.
            let low = b.parts.iter().flat_map(|p| p.mesh.positions.iter().map(|q| p.at.transform_point3(Vec3::from(*q)).z)).fold(f32::MAX, f32::min);
            assert!(low > 7.0 || sport == BSport::Soccer, "{sport:?}: the board's foot at {low}");
            let one = font::mesh("8", 1.0, WEIGHT, GAP).triangle_count();
            let faces = if sport == BSport::Soccer { 1 } else { 4 };
            let eight = b.digits.iter().find(|d| d.value == 8).unwrap();
            assert_eq!(eight.part.mesh.triangle_count(), one * faces, "{sport:?}");
        }
    }
}
