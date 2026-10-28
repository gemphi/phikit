//! A venue's light: lamps over the field (a grid of area lights under a
//! roof, the sun outdoors), the façade's light groups — strips round the
//! shell, each its own material, lit by the venue's show — and, round an
//! indoor court, LED boards with the theme's squarish blooms on them.

use super::super::image::BImage;
use super::super::mesh::BMesh;
use super::super::part::{BMat, BPart};
use super::bowl::BBowl;
use super::shell::{base, height, profile};
use super::{BLamp, BLightGroup, BVenueBuild};
use crate::build::field::squircle;
use crate::choice::BChoice;
use crate::venue::BRoof;
use glam::{Mat4, Vec3};
use std::sync::Arc;

/// The façade strips' band, as shares of the shell's height.
const BAND: (f32, f32) = (0.55, 0.72);
/// A strip's emission at full level.
const FACADE: f32 = 6.0;
/// Watts of lamp a square metre of field gets under a roof.
const LUX: f32 = 40.0;

/// How wide one ceiling lamp is, in metres: a cluster of fixtures, not a
/// share of the floor. Eight lamps this size still light the court evenly
/// and leave each player a shadow under her.
const FIXTURE: f32 = 2.0;

pub fn lights(choice: &BChoice, bowl: &BBowl, out: &mut BVenueBuild) {
    let sport = choice.sport.spec();
    let venue = choice.venue.spec();
    let theme = choice.theme.spec();
    let h = height(choice, bowl);
    // Lamps: a 2 × 4 grid of area lights under any roof; the sun outdoors.
    let [hx, hy] = sport.field.half;
    if venue.roof != BRoof::Open && sport.indoor {
        let (cols, rows) = (4, 2);
        let power = LUX * 4.0 * hx * hy / (cols * rows) as f32;
        for c in 0..cols {
            for r in 0..rows {
                let x = -hx + (2.0 * c as f32 + 1.0) * hx / cols as f32;
                let y = -hy + (2.0 * r as f32 + 1.0) * hy / rows as f32;
                let at = Mat4::from_translation(Vec3::new(x, y, h - 2.0));
                // A fixture's size, not a share of the floor: sized by the
                // court, each of the eight lamps came out 4 m across and
                // lit the players from so many directions at once that
                // nothing cast a shadow and everyone floated (2026-10-28).
                // The power is unchanged — only how gathered the light is.
                out.lamps.push(BLamp::Area { at, size: [FIXTURE, FIXTURE], colour: theme.key, power });
            }
        }
    } else {
        // Late afternoon from the south-west, 50° up.
        let at = Mat4::from_rotation_z(-0.7) * Mat4::from_rotation_x(40f32.to_radians());
        out.lamps.push(BLamp::Sun { at, colour: theme.key, strength: 3.5 });
    }
    // The façade's groups.
    let ring = base(choice, bowl);
    let count = usize::from(venue.panels);
    for k in 0..count {
        let (s0, s1) = (k as f32 / count as f32, (k + 1) as f32 / count as f32);
        let mut m = BMesh::default();
        let at = |s: f32, t: f32| {
            let (p, n) = ring.at(s);
            let (off, z) = profile(venue.shell, &ring, h, s, t, (0, 0));
            let q = p + n * (off + 0.15);
            Vec3::new(q.x, q.y, z)
        };
        let gap = (s1 - s0) * 0.08;
        let steps = 4;
        for i in 0..steps {
            let (a, b) = (s0 + gap + (s1 - s0 - 2.0 * gap) * i as f32 / steps as f32, s0 + gap + (s1 - s0 - 2.0 * gap) * (i + 1) as f32 / steps as f32);
            m.quad(at(a, BAND.0), at(b, BAND.0), at(b, BAND.1), at(a, BAND.1));
        }
        let name = format!("Facade {k:02}");
        out.parts.push(BPart::new(name.clone(), m, BMat::glowing(name.clone(), theme.glow, FACADE), Mat4::IDENTITY));
        out.groups.push(BLightGroup { material: name, s: (s0 + s1) / 2.0, t: (BAND.0 + BAND.1) / 2.0, strength: FACADE });
    }
    // LED boards round an indoor court.
    if sport.indoor {
        let image = Arc::new(boards(theme));
        let mat = BMat::glowing("LED Boards", theme.primary, 1.5).with_image(image);
        let [fx, fy] = sport.field.floor_half();
        let mut m = BMesh::default();
        for (centre, size) in [
            (Vec3::new(0.0, fy - 0.4, 0.45), Vec3::new(2.0 * hx + 2.0, 0.12, 0.9)),
            (Vec3::new(0.0, -(fy - 0.4), 0.45), Vec3::new(2.0 * hx + 2.0, 0.12, 0.9)),
            (Vec3::new(fx - 0.2, 0.0, 0.45), Vec3::new(0.12, 2.0 * hy + 1.0, 0.9)),
            (Vec3::new(-(fx - 0.2), 0.0, 0.45), Vec3::new(0.12, 2.0 * hy + 1.0, 0.9)),
        ] {
            m.merge(&BMesh::cuboid(size), Mat4::from_translation(centre));
        }
        out.parts.push(BPart::new("LED Boards", m, mat, Mat4::IDENTITY));
        out.groups.push(BLightGroup { material: "LED Boards".into(), s: 0.0, t: 0.0, strength: 1.5 });
    }
}

/// A strip of blooms: concentric softly squarish rings on the theme's
/// primary, in its other colours.
fn boards(theme: &crate::theme::BThemeSpec) -> BImage {
    let cols = [theme.secondary, theme.accent, theme.glow, theme.line];
    BImage::painted(1024, 128, |u, v| {
        let (x, y) = (u * 1024.0, v * 128.0);
        let k = (x / 128.0).floor();
        let (cx, cy) = (k * 128.0 + 64.0, if k as i32 % 2 == 0 { 128.0 } else { 64.0 });
        let r = squircle(x - cx, y - cy, 3.2, 1.0);
        for j in 0..4 {
            if (r - (16.0 + j as f32 * 18.0)).abs() < 6.0 {
                return cols[(k as usize + j) % cols.len()];
            }
        }
        theme.primary.mix(crate::colour::INK, 0.6)
    })
}
