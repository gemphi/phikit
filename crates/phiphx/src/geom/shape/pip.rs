//! The dice's pips: which face shows which value and where its pips sit,
//! from `data/pip.tsv`; each pip a shallow closed disc on its face.

use super::lathe;
use super::part::{Frame, ShapeMesh};
use crate::table::TsvTable;

/// One face of the dice: the value it shows, the axis it looks along
/// (0 x, 1 y, 2 z) and which way (+1 / -1), and its pips on the face's
/// 3 × 3 grid (u, v each -1, 0 or 1).
#[derive(Clone, Debug, PartialEq)]
pub struct PipFace {
    pub value: u32,
    pub axis: usize,
    pub sign: f32,
    pub cell: Vec<[f32; 2]>,
}

impl PipFace {
    /// Every face, as the table lists them.
    pub fn all() -> Vec<PipFace> {
        let table = TsvTable::parse(include_str!("../../../data/pip.tsv"));
        table.row().filter_map(|row| {
            let (sign, axis) = row.get("face").split_at(1);
            let axis = match axis {
                "x" => 0,
                "y" => 1,
                "z" => 2,
                _ => return None,
            };
            let sign = match sign {
                "-" => -1.0,
                _ => 1.0,
            };
            let cell = row
                .word_list("cell")
                .iter()
                .filter_map(|c| c.split_once(','))
                .map(|(u, v)| [u.parse().unwrap_or(0.0), v.parse().unwrap_or(0.0)])
                .collect();
            Some(PipFace { value: row.count("value") as u32, axis, sign, cell })
        })
        .collect()
    }

    /// The centre of each of its pips on a dice `size` across, `spread` the grid step.
    pub fn centre(&self, size: f32, spread: f32) -> Vec<[f32; 3]> {
        let (u, v) = ((self.axis + 1) % 3, (self.axis + 2) % 3);
        self.cell
            .iter()
            .map(|c| {
                let mut p = [0.0; 3];
                p[self.axis] = self.sign * size / 2.0;
                p[u] = c[0] * spread;
                p[v] = c[1] * spread;
                p
            })
            .collect()
    }

    /// Its pips as discs `radius` wide and `depth` thick, centred on the face.
    pub fn disc(&self, mesh: &mut ShapeMesh, size: f32, spread: f32, radius: f32, depth: f32, segment: usize) {
        let (u, v) = ((self.axis + 1) % 3, (self.axis + 2) % 3);
        let unit = |a: usize, s: f32| {
            let mut e = [0.0; 3];
            e[a] = s;
            e
        };
        // Local y out of the face, x along u, z = x × y: a right-handed turn.
        let profile = lathe::cylinder(radius, depth);
        for origin in self.centre(size, spread) {
            let frame = Frame { origin, x: unit(u, 1.0), y: unit(self.axis, self.sign), z: unit(v, -self.sign) };
            lathe::sweep(mesh, &profile, segment, &frame);
        }
    }
}
