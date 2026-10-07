//! Unwrapping a mesh into UVs — Blender's `U` menu in Edit Mode.
//!
//! * [`lscm`] — **Unwrap ▸ Conformal**: least-squares conformal maps, the
//!   solver Blender's `param_lscm_*` runs (and the second half of its
//!   Angle Based method). Angles are preserved as nearly as the mesh
//!   allows; area is not, which is what "conformal" means.
//! * [`project`] — the projections that are not solvers at all: Cube,
//!   Cylinder, Sphere and Project from View.
//! * [`island`] — the mesh cut into the pieces that unwrap separately,
//!   along the seams.
//! * [`pack`] — the islands laid into the 0..1 square afterwards, as
//!   Blender packs them.
//!
//! **Pins are the point.** A conformal map is only defined up to a
//! similarity — rotate, scale and move an island and it is still
//! conformal — so a solver must be told at least two points. Blender uses
//! the user's pinned vertices when there are any and picks two itself when
//! there are not; the pinned ones are fixed and everything else is solved
//! around them, which is how a pin actually holds a UV in place rather
//! than merely being drawn differently.

pub mod island;
pub mod lscm;
pub mod pack;
pub mod project;

/// A mesh to unwrap: positions, and faces as runs of corner → vertex.
#[derive(Clone, Copy, Debug)]
pub struct BUvMesh<'a> {
    pub pos: &'a [[f32; 3]],
    /// Corner `c`'s vertex.
    pub corner: &'a [u32],
    /// `starts[f] .. starts[f + 1]` is face `f`'s run of corners.
    pub starts: &'a [u32],
}

impl BUvMesh<'_> {
    pub fn faces(&self) -> usize {
        self.starts.len().saturating_sub(1)
    }

    /// Face `f`'s corners.
    pub fn face(&self, f: usize) -> &[u32] {
        let (a, b) = (self.starts[f] as usize, self.starts[f + 1] as usize);
        self.corner.get(a..b).unwrap_or(&[])
    }

    /// Every face triangulated as a fan, as corner indices.
    pub fn triangles(&self) -> Vec<[usize; 3]> {
        let mut out = Vec::new();
        for f in 0..self.faces() {
            let (a, b) = (self.starts[f] as usize, self.starts[f + 1] as usize);
            for k in a + 1..b.saturating_sub(1) {
                out.push([a, k, k + 1]);
            }
        }
        out
    }
}

/// How to unwrap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BUnwrap {
    /// `uv.unwrap(method='CONFORMAL')`.
    Conformal,
    /// `uv.cube_project`.
    Cube,
    /// `uv.cylinder_project`.
    Cylinder,
    /// `uv.sphere_project`.
    Sphere,
    /// `uv.project_from_view`: the view matrix's own axes.
    View([[f32; 3]; 2]),
}

/// A pinned corner: Blender pins a *UV*, which is a corner, not a vertex —
/// the two sides of a seam can be pinned apart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BUvPin {
    pub corner: usize,
    pub uv: [f32; 2],
}

/// Unwrap `mesh` into one UV a corner.
///
/// `seam` is the set of edges the islands are cut along, by vertex pair in
/// either order; `pin` holds the corners the user has pinned. The result is
/// packed into 0..1 for a solver method, as Blender's Unwrap does; a
/// projection is left where the projection put it, as Blender's do.
pub fn unwrap(mesh: BUvMesh<'_>, how: BUnwrap, seam: &[(u32, u32)], pin: &[BUvPin]) -> Vec<[f32; 2]> {
    match how {
        BUnwrap::Conformal => {
            let islands = island::split(mesh, seam);
            let mut uv = vec![[0.0f32; 2]; mesh.corner.len()];
            for part in &islands {
                lscm::solve(mesh, part, pin, &mut uv);
            }
            pack::into_square(mesh, &islands, &mut uv, pin);
            uv
        }
        other => project::project(mesh, other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A quad grid: `n x n` quads over `(n+1)²` vertices on the XY plane.
    pub(super) fn grid(n: usize) -> (Vec<[f32; 3]>, Vec<u32>, Vec<u32>) {
        let mut pos = Vec::new();
        for j in 0..=n {
            for i in 0..=n {
                pos.push([i as f32 / n as f32, j as f32 / n as f32, 0.0]);
            }
        }
        let (mut corner, mut starts) = (Vec::new(), vec![0u32]);
        let stride = (n + 1) as u32;
        for j in 0..n as u32 {
            for i in 0..n as u32 {
                let a = j * stride + i;
                corner.extend_from_slice(&[a, a + 1, a + stride + 1, a + stride]);
                starts.push(corner.len() as u32);
            }
        }
        (pos, corner, starts)
    }

    #[test]
    fn a_quad_fans_into_two_triangles() {
        let (pos, corner, starts) = grid(1);
        let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
        assert_eq!(mesh.faces(), 1);
        assert_eq!(mesh.face(0), &[0, 1, 3, 2]);
        assert_eq!(mesh.triangles(), vec![[0, 1, 2], [0, 2, 3]]);
    }
}
