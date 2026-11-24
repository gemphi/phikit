//! Letters and drawings raised into solids, as Blender raises a text
//! object: the outlines filled front and back, swept to a depth, their
//! edges rounded (`blend::text`'s display list, the same path a text in
//! a `.blend` takes). Each is then fitted to the part's size: its height
//! the size's, its width the size's or, where that is 0, as wide as the
//! letter's own shape makes it, its depth the size's; its middle at the
//! origin.

use std::path::Path;
use std::sync::Arc;

use glam::Vec3;

use super::svg;
use crate::blend::text::font::BFont;
use crate::blend::text::{BText, CU_BACK, CU_FRONT, TCharInfo, TMesh, TNurb, TSpline, bevlist, displist, mesh, text_mesh};
use crate::geom::shape::part::{ShapeMesh, unit};

/// A font's letter as a solid of `size`, its edges rounded by `round` of
/// its height; `step` is the steps along a curve and round an edge.
pub fn letter(font: &Path, letter: char, size: [f32; 3], round: f32, step: [usize; 2]) -> Option<ShapeMesh> {
    let face = Arc::new(BFont::from_bytes(std::fs::read(font).ok()?)?);
    let make = |extrude: f32, bevel: f32| {
        let text = BText {
            flag: CU_FRONT | CU_BACK,
            fsize: 1.0,
            spacing: 1.0,
            linedist: 1.0,
            wordspace: 1.0,
            smallcaps_scale: 0.75,
            resolu: step[0].max(1),
            extrude,
            bevel_radius: bevel,
            offset: 1.0,
            bevresol: step[1],
            text: vec![letter],
            info: vec![TCharInfo::default()],
            fonts: [Some(face.clone()), None, None, None],
            ..BText::default()
        };
        text_mesh(&text)
    };
    fitted(make, size, round, false)
}

/// A drawing's outlines (the paths of an svg file) as a solid, as a
/// letter is.
pub fn drawing(file: &Path, size: [f32; 3], round: f32, step: [usize; 2]) -> Option<ShapeMesh> {
    raised(svg::rings(&std::fs::read_to_string(file).ok()?, step[0].max(1)), size, round, step[1], false)
}

/// Closed rings of points (a hole inside a ring) as a solid, as a
/// letter is; `bevel` the steps round its rounded edge. Where `own` is
/// true it keeps the rings' own middle (a bloom of odd lobes is not
/// even in its box) and its size is twice its reach from there.
pub fn raised(ring: Vec<Vec<[f32; 2]>>, size: [f32; 3], round: f32, bevel: usize, own: bool) -> Option<ShapeMesh> {
    let nurb: Vec<TNurb> = ring
        .into_iter()
        .map(|r| TNurb { spline: TSpline::Poly(r.into_iter().map(|p| Vec3::new(p[0], p[1], 0.0)).collect()), resolu: 1, charidx: 0, mat_nr: 0 })
        .collect();
    let make = |extrude: f32, bevel_radius: f32| {
        let shape = BText { flag: CU_FRONT | CU_BACK, extrude, bevel_radius, offset: 1.0, bevresol: bevel, ..BText::default() };
        let bev = bevlist::bevel_lists(&shape, &nurb);
        mesh::from_displist(shape.flag, &displist::evaluate(&shape, &nurb, &bev))
    };
    fitted(make, size, round, own)
}

/// Made once flat, to learn how high its shape is; then again, swept to
/// the depth the size asks of that height and rounded; then fitted.
fn fitted(make: impl Fn(f32, f32) -> TMesh, size: [f32; 3], round: f32, own: bool) -> Option<ShapeMesh> {
    let [lo, hi] = extent(&make(0.0, 0.0).vertices)?;
    let high = (hi[1] - lo[1]).max(f32::EPSILON);
    // A letter's edge is rounded by a share of its height; a bloom's,
    // flat and wide, by a share of how thick it is.
    let bevel = round * high * match own {
        true => size[2] / size[1].max(f32::EPSILON),
        false => 1.0,
    };
    let extrude = (size[2] / size[1].max(f32::EPSILON) * high / 2.0 - bevel).max(high * 0.001);
    let solid = make(extrude, bevel);
    let [lo, hi] = extent(&solid.vertices)?;
    let middle = [0, 1, 2].map(|i| match own && i < 2 {
        true => 0.0,
        false => (lo[i] + hi[i]) / 2.0,
    });
    // Its own middle kept, it reaches as far one way as any other: the
    // farthest point from that middle, in its plane.
    let reach = solid.vertices.iter().map(|v| (v[0] * v[0] + v[1] * v[1]).sqrt()).fold(0.0f32, f32::max);
    let across = [0, 1, 2].map(|i| match own && i < 2 {
        true => 2.0 * reach,
        false => hi[i] - lo[i],
    }.max(f32::EPSILON));
    let tall = size[1] / across[1];
    let scale = [match size[0] > 0.0 {
        true => size[0] / across[0],
        false => tall,
    }, tall, size[2] / across[2]];
    let place = |v: [f32; 3]| [0, 1, 2].map(|i| (v[i] - middle[i]) * scale[i]);
    let mut out = ShapeMesh::default();
    for face in &solid.faces {
        let corner: Vec<[f32; 3]> = face.iter().filter_map(|&i| solid.vertices.get(i as usize)).map(|&v| place(v)).collect();
        if corner.len() < 3 {
            continue;
        }
        let normal = newell(&corner);
        if normal == [0.0; 3] {
            continue;
        }
        let with: Vec<([f32; 3], [f32; 3])> = corner.into_iter().map(|p| (p, normal)).collect();
        out.face(&with);
    }
    Some(out)
}

/// The smallest and largest corner of the points' box; none for none.
fn extent(point: &[[f32; 3]]) -> Option<[[f32; 3]; 2]> {
    let first = *point.first()?;
    Some(point.iter().fold([first, first], |[lo, hi], p| [[0, 1, 2].map(|i| lo[i].min(p[i])), [0, 1, 2].map(|i| hi[i].max(p[i]))]))
}

/// A polygon's normal, its corners counter-clockwise seen from where it
/// points (Newell's way, which a bent polygon also has).
pub(super) fn newell(corner: &[[f32; 3]]) -> [f32; 3] {
    let mut n = [0.0f32; 3];
    for (k, a) in corner.iter().enumerate() {
        let b = corner[(k + 1) % corner.len()];
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    unit(n)
}
