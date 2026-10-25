//! The field: the floor under the players and its texture — the turf's
//! grain, the sport's painted areas, a bloom of the theme's colours at the
//! centre of an indoor court, and the lines, every one from the sport's
//! markings (`field::BMark`), antialiased.
//!
//! One image covers the whole floor, run-off included, at up to 40 pixels
//! a metre and never more than 2048 across: a court gets 40 px/m, a soccer
//! pitch 17.5 (its lines are 12 cm, two pixels).

use super::image::BImage;
use super::mesh::BMesh;
use super::part::{BMat, BPart};
use crate::colour::Rgb;
use crate::field::BFieldSpec;
use crate::sport::BSportSpec;
use crate::theme::{BRole, BThemeSpec};
use crate::turf::{BGrain, BTurfSpec};
use glam::{Mat4, Vec3};
use std::sync::Arc;

const MAX_PX_PER_M: f32 = 40.0;
const MAX_SIDE: f32 = 2048.0;

/// The floor, painted.
pub fn floor(sport: &BSportSpec, turf: &BTurfSpec, theme: &BThemeSpec) -> BPart {
    let [fx, fy] = sport.field.floor_half();
    let image = Arc::new(texture(sport, turf, theme));
    let mut mesh = BMesh::default();
    mesh.quad(Vec3::new(-fx, -fy, 0.0), Vec3::new(fx, -fy, 0.0), Vec3::new(fx, fy, 0.0), Vec3::new(-fx, fy, 0.0));
    let mut mat = BMat::plain(format!("{} Floor", sport.label), turf.base, turf.roughness).with_image(image);
    if turf.emissive {
        // The glass floor lights its own tiles, at a strength that leaves
        // the lines the brightest thing on it.
        mat.emission = Rgb(1.0, 1.0, 1.0);
        mat.strength = 0.6;
    }
    BPart::new("Floor", mesh, mat, Mat4::IDENTITY)
}

/// Pixels a metre for a field this size.
pub fn resolution(field: &BFieldSpec) -> f32 {
    let [fx, _] = field.floor_half();
    MAX_PX_PER_M.min(MAX_SIDE / (2.0 * fx))
}

/// The floor's texture: row 0 is the +y edge, column 0 the −x edge.
pub fn texture(sport: &BSportSpec, turf: &BTurfSpec, theme: &BThemeSpec) -> BImage {
    let field = &sport.field;
    let [fx, fy] = field.floor_half();
    let ppm = resolution(field);
    let (w, h) = ((2.0 * fx * ppm) as u32, (2.0 * fy * ppm) as u32);
    let pixel = 1.0 / ppm;
    // Outdoors the run-off is the turf itself; indoors it is the theme's apron.
    let natural = matches!(turf.grain, BGrain::Mown { .. } | BGrain::Chequer { .. } | BGrain::Ripples) || turf.label == "Grass";
    BImage::painted(w, h, |u, v| {
        let p = [-fx + u * 2.0 * fx, fy - v * 2.0 * fy];
        let on = field.contains(p);
        let mut c = if on || natural { grain(turf, p) } else { theme.apron };
        for paint in field.paint {
            if p[0] >= paint.min[0] && p[0] <= paint.max[0] && p[1] >= paint.min[1] && p[1] <= paint.max[1] {
                c = c.mix(theme.colour(paint.role), 0.75);
            }
        }
        if sport.indoor && on {
            if let Some(ring) = bloom(p) {
                c = c.mix(theme.colour(ring), 0.8);
            }
        }
        let d = field.marks.iter().map(|m| m.distance(p)).fold(f32::MAX, f32::min);
        let cover = ((field.line / 2.0 - d) / pixel + 0.5).clamp(0.0, 1.0);
        c.mix(theme.line, cover)
    })
}

/// The centre bloom: softly squarish rings, outermost first in the
/// theme's colours, as the demo court's was.
fn bloom(p: [f32; 2]) -> Option<BRole> {
    let r = squircle(p[0], p[1], 3.2, 1.15);
    const RINGS: [(f32, BRole); 3] = [(1.4, BRole::Secondary), (1.0, BRole::Accent), (0.6, BRole::Primary)];
    RINGS.iter().find(|(radius, _)| (r - radius).abs() < 0.18).map(|(_, role)| *role)
}

/// A superellipse's radius: a circle at power 2, squarer above it, `wide`
/// times longer across x than y — the phiegg's softly squarish oval.
pub fn squircle(x: f32, y: f32, n: f32, wide: f32) -> f32 {
    ((x / wide).abs().powf(n) + y.abs().powf(n)).powf(1.0 / n)
}

