//! Filling a band of a 3D picture's rows with the triangles over it:
//! each point whose middle is inside a triangle, the nearer over what is
//! there, lit where it is from the normals at the triangle's corners, so
//! a curve looks round and not cut in faces. A shadow fills the same way
//! but only darkens what it falls on, once however many triangles of it
//! fall there.

use super::material::PMaterial;
use super::sun::PSun;

/// A triangle on the picture: its corners (across, down, depth), the
/// way the surface faces at each corner, where each stands in space, its
/// colour, and what it is made of, where that is more than a colour.
pub(super) struct Flat<'a> {
    pub corner: [[f32; 3]; 3],
    pub normal: [[f32; 3]; 3],
    pub world: [[f32; 3]; 3],
    pub colour: [f32; 3],
    pub material: Option<&'a PMaterial>,
}

/// What a fill does: lights each point, or darkens what is there by a
/// share.
#[derive(Clone, Copy)]
pub(super) enum Ink {
    Lit,
    Shadow(f32),
}

/// A band of a picture: its points, their depths, and its first row.
pub(super) struct Band<'a> {
    pub pixel: &'a mut [u8],
    pub depth: &'a mut [f32],
    pub width: usize,
    pub top: usize,
}

/// How near a shadow must be to what is there to fall on it, as a share
/// of the depth.
const ON: f32 = 0.003;

pub(super) fn fill(flat: &[Flat<'_>], band: Band, sun: &PSun, view: [f32; 3], ink: Ink) {
    let Band { pixel, depth, width, top } = band;
    let height = depth.len() / width.max(1);
    let (low, high) = (top as f32, (top + height) as f32);
    let mut dark = vec![false; depth.len()];
    for t in flat {
        let [a, b, c] = t.corner;
        // From the first middle at or past the low edge to the last at or
        // before the high one.
        let (y0, y1) = ((a[1].min(b[1]).min(c[1]) - 0.5).ceil().max(low), (a[1].max(b[1]).max(c[1]) - 0.5).floor().min(high - 1.0));
        let (x0, x1) = ((a[0].min(b[0]).min(c[0]) - 0.5).ceil().max(0.0), (a[0].max(b[0]).max(c[0]) - 0.5).floor().min(width as f32 - 1.0));
        let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if y0 > y1 || x0 > x1 || area.abs() < 1e-9 {
            continue;
        }
        for y in y0 as usize..=y1 as usize {
            let py = y as f32 + 0.5;
            for x in x0 as usize..=x1 as usize {
                let px = x as f32 + 0.5;
                let w0 = ((b[0] - px) * (c[1] - py) - (b[1] - py) * (c[0] - px)) / area;
                let w1 = ((c[0] - px) * (a[1] - py) - (c[1] - py) * (a[0] - px)) / area;
                let w2 = 1.0 - w0 - w1;
                // Depth across the picture is exact as one over it.
                let z = 1.0 / (w0 / a[2] + w1 / b[2] + w2 / c[2]);
                let at = (y - top) * width + x;
                let inside = w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0;
                match (inside, ink) {
                    (true, Ink::Lit) if z < depth[at] => {
                        depth[at] = z;
                        let n = [0, 1, 2].map(|i| w0 * t.normal[0][i] + w1 * t.normal[1][i] + w2 * t.normal[2][i]);
                        let lit = match t.material {
                            // Where it stands in space, true across the
                            // picture as its depth is, for the pattern.
                            Some(material) => {
                                let k = [w0 * z / a[2], w1 * z / b[2], w2 * z / c[2]];
                                let p = [0, 1, 2].map(|i| k[0] * t.world[0][i] + k[1] * t.world[1][i] + k[2] * t.world[2][i]);
                                sun.lit_surface(&material.surface(t.colour, p, n), view)
                            }
                            None => sun.lit(t.colour, n, view),
                        }.map(|v| (v * 255.0) as u8);
                        pixel[at * 4..at * 4 + 4].copy_from_slice(&[lit[2], lit[1], lit[0], 255]);
                    }
                    (true, Ink::Shadow(_)) => dark[at] |= depth[at] < f32::MAX && (z - depth[at]).abs() <= depth[at] * ON,
                    _ => {}
                }
            }
        }
    }
    let Ink::Shadow(share) = ink else { return };
    for (at, _) in dark.iter().enumerate().filter(|(_, d)| **d) {
        pixel[at * 4..at * 4 + 3].iter_mut().for_each(|v| *v = (*v as f32 * (1.0 - share)) as u8);
    }
}
