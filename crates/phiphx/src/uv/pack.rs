//! The islands laid into the 0..1 square, as Blender's Unwrap does after
//! it solves (`uv.pack_islands`, in its simplest form).
//!
//! A shelf packer: the islands in order of height, laid left to right in
//! rows, each scaled by the one factor that makes the whole set fit. One
//! scale for every island rather than one each, because scaling islands
//! independently changes their relative texel density — which is the thing
//! a UV layout is *for*.

use super::island::BUvIsland;
use super::{BUvMesh, BUvPin};

/// The box an island's UVs span.
fn box_of(mesh: BUvMesh<'_>, part: &BUvIsland, uv: &[[f32; 2]]) -> ([f32; 2], [f32; 2]) {
    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for &f in &part.face {
        for c in mesh.starts[f] as usize..mesh.starts[f + 1] as usize {
            for k in 0..2 {
                lo[k] = lo[k].min(uv[c][k]);
                hi[k] = hi[k].max(uv[c][k]);
            }
        }
    }
    (lo, hi)
}

/// Lay `islands` into 0..1. An island holding a **pinned** corner is left
/// exactly where it is: a pin that the packer then moves is not a pin.
pub fn into_square(mesh: BUvMesh<'_>, islands: &[BUvIsland], uv: &mut [[f32; 2]], pin: &[BUvPin]) {
    let pinned: Vec<bool> = islands
        .iter()
        .map(|part| {
            pin.iter().any(|p| {
                part.face.iter().any(|&f| (mesh.starts[f] as usize..mesh.starts[f + 1] as usize).contains(&p.corner))
            })
        })
        .collect();
    let movable: Vec<usize> = (0..islands.len()).filter(|k| !pinned[*k]).collect();
    if movable.is_empty() {
        return;
    }
    let mut boxes: Vec<(usize, [f32; 2], [f32; 2])> = movable.iter().map(|&k| {
        let (lo, hi) = box_of(mesh, &islands[k], uv);
        (k, lo, [hi[0] - lo[0], hi[1] - lo[1]])
    }).collect();
    // Tallest first, which is what makes a shelf packer waste little.
    boxes.sort_by(|a, b| b.2[1].total_cmp(&a.2[1]));
    let margin = 0.01f32;
    // One scale for the set: the square's area over the islands' own,
    // tempered so a single island does not blow up to fill the square.
    let total: f32 = boxes.iter().map(|b| (b.2[0] + margin) * (b.2[1] + margin)).sum();
    let widest = boxes.iter().map(|b| b.2[0].max(b.2[1])).fold(1e-6, f32::max);
    let mut scale = (0.98 / total.max(1e-12)).sqrt().min((1.0 - 2.0 * margin) / widest);
    for _ in 0..24 {
        if shelf(&boxes, scale, margin).is_some() {
            break;
        }
        scale *= 0.85;
    }
    let Some(place) = shelf(&boxes, scale, margin) else { return };
    for (n, (k, lo, _)) in boxes.iter().enumerate() {
        let at = place[n];
        for &f in &islands[*k].face {
            for c in mesh.starts[f] as usize..mesh.starts[f + 1] as usize {
                uv[c] = [(uv[c][0] - lo[0]) * scale + at[0], (uv[c][1] - lo[1]) * scale + at[1]];
            }
        }
    }
}

/// Where each box lands on the shelves at `scale`, or `None` when they do
/// not fit in the square.
fn shelf(boxes: &[(usize, [f32; 2], [f32; 2])], scale: f32, margin: f32) -> Option<Vec<[f32; 2]>> {
    let (mut x, mut y, mut row) = (margin, margin, 0.0f32);
    let mut out = Vec::with_capacity(boxes.len());
    for (_, _, size) in boxes {
        let (w, h) = (size[0] * scale, size[1] * scale);
        // An island wider than the square never fits, however the shelves
        // are arranged — say so rather than laying it past the edge, which
        // is what put a whole unwrapped cylinder outside 0..1.
        if w + 2.0 * margin > 1.0 || h + 2.0 * margin > 1.0 {
            return None;
        }
        if x + w + margin > 1.0 {
            x = margin;
            y += row + margin;
            row = 0.0;
        }
        if y + h + margin > 1.0 {
            return None;
        }
        out.push([x, y]);
        x += w + margin;
        row = row.max(h);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::super::tests::grid;
    use super::super::{BUnwrap, BUvPin, unwrap};
    use super::*;

    /// Two islands come back inside the square and not on top of each
    /// other.
    #[test]
    fn the_islands_land_in_the_square_apart() {
        let (pos, corner, starts) = grid(2);
        let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
        let uv = unwrap(mesh, BUnwrap::Conformal, &[(1, 4), (4, 7)], &[]);
        assert!(uv.iter().all(|c| (0.0..=1.0).contains(&c[0]) && (0.0..=1.0).contains(&c[1])), "inside the square: {uv:?}");
        let islands = super::super::island::split(mesh, &[(1, 4), (4, 7)]);
        assert_eq!(islands.len(), 2);
        let a = box_of(mesh, &islands[0], &uv);
        let b = box_of(mesh, &islands[1], &uv);
        let apart = a.1[0] <= b.0[0] + 1e-4 || b.1[0] <= a.0[0] + 1e-4 || a.1[1] <= b.0[1] + 1e-4 || b.1[1] <= a.0[1] + 1e-4;
        assert!(apart, "the two islands do not overlap: {a:?} {b:?}");
    }

    /// An island holding a pin is left exactly where the solver put it —
    /// the packer moving it would undo the pin.
    #[test]
    fn a_pinned_island_is_not_moved_by_the_packer() {
        let (pos, corner, starts) = grid(2);
        let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
        let seam = [(1u32, 4u32), (4, 7)];
        let pins = [BUvPin { corner: 0, uv: [0.3, 0.4] }, BUvPin { corner: 1, uv: [0.7, 0.4] }];
        let uv = unwrap(mesh, BUnwrap::Conformal, &seam, &pins);
        for p in pins {
            assert!((uv[p.corner][0] - p.uv[0]).abs() < 1e-4 && (uv[p.corner][1] - p.uv[1]).abs() < 1e-4, "pin {p:?} held: {:?}", uv[p.corner]);
        }
    }
}
