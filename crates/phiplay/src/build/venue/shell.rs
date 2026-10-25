//! The shell round the bowl and the roof over it — each of the eight
//! after one of the 2022 stadiums, built from one wall walked round a ring
//! and pushed out and up by the shell's own profile (`profile`), and
//! painted with its own pattern (`paint`). The Containers stack boxes
//! instead of a wall; the Arches add two arches over a plain bowl.

use super::super::image::BImage;
use super::super::mesh::BMesh;
use super::super::part::{BMat, BPart};
use super::super::ring::BRing;
use super::BVenueBuild;
use super::bowl::BBowl;
use crate::choice::BChoice;
use crate::theme::BThemeSpec;
use crate::venue::{BRoof, BShell};
use glam::{Mat4, Quat, Vec2, Vec3};
use std::f32::consts::{PI, TAU};
use std::sync::Arc;

/// Columns round a wall (per metre of ring) and rows up it.
const COLUMNS_PER_M: f32 = 0.5;
const ROWS: usize = 16;

/// The ring a venue's shell stands on.
pub fn base(choice: &BChoice, bowl: &BBowl) -> BRing {
    match choice.venue.spec().shell {
        BShell::Cap => BRing::circle(bowl.half[0].hypot(bowl.half[1]) + bowl.depth + 1.0, 192),
        _ => BRing::rounded(bowl.half, bowl.depth + 0.4, 12),
    }
}

/// The shell's height.
pub fn height(choice: &BChoice, bowl: &BBowl) -> f32 {
    bowl.top + choice.venue.spec().rise
}

/// How far out (along the ring's normal) and how high the shell stands
/// at `s` of the way round and `t` of the way up; `cell` is the grid cell's
/// (column, row) for the faceted shells.
pub fn profile(shell: BShell, ring: &BRing, h: f32, s: f32, t: f32, cell: (usize, usize)) -> (f32, f32) {
    let (_, n) = ring.at(s);
    let corner = (n.x.abs() * n.y.abs() * 2.0).clamp(0.0, 1.0);
    let sails = (ring.length() / 14.0).round().max(4.0);
    match shell {
        BShell::Tent => (2.0 * t + 1.5 * corner * t, t * h * (1.0 + 0.3 * corner)),
        BShell::Lantern => (3.0 * (PI * t).sin() - 0.5 * t, t * h),
        BShell::Sails => {
            let f = (s * sails).fract();
            (0.4 + 2.2 * f * (0.4 + t), t * h * (0.82 + 0.3 * f))
        }
        BShell::Diamond => (if (cell.0 + cell.1) % 2 == 0 { 1.2 } else { 0.0 } * (0.3 + 0.7 * t), t * h),
        BShell::Cap => (if t > 0.8 { -(t - 0.8) / 0.2 * 4.0 } else { 0.0 }, t * h),
        BShell::Dunes => (0.8 * (TAU * (3.0 * s + 0.5 * t)).sin(), t * h * (0.78 + 0.22 * (TAU * 3.0 * s).sin())),
        BShell::Arches | BShell::Containers => (0.0, t * h * 0.85),
    }
}

/// Lays the shell and the roof into `out`.
pub fn shell(choice: &BChoice, bowl: &BBowl, out: &mut BVenueBuild) {
    let spec = choice.venue.spec();
    let theme = choice.theme.spec();
    let ring = base(choice, bowl);
    let h = height(choice, bowl);
    match spec.shell {
        BShell::Containers => containers(&ring, h, theme, out),
        kind => {
            let flat = kind == BShell::Diamond;
            let mesh = wall(&ring, h, flat, |s, t, cell| profile(kind, &ring, h, s, t, cell));
            let image = Arc::new(paint(kind, &ring, theme));
            out.parts.push(BPart::new("Shell", mesh, BMat::plain("Shell", theme.secondary, 0.5).with_image(image), Mat4::IDENTITY));
            if kind == BShell::Arches {
                arches(bowl, &ring, h + spec.rise, theme, out);
            }
        }
    }
    roof(spec.roof, spec.shell, &ring, bowl, h, theme, out);
}

