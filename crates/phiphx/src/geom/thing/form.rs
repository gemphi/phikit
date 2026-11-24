//! A part's own mesh, made as its form says and as large as its size,
//! its middle at its own origin: a box rounded at its edges, a ball, a
//! pill, a can, a cone, a ring and a dome (each turned about its up
//! axis, then stretched to its size), a letter of a font or an outline
//! of a drawing (raised to its depth, its edges rounded).

use super::outline;
use super::read::{ThingForm, ThingRow, ThingSet};
use crate::geom::shape::lathe::{self, Profile};
use crate::geom::shape::part::{Frame, ShapeMesh, unit};
use crate::geom::shape::solid;
use crate::geom::shape::{ShapeLook, ShapePart};

/// The part's mesh in its own space, under its name and in its look;
/// none where it cannot be made (a letter whose font is missing).
pub fn mesh(set: &ThingSet, row: &ThingRow, look: &ShapeLook) -> Option<ShapePart> {
    let [x, y, z] = row.size;
    let word = form_word(row.form);
    let [segment, ring, bevel] = set.step(word);
    // A turned form scalloped where its row says how: so many lobes
    // round it, each so deep a share of its radius.
    let lobe = lobes(&row.from);
    // Eight steps a lobe at least, so a lobe is round, not a corner.
    let round_steps = match lobe {
        Some((n, _)) => segment.max(n as usize * 8),
        None => segment,
    };
    let turned = |profile: Profile, stretch: [f32; 3]| {
        let mut mesh = ShapeMesh::default();
        lathe::sweep(&mut mesh, &profile, round_steps, &Frame::IDENTITY);
        let part = mesh.part(look);
        let part = match lobe {
            Some((n, deep)) => lobed(part, n, deep),
            None => part,
        };
        Some(stretched(part, stretch))
    };
    let r = x / 2.0;
    match row.form {
        ThingForm::Box => {
            let mut mesh = ShapeMesh::default();
            solid::rounded_block(&mut mesh, row.size, row.round * x.min(y).min(z), ring);
            Some(mesh.part(look))
        }
        ThingForm::Sphere => turned(lathe::sphere(r, ring), [1.0, y / x, z / x]),
        ThingForm::Capsule => turned(lathe::capsule(r, y.max(x), ring), [1.0, 1.0, z / x]),
        ThingForm::Cylinder => turned(lathe::cylinder(r, y), [1.0, 1.0, z / x]),
        ThingForm::Cone => turned(lathe::cone(r, y), [1.0, 1.0, z / x]),
        ThingForm::Torus => turned(lathe::torus(x, y, ring), [1.0, 1.0, z / x]),
        // Its floor at the middle's height less half its own, so its
        // middle is its box's.
        ThingForm::Dome => turned(lathe::dome(r, ring), [1.0, y / r, z / x]).map(|p| moved(p, [0.0, -y / 2.0, 0.0])),
        ThingForm::Letter => {
            let (source, letter) = row.from.split_once(':')?;
            let font = set.source.get(source)?;
            outline::letter(font, letter.chars().next()?, row.size, row.round, [ring, bevel]).map(|m| m.part(look))
        }
        ThingForm::Outline => outline::drawing(set.source.get(&row.from)?, row.size, row.round, [ring, bevel]).map(|m| m.part(look)),
        ThingForm::Bloom => {
            let (n, deep) = lobe.unwrap_or((0.0, 0.0));
            let hole = row.from.split_whitespace().nth(2).and_then(|w| w.parse::<f32>().ok()).unwrap_or(0.0);
            outline::raised(bloom(n, deep, hole, segment.max(12)), row.size, row.round, bevel, true).map(|m| m.part(look))
        }
        ThingForm::Thing => None,
    }
}

/// The word `data/thing.tsv` knows a form by.
fn form_word(form: ThingForm) -> &'static str {
    match form {
        ThingForm::Box => "box",
        ThingForm::Sphere => "sphere",
        ThingForm::Capsule => "capsule",
        ThingForm::Cylinder => "cylinder",
        ThingForm::Cone => "cone",
        ThingForm::Torus => "torus",
        ThingForm::Dome => "dome",
        ThingForm::Letter => "letter",
        ThingForm::Outline => "outline",
        ThingForm::Bloom => "bloom",
        ThingForm::Thing => "thing",
    }
}

/// How many lobes and how deep, from a row's `from` (`8 0.2`).
fn lobes(from: &str) -> Option<(f32, f32)> {
    let mut word = from.split_whitespace().map(|w| w.parse::<f32>().ok());
    match (word.next()??, word.next()??) {
        (n, deep) if n >= 1.0 => Some((n, deep)),
        _ => None,
    }
}

/// A part's points pushed out and drawn in round its up axis, so many
/// lobes round, each `deep` of the radius; each corner then faces as
/// its face does.
fn lobed(mut part: ShapePart, n: f32, deep: f32) -> ShapePart {
    part.point.iter_mut().for_each(|p| {
        let grow = 1.0 + deep * (n * p[2].atan2(p[0])).cos();
        *p = [p[0] * grow, p[1], p[2] * grow];
    });
    let mut at = 0usize;
    for &count in &part.count {
        let corner: Vec<[f32; 3]> = part.index[at..at + count as usize].iter().map(|&i| part.point[i as usize]).collect();
        let normal = outline::newell(&corner);
        part.normal[at..at + count as usize].iter_mut().for_each(|n| *n = normal);
        at += count as usize;
    }
    part
}

/// A bloom's outline, a unit across: a ring of `n` lobes each `deep` of
/// its radius, and where `hole` is more than 0, a hole of the same
/// shape that share as large; `step` points round.
fn bloom(n: f32, deep: f32, hole: f32, step: usize) -> Vec<Vec<[f32; 2]>> {
    let ring = |scale: f32| -> Vec<[f32; 2]> {
        (0..step)
            .map(|k| {
                let a = std::f32::consts::TAU * k as f32 / step as f32;
                let r = scale * 0.5 * (1.0 + deep * (n * a).cos());
                [r * a.cos(), r * a.sin()]
            })
            .collect()
    };
    match hole > 0.0 {
        true => vec![ring(1.0), ring(hole).into_iter().rev().collect()],
        false => vec![ring(1.0)],
    }
}

/// A part stretched along each axis, its normals kept square to it.
fn stretched(mut part: ShapePart, by: [f32; 3]) -> ShapePart {
    let by = by.map(|v| match v.is_finite() && v > 0.0 {
        true => v,
        false => 1.0,
    });
    part.point.iter_mut().for_each(|p| *p = [p[0] * by[0], p[1] * by[1], p[2] * by[2]]);
    part.normal.iter_mut().for_each(|n| *n = unit([n[0] / by[0], n[1] / by[1], n[2] / by[2]]));
    part
}

/// A part moved by `by`.
fn moved(mut part: ShapePart, by: [f32; 3]) -> ShapePart {
    part.point.iter_mut().for_each(|p| *p = [p[0] + by[0], p[1] + by[1], p[2] + by[2]]);
    part
}
