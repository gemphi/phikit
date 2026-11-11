//! Blender `subdiv_displacement_multires.cc`: the displacement a multires
//! grid adds at a PTEX point — read from the corner's `MDisps` grid in
//! tangent space, turned into object space by
//! `BKE_multires_construct_tangent_matrix`, and on a grid's seams averaged
//! with the neighbouring grids (`average_displacement`).

use super::grid::{ptex_to_grid, rotate_grid_to_quad, rotate_quad_to_corner, BGrids};
use super::limit::BLimit;
use super::read::BMDisps;

type F3 = [f32; 3];

/// `AverageWith`: which grids a grid-boundary point is averaged with.
#[derive(Clone, Copy, PartialEq)]
enum BAverageWith {
    None,
    All,
    Prev,
    Next,
}

/// `MultiresDisplacementData`.
pub(super) struct BDisplacement<'a> {
    pub grids: &'a BGrids,
    pub limit: &'a BLimit<'a>,
    pub mdisps: &'a BMDisps,
    /// `grid_size_from_level(totlvl)`: the saved grids are always at the top
    /// level, so a lower viewport level reads them at a stride.
    pub grid_size: usize,
}

/// `math::normalize`: zero below Blender's threshold.
fn normalize(a: F3) -> F3 {
    let l2 = a[0] * a[0] + a[1] * a[1] + a[2] * a[2];
    match l2 > 1.0e-35 {
        true => {
            let l = l2.sqrt();
            [a[0] / l, a[1] / l, a[2] / l]
        }
        false => [0.0; 3],
    }
}

fn cross(a: F3, b: F3) -> F3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// `BKE_multires_construct_tangent_matrix` (`multires_inline.hh`): the
/// matrix's x, y, z axes.
fn tangent_matrix(dpdu: F3, dpdv: F3, corner: usize) -> [F3; 3] {
    let n = |a: F3| [-a[0], -a[1], -a[2]];
    let (x, y) = match corner {
        0 => (n(dpdv), n(dpdu)),
        1 => (dpdu, n(dpdv)),
        2 => (dpdv, dpdu),
        _ => (n(dpdu), dpdv),
    };
    [normalize(x), normalize(y), normalize(cross(dpdu, dpdv))]
}

/// `math::transform_direction`.
fn transform(m: &[F3; 3], d: F3) -> F3 {
    std::array::from_fn(|i| m[0][i] * d[0] + m[1][i] * d[1] + m[2][i] * d[2])
}

impl BDisplacement<'_> {
    /// `read_displacement_grid`: the nearest saved value, and whether the
    /// point is on the grid's centre or one of its two inner edges.
    fn read_grid(&self, grid: Option<&Vec<F3>>, gu: f32, gv: f32) -> (F3, BAverageWith) {
        let Some(disps) = grid else { return ([0.0; 3], BAverageWith::None) };
        let gs = self.grid_size;
        let x = (gu * (gs - 1) as f32).round() as usize;
        let y = (gv * (gs - 1) as f32).round() as usize;
        let d = disps.get(y * gs + x).copied().unwrap_or([0.0; 3]);
        let with = match (x == 0, y == 0) {
            (true, true) => BAverageWith::All,
            (true, false) => BAverageWith::Prev,
            (false, true) => BAverageWith::Next,
            (false, false) => BAverageWith::None,
        };
        (d, with)
    }

    fn grid(&self, loop_index: usize) -> Option<&Vec<F3>> {
        self.mdisps.get(loop_index).and_then(|g| g.as_ref())
    }

    /// `displacement_get_grid_and_coord`: the grid, the grid coordinates,
    /// and the quad corner (0 off a quad).
    fn grid_and_coord(&self, ptex: usize, u: f32, v: f32) -> (usize, f32, f32, usize) {
        let (f, pc) = self.grids.ptex_corner[ptex];
        let start = self.grids.face_start[f] + pc;
        match self.grids.face_size[f] {
            4 => {
                let (corner, cu, cv) = rotate_quad_to_corner(u, v);
                let (gu, gv) = ptex_to_grid(cu, cv);
                (start + corner, gu, gv, corner)
            }
            _ => {
                let (gu, gv) = ptex_to_grid(u, v);
                (start, gu, gv, 0)
            }
        }
    }

    /// `average_with_other`: the other grid's value at the same point, in
    /// object space by the derivatives of ITS PTEX face, averaged in.
    fn average_with_other(&self, ptex: usize, corner: usize, gu: f32, gv: f32, delta: isize, d: &mut F3) {
        let (f, pc) = self.grids.ptex_corner[ptex];
        let n = self.grids.face_size[f] as isize;
        let quad = n == 4;
        let effective = match quad {
            true => corner,
            false => pc,
        } as isize;
        let other_grid = self.grid(self.grids.face_start[f] + ((effective + delta + n) % n) as usize);
        let other_corner = ((corner as isize + delta + n) % n) as usize;
        let other_ptex = self.grids.ptex_offset[f] + if quad { 0 } else { other_corner };
        let (u, v) = match quad {
            true => rotate_grid_to_quad(other_corner, gu, gv),
            false => ptex_to_grid(gu, gv),
        };
        let (dpdu, dpdv) = self.limit.derivatives(other_ptex, u, v);
        let m = tangent_matrix(dpdu, dpdv, if quad { other_corner } else { 0 });
        let other = transform(&m, self.read_grid(other_grid, gu, gv).0);
        *d = std::array::from_fn(|i| (d[i] + other[i]) * 0.5);
    }

    /// `eval_displacement`: the object-space displacement at PTEX
    /// `(ptex, u, v)`, given the limit derivatives there.
    pub(super) fn eval(&self, ptex: usize, u: f32, v: f32, dpdu: F3, dpdv: F3) -> F3 {
        let (l, gu, gv, corner_of_quad) = self.grid_and_coord(ptex, u, v);
        let (tangent_d, with) = self.read_grid(self.grid(l), gu, gv);
        let m = tangent_matrix(dpdu, dpdv, corner_of_quad);
        let mut d = transform(&m, tangent_d);
        // `displacement_get_face_corner`.
        let (f, pc) = self.grids.ptex_corner[ptex];
        let corner = match self.grids.face_size[f] {
            4 => rotate_quad_to_corner(u, v).0,
            _ => pc,
        };
        match with {
            BAverageWith::All => {
                for delta in 1..self.grids.face_size[f] as isize {
                    self.average_with_other(ptex, corner, 0.0, 0.0, delta, &mut d);
                }
            }
            BAverageWith::Prev => self.average_with_other(ptex, corner, gv, 0.0, -1, &mut d),
            BAverageWith::Next => self.average_with_other(ptex, corner, 0.0, gu, 1, &mut d),
            BAverageWith::None => {}
        }
        d
    }

    /// `eval_final_point`'s displacement half: derivatives, then `eval`.
    pub(super) fn at(&self, ptex: usize, u: f32, v: f32) -> F3 {
        let (dpdu, dpdv) = self.limit.derivatives(ptex, u, v);
        self.eval(ptex, u, v, dpdu, dpdv)
    }
}