/// A wall walked round `ring`: columns along it, `ROWS` up it, each vertex
/// placed by `shape(s, t, cell)` → (out, z). Faceted when `flat`.
pub fn wall(ring: &BRing, _h: f32, flat: bool, shape: impl Fn(f32, f32, (usize, usize)) -> (f32, f32)) -> BMesh {
    let cols = ((ring.length() * COLUMNS_PER_M) as usize).max(32);
    let at = |c: usize, r: usize| {
        let (s, t) = (c as f32 / cols as f32, r as f32 / ROWS as f32);
        let (p, n) = ring.at(s);
        let (off, z) = shape(s, t, (c, r));
        let q = p + n * off;
        (Vec3::new(q.x, q.y, z), Vec2::new(s, t))
    };
    let mut m = BMesh::default();
    if flat {
        for c in 0..cols {
            for r in 0..ROWS {
                let [a, b, d, e] = [at(c, r), at(c + 1, r), at(c + 1, r + 1), at(c, r + 1)];
                for tri in [[a, b, d], [a, d, e]] {
                    let n = (tri[1].0 - tri[0].0).cross(tri[2].0 - tri[0].0);
                    let i = m.vertex(tri[0].0, n, tri[0].1);
                    m.vertex(tri[1].0, n, tri[1].1);
                    m.vertex(tri[2].0, n, tri[2].1);
                    m.triangle(i, i + 1, i + 2);
                }
            }
        }
        return m;
    }
    // Smooth: shared vertices, normals from the grid's neighbours.
    let grid: Vec<Vec<(Vec3, Vec2)>> = (0..=cols).map(|c| (0..=ROWS).map(|r| at(c, r)).collect()).collect();
    for c in 0..=cols {
        for r in 0..=ROWS {
            let along = grid[(c + 1).min(cols)][r].0 - grid[c.saturating_sub(1)][r].0;
            let up = grid[c][(r + 1).min(ROWS)].0 - grid[c][r.saturating_sub(1)].0;
            m.vertex(grid[c][r].0, along.cross(up), grid[c][r].1);
        }
    }
    let row = (ROWS + 1) as u32;
    for c in 0..cols as u32 {
        for r in 0..ROWS as u32 {
            let (a, b) = (c * row + r, (c + 1) * row + r);
            m.triangle(a, b, b + 1);
            m.triangle(a, b + 1, a + 1);
        }
    }
    m
}

/// The shell's pattern, in (s round, t up), in the theme's colours.
fn paint(kind: BShell, ring: &BRing, theme: &BThemeSpec) -> BImage {
    let round = |metres: f32| (ring.length() / metres).round().max(1.0);
    let line = |x: f32, width: f32| {
        let f = x.fract();
        f < width || f > 1.0 - width
    };
    let (light, dark) = (theme.line, theme.primary);
    BImage::painted(2048, 256, |s, v| {
        let t = 1.0 - v;
        match kind {
            BShell::Tent => {
                if t < 0.14 {
                    // The sadu band: a zigzag of the line colour on the primary.
                    let zig = ((s * round(0.8)).fract() - 0.5).abs() * 2.0;
                    if (zig - t / 0.14).abs() < 0.12 { light } else { dark }
                } else if ((s * round(1.6)) as u32) % 2 == 0 {
                    crate::colour::INK
                } else {
                    light
                }
            }
            BShell::Lantern => {
                let (a, b) = (s * round(3.0) + t * 6.0, s * round(3.0) - t * 6.0);
                if line(a, 0.06) || line(b, 0.06) || line(t * 6.0, 0.04) { theme.accent } else { theme.apron }
            }
            BShell::Sails => {
                if line(s * round(14.0), 0.004) { theme.apron } else { light.mix(theme.secondary, 0.15 * (t * 12.0).sin().abs()) }
            }
            BShell::Diamond => {
                let tri = ((s * round(2.0)).floor() + (t * 16.0).floor()) as u32 % 2;
                if tri == 0 { theme.secondary } else { theme.secondary.mix(theme.primary, 0.35) }
            }
            BShell::Cap => {
                let (a, b) = ((s * round(1.2) + t * 8.0).fract(), (s * round(1.2) - t * 8.0).fract());
                let over = (s * round(1.2) * 2.0 + t * 16.0) as u32 % 2 == 0;
                let band = |x: f32| (x - 0.5).abs() < 0.3;
                match (band(a), band(b)) {
                    (true, true) => if over { light } else { light.mix(dark, 0.25) },
                    (true, false) | (false, true) => light,
                    _ => dark,
                }
            }
            BShell::Dunes => {
                let ribbon = ((t + 0.08 * (TAU * 3.0 * s).sin()) * 7.0) as u32 % 2;
                if ribbon == 0 { theme.accent.mix(light, 0.4) } else { theme.accent.mix(theme.primary, 0.3) }
            }
            BShell::Arches | BShell::Containers => {
                if line(t * 5.0, 0.03) { theme.primary } else { theme.secondary }
            }
        }
    })
}

