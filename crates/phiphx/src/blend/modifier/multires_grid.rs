//! The refined cage seen through Blender's two parametrisations of a face:
//! PTEX faces (`subdiv_foreach.cc`: one per quad, one per corner of any
//! other face) and the per-corner grids `MDisps` are stored on
//! (`subdiv_displacement_multires.cc`), plus the coordinate conversions of
//! `subdiv_inline.hh` between them.
//!
//! A grid here is a coarse corner's quarter of its face in corner-local
//! coordinates `(s, t)`: `(0, 0)` at the corner's vertex, `s` along the edge
//! to the next corner, `t` along the edge to the previous one, `(1, 1)` at
//! the face centre — Blender's `corner_u, corner_v` after
//! `rotate_quad_to_corner`, and a non-quad's PTEX `(u, v)`.

use crate::blend::BFaces;

/// Blender `rotate_quad_to_corner`: the corner of a quad PTEX face `(u, v)`
/// falls in, and the point in that corner's coordinates.
pub(super) fn rotate_quad_to_corner(u: f32, v: f32) -> (usize, f32, f32) {
    match (u <= 0.5, v <= 0.5) {
        (true, true) => (0, 2.0 * u, 2.0 * v),
        (false, true) => (1, 2.0 * v, 2.0 * (1.0 - u)),
        (false, false) => (2, 2.0 * (1.0 - u), 2.0 * (1.0 - v)),
        (true, false) => (3, 2.0 * (1.0 - v), 2.0 * u),
    }
}

/// Blender `rotate_grid_to_quad`: grid `(u, v)` of `corner` back to the quad.
pub(super) fn rotate_grid_to_quad(corner: usize, gu: f32, gv: f32) -> (f32, f32) {
    match corner {
        0 => (0.5 - gv * 0.5, 0.5 - gu * 0.5),
        1 => (0.5 + gu * 0.5, 0.5 - gv * 0.5),
        2 => (0.5 + gv * 0.5, 0.5 + gu * 0.5),
        _ => (0.5 - gu * 0.5, 0.5 + gv * 0.5),
    }
}

/// Blender `ptex_face_uv_to_grid_uv` (its own inverse,
/// `grid_uv_to_ptex_face_uv`): the grid's origin is the face centre.
pub(super) fn ptex_to_grid(u: f32, v: f32) -> (f32, f32) {
    (1.0 - v, 1.0 - u)
}

/// Every coarse corner's grid of refined-cage vertices, and the PTEX
/// bookkeeping of `displacement_data_init_mapping`.
pub(super) struct BGrids {
    /// Grid side less one, `2^(level - 1)`.
    pub r: usize,
    /// The cage vertex at each grid point, `(r + 1)²` per coarse corner,
    /// row `t`, column `s`, grids in corner (loop) order.
    cells: Vec<u32>,
    /// Each PTEX face's coarse face and corner (`ptex_face_corner`; a quad's
    /// is its first corner).
    pub ptex_corner: Vec<(usize, usize)>,
    /// Each coarse face's first PTEX face (`face_ptex_offset`).
    pub ptex_offset: Vec<usize>,
    /// Each coarse face's first corner and corner count.
    pub face_start: Vec<usize>,
    pub face_size: Vec<usize>,
}

impl BGrids {
    /// The grids of `coarse` refined `level` (≥ 1) times into `cage`, as
    /// `subsurf.rs` refines: the first level turns corner `c` of a face into
    /// the quad `[corner, next edge point, face point, previous edge point]`
    /// at the corner's loop index, every later level splits quad `q` into
    /// `4q + i`, the child at its corner `i` laid out the same way. `None`
    /// when `cage` is not that refinement (a refinement cut short).
    pub(super) fn build(coarse: &BFaces, cage: &BFaces, level: u32) -> Option<Self> {
        if level == 0 {
            return None;
        }
        let r = 1usize << (level - 1);
        let side = r + 1;
        let corners = coarse.corners();
        if cage.len() != corners << (2 * (level - 1)) {
            return None;
        }
        let mut cells = vec![u32::MAX; corners * side * side];
        let ri = r as i32;
        let mut stack: Vec<(usize, u32, [[i32; 2]; 4])> = Vec::new();
        for l in 0..corners {
            stack.push((l, 1, [[0, 0], [ri, 0], [ri, ri], [0, ri]]));
            while let Some((q, k, c)) = stack.pop() {
                if k == level {
                    let face = cage.get(q);
                    if face.len() != 4 {
                        return None;
                    }
                    for j in 0..4 {
                        cells[l * side * side + c[j][1] as usize * side + c[j][0] as usize] = face[j];
                    }
                    continue;
                }
                let mid = |a: [i32; 2], b: [i32; 2]| [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2];
                let centre = mid(c[0], c[2]);
                for i in 0..4 {
                    stack.push((4 * q + i, k + 1, [c[i], mid(c[i], c[(i + 1) % 4]), centre, mid(c[(i + 3) % 4], c[i])]));
                }
            }
        }
        let (mut ptex_corner, mut ptex_offset) = (Vec::new(), Vec::with_capacity(coarse.len()));
        let (mut face_start, mut face_size) = (Vec::with_capacity(coarse.len()), Vec::with_capacity(coarse.len()));
        for (f, face) in coarse.iter().enumerate() {
            ptex_offset.push(ptex_corner.len());
            face_start.push(coarse.start(f));
            face_size.push(face.len());
            match face.len() {
                4 => ptex_corner.push((f, 0)),
                n => ptex_corner.extend((0..n).map(|c| (f, c))),
            }
        }
        Some(Self { r, cells, ptex_corner, ptex_offset, face_start, face_size })
    }

    /// The cage vertex at grid point `(s, t)` of corner (loop) `l`.
    pub(super) fn vertex(&self, l: usize, s: usize, t: usize) -> u32 {
        let side = self.r + 1;
        self.cells[l * side * side + t.min(self.r) * side + s.min(self.r)]
    }

    /// Where PTEX `(ptex, u, v)` lands: the corner (loop) whose grid holds
    /// it, the grid point, and — for a quad — which of its corners
    /// `rotate_quad_to_corner` chose (what the derivatives are turned by).
    pub(super) fn locate(&self, ptex: usize, u: f32, v: f32) -> (usize, usize, usize, Option<usize>) {
        let (f, pc) = self.ptex_corner[ptex];
        let (corner, s, t, quad) = match self.face_size[f] {
            4 => {
                let (c, s, t) = rotate_quad_to_corner(u, v);
                (c, s, t, Some(c))
            }
            _ => (pc, u, v, None),
        };
        let r = self.r as f32;
        (self.face_start[f] + corner, (s * r).round() as usize, (t * r).round() as usize, quad)
    }

    /// The cage vertex PTEX `(ptex, u, v)` lands on.
    pub(super) fn vertex_at(&self, ptex: usize, u: f32, v: f32) -> u32 {
        let (l, s, t, _) = self.locate(ptex, u, v);
        self.vertex(l, s, t)
    }
}