/// The turf at a point of the floor.
pub fn grain(turf: &BTurfSpec, p: [f32; 2]) -> Rgb {
    let (base, alt) = (turf.base, turf.alt);
    match turf.grain {
        BGrain::Planks { width } => {
            // Boards along the length, their joints staggered board by
            // board, each board a shade of its own.
            let board = (p[1] / width).floor() as i32;
            let offset = hash(board, 7) * 3.0;
            let along = (p[0] + offset).rem_euclid(3.0);
            let tone = (hash(board, 3) - 0.5) * 0.08;
            let seam = (p[1] / width).fract() < 0.03 || along < 0.012;
            let wood = base.mix(alt, 0.5 + tone * 4.0);
            let grainy = 0.03 * (p[0] * 9.0 + 5.0 * (p[1] * 7.0 + board as f32).sin()).sin();
            let wood = Rgb(wood.0 * (1.0 + grainy), wood.1 * (1.0 + grainy), wood.2 * (1.0 + grainy));
            if seam { wood.mix(Rgb(0.0, 0.0, 0.0), 0.12) } else { wood }
        }
        BGrain::Speckle => base.mix(alt, hash((p[0] * 50.0) as i32, (p[1] * 50.0) as i32)),
        BGrain::Tiles { size } => {
            let (i, j) = ((p[0] / size).floor() as i32, (p[1] / size).floor() as i32);
            let edge = (p[0] / size).fract().min(1.0 - (p[0] / size).fract()).min((p[1] / size).fract().min(1.0 - (p[1] / size).fract()));
            if edge < 0.02 { alt.mix(Rgb(0.0, 0.0, 0.0), 0.5) } else { base.mix(alt, hash(i, j)) }
        }
        BGrain::Mown { width } => if ((p[0] / width).floor() as i32).rem_euclid(2) == 0 { base } else { alt },
        BGrain::Chequer { size } => {
            let (i, j) = ((p[0] / size).floor() as i32, (p[1] / size).floor() as i32);
            if (i + j).rem_euclid(2) == 0 { base } else { alt }
        }
        BGrain::Ripples => {
            let wave = 0.5 + 0.5 * (p[0] * 2.3 + 0.8 * (p[1] * 0.9).sin() + 1.7 * (p[1] * 0.37).cos()).sin();
            base.mix(alt, wave * 0.7 + 0.3 * hash((p[0] * 30.0) as i32, (p[1] * 30.0) as i32))
        }
    }
}

/// A repeatable 0–1 value for an integer cell: the generators' noise,
/// with no random generator to seed.
pub fn hash(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h = (h ^ (h >> 13)).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    (h & 0xffff) as f32 / 65535.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sport::BSport;
    use crate::theme::BTheme;
    use crate::turf::BTurf;

    #[test]
    fn the_lines_land_where_the_rulebook_draws_them() {
        let sport = BSport::Basketball.spec();
        let theme = BTheme::Studio.spec();
        let image = texture(sport, BTurf::Maple.spec(), theme);
        let ppm = resolution(&sport.field);
        assert_eq!(ppm, 40.0);
        let [fx, fy] = sport.field.floor_half();
        let px = |x: f32, y: f32| image.at(((x + fx) * ppm) as u32, ((fy - y) * ppm) as u32);
        // On the halfway line, the line's colour; half a metre off it, wood.
        let line = theme.line;
        let close = |a: Rgb, b: Rgb| (a.0 - b.0).abs() + (a.1 - b.1).abs() + (a.2 - b.2).abs() < 0.08;
        assert!(close(px(0.0, 3.0), line), "the halfway line");
        assert!(!close(px(0.5, 3.0), line), "beside it");
        // The three-point arc at the top of the key, 6.75 m from the basket.
        assert!(close(px(14.0 - 1.575 - 6.75, 0.0), line), "the arc's apex");
        // The key is painted toward the primary colour; the wood beside it
        // is not.
        let off = |a: Rgb, b: Rgb| (a.0 - b.0).abs() + (a.1 - b.1).abs() + (a.2 - b.2).abs();
        assert!(off(px(12.0, 1.0), theme.primary) < off(px(12.0, 4.0), theme.primary) / 2.0, "the key is painted");
        // Outside the lines, the apron.
        assert!(close(px(15.5, 0.0), theme.apron), "the apron indoors");
        let pitch = BSport::Soccer.spec();
        assert!(resolution(&pitch.field) < 18.0 && texture(pitch, BTurf::Stripes.spec(), theme).width <= 2048);
    }
}