/// Stadium 974's boxes: twenty-foot containers round the ring in staggered
/// stacks, one mesh per colour.
fn containers(ring: &BRing, h: f32, theme: &BThemeSpec, out: &mut BVenueBuild) {
    let colours = [theme.primary, theme.secondary, theme.accent, theme.apron];
    let mut meshes: Vec<BMesh> = vec![BMesh::default(); colours.len()];
    let box_mesh = BMesh::cuboid(Vec3::new(6.06, 2.44, 2.59));
    let levels = (h / 2.59).ceil() as usize;
    let count = (ring.length() / 6.3) as usize;
    for level in 0..levels {
        for k in 0..count {
            let s = (k as f32 + 0.5 * (level % 2) as f32 + 0.5) / count as f32;
            let (p, n) = ring.at(s);
            let turn = Quat::from_rotation_z(n.y.atan2(n.x) + PI / 2.0);
            let at = Mat4::from_rotation_translation(turn, Vec3::new(p.x + n.x * 1.3, p.y + n.y * 1.3, 1.3 + level as f32 * 2.59));
            meshes[(k * 7 + level * 3) % colours.len()].merge(&box_mesh, at);
        }
    }
    for (i, (mesh, colour)) in meshes.into_iter().zip(colours).enumerate() {
        out.parts.push(BPart::new(format!("Containers {i}"), mesh, BMat::metal(format!("Container {i}"), colour, 0.55), Mat4::IDENTITY));
    }
}

/// Khalifa's twin arches: parabolas along the length, over the bowl.
fn arches(bowl: &BBowl, ring: &BRing, peak: f32, theme: &BThemeSpec, out: &mut BVenueBuild) {
    let reach = ring.points().iter().map(|p| p.x.abs()).fold(0.0, f32::max) + 2.0;
    let mut m = BMesh::default();
    for side in [-1.0f32, 1.0] {
        let y = side * bowl.half[1] * 0.6;
        let steps = 48;
        for k in 0..steps {
            let x = |k: usize| -reach + 2.0 * reach * k as f32 / steps as f32;
            let z = |x: f32| peak * (1.0 - (x / reach).powi(2));
            let (a, b) = (Vec3::new(x(k), y, z(x(k))), Vec3::new(x(k + 1), y, z(x(k + 1))));
            m.merge(&BMesh::rod(a, b, 0.8), Mat4::IDENTITY);
        }
    }
    out.parts.push(BPart::new("Arches", m, BMat::metal("Arches", theme.line, 0.3), Mat4::IDENTITY));
}

/// The roof: a ring over the stands (open over the field), or closed over
/// everything with a low dome.
fn roof(kind: BRoof, shell: BShell, ring: &BRing, bowl: &BBowl, h: f32, theme: &BThemeSpec, out: &mut BVenueBuild) {
    if kind == BRoof::Open {
        return;
    }
    let inner = BRing::rounded(bowl.half, bowl.depth * 0.35, 12);
    let cols = 128;
    let mut m = BMesh::default();
    let edge = |s: f32| {
        let (p, n) = ring.at(s);
        let (off, _) = profile(shell, ring, h, s, 1.0, (0, 0));
        Vec3::new(p.x + n.x * off, p.y + n.y * off, h)
    };
    for c in 0..cols {
        let (s0, s1) = (c as f32 / cols as f32, (c + 1) as f32 / cols as f32);
        let (i0, i1) = (inner.at(s0).0, inner.at(s1).0);
        let lip = h - 1.5;
        m.quad(edge(s0), edge(s1), Vec3::new(i1.x, i1.y, lip), Vec3::new(i0.x, i0.y, lip));
        if kind == BRoof::Closed {
            // The dome: from the ring's inner edge up to a crown over the
            // centre, 4 m above it.
            let crown = Vec3::new(0.0, 0.0, lip + 4.0);
            m.quad(Vec3::new(i0.x, i0.y, lip), Vec3::new(i1.x, i1.y, lip), crown, crown);
        }
    }
    out.parts.push(BPart::new("Roof", m, BMat::plain("Roof", theme.line.mix(theme.apron, 0.3), 0.6), Mat4::IDENTITY));
}
