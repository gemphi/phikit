//! Where a word is: `Pose` (local) and `WorldMatrix` (derived), as plain f32
//! arrays so the core takes no math crate.
//!
//! Apple's `Transform { scale, rotation, translation }` (RF:15094-15137) is
//! the model. Matrices are column-major, `m[column][row]`, as simd's are, so
//! they hand to a renderer unchanged.
use crate::comp::aspect::Aspect;

/// A place relative to the parent: translation, rotation as a quaternion
/// `[x, y, z, w]`, and scale. Written by animation, physics and the user.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Pose {
    pub t: [f32; 3],
    pub r: [f32; 4],
    pub s: [f32; 3],
}

impl Aspect for Pose {}

impl Default for Pose {
    fn default() -> Self {
        Self { t: [0.0; 3], r: [0.0, 0.0, 0.0, 1.0], s: [1.0; 3] }
    }
}

impl Pose {
    pub fn at(t: [f32; 3]) -> Self {
        Self { t, ..Self::default() }
    }

    /// A turn of `angle` radians about the unit vector `axis`.
    pub fn turned(axis: [f32; 3], angle: f32) -> [f32; 4] {
        let (s, c) = (angle / 2.0).sin_cos();
        [axis[0] * s, axis[1] * s, axis[2] * s, c]
    }

    /// `T x R x S` as a matrix.
    pub fn matrix(&self) -> WorldMatrix {
        let [x, y, z, w] = self.r;
        let cols = [
            [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + z * w), 2.0 * (x * z - y * w)],
            [2.0 * (x * y - z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + x * w)],
            [2.0 * (x * z + y * w), 2.0 * (y * z - x * w), 1.0 - 2.0 * (x * x + y * y)],
        ];
        let mut m = WorldMatrix::IDENTITY;
        for j in 0..3 {
            for i in 0..3 {
                m.0[j][i] = cols[j][i] * self.s[j];
            }
            m.0[3][j] = self.t[j];
        }
        m
    }
}

/// Where a word is in the world: its parents' poses folded down to it. Derived
/// every step by `propagate`, never written by hand.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct WorldMatrix(pub [[f32; 4]; 4]);

impl Aspect for WorldMatrix {}

impl Default for WorldMatrix {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl WorldMatrix {
    pub const IDENTITY: Self = Self([[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]);

    /// `self x other`: `other` applied first.
    pub fn mul(&self, other: &Self) -> Self {
        let (a, b) = (&self.0, &other.0);
        let mut m = [[0.0; 4]; 4];
        for j in 0..4 {
            for i in 0..4 {
                m[j][i] = (0..4).map(|k| a[k][i] * b[j][k]).sum();
            }
        }
        Self(m)
    }

    /// Where the point `p` goes.
    pub fn point(&self, p: [f32; 3]) -> [f32; 3] {
        let m = &self.0;
        std::array::from_fn(|i| m[0][i] * p[0] + m[1][i] * p[1] + m[2][i] * p[2] + m[3][i])
    }

    /// The inverse of an affine matrix (a pose, a product of poses); `None`
    /// when a scale is zero.
    pub fn inverse(&self) -> Option<Self> {
        let m = &self.0;
        let c = |j: usize, i: usize| m[j][i];
        // Cofactors of the 3x3 block, laid out as the inverse's columns.
        let cof = |a: usize, b: usize| {
            let (a1, a2, b1, b2) = ((a + 1) % 3, (a + 2) % 3, (b + 1) % 3, (b + 2) % 3);
            c(a1, b1) * c(a2, b2) - c(a2, b1) * c(a1, b2)
        };
        let det = c(0, 0) * cof(0, 0) + c(1, 0) * cof(1, 0) + c(2, 0) * cof(2, 0);
        if det.abs() < 1e-12 {
            return None;
        }
        let mut out = Self::IDENTITY;
        for j in 0..3 {
            for i in 0..3 {
                out.0[j][i] = cof(i, j) / det;
            }
        }
        let t = [m[3][0], m[3][1], m[3][2]];
        for i in 0..3 {
            out.0[3][i] = -(0..3).map(|k| out.0[k][i] * t[k]).sum::<f32>();
        }
        Some(out)
    }

    /// The `Pose` that makes this matrix. Exact for a rotation, a scale and a
    /// translation; a shear (a rotation under a non-uniform scale) is lost.
    pub fn pose(&self) -> Pose {
        let m = &self.0;
        let len = |j: usize| (m[j][0] * m[j][0] + m[j][1] * m[j][1] + m[j][2] * m[j][2]).sqrt();
        let mut s = [len(0), len(1), len(2)];
        let r: [[f32; 3]; 3] = std::array::from_fn(|j| std::array::from_fn(|i| if s[j] > 0.0 { m[j][i] / s[j] } else { 0.0 }));
        let det = r[0][0] * (r[1][1] * r[2][2] - r[2][1] * r[1][2]) - r[1][0] * (r[0][1] * r[2][2] - r[2][1] * r[0][2])
            + r[2][0] * (r[0][1] * r[1][2] - r[1][1] * r[0][2]);
        let flip = if det < 0.0 { -1.0 } else { 1.0 };
        s[0] *= flip;
        let r: [[f32; 3]; 3] = std::array::from_fn(|j| std::array::from_fn(|i| r[j][i] * if j == 0 { flip } else { 1.0 }));
        Pose { t: [m[3][0], m[3][1], m[3][2]], r: quat(&r), s }
    }
}

/// The quaternion `[x, y, z, w]` of a rotation matrix given by its columns.
fn quat(r: &[[f32; 3]; 3]) -> [f32; 4] {
    let trace = r[0][0] + r[1][1] + r[2][2];
    let q = if trace > 0.0 {
        let k = (trace + 1.0).sqrt() * 2.0;
        [(r[1][2] - r[2][1]) / k, (r[2][0] - r[0][2]) / k, (r[0][1] - r[1][0]) / k, k / 4.0]
    } else if r[0][0] >= r[1][1] && r[0][0] >= r[2][2] {
        let k = (1.0 + r[0][0] - r[1][1] - r[2][2]).sqrt() * 2.0;
        [k / 4.0, (r[1][0] + r[0][1]) / k, (r[2][0] + r[0][2]) / k, (r[1][2] - r[2][1]) / k]
    } else if r[1][1] >= r[2][2] {
        let k = (1.0 + r[1][1] - r[0][0] - r[2][2]).sqrt() * 2.0;
        [(r[1][0] + r[0][1]) / k, k / 4.0, (r[2][1] + r[1][2]) / k, (r[2][0] - r[0][2]) / k]
    } else {
        let k = (1.0 + r[2][2] - r[0][0] - r[1][1]).sqrt() * 2.0;
        [(r[2][0] + r[0][2]) / k, (r[2][1] + r[1][2]) / k, k / 4.0, (r[0][1] - r[1][0]) / k]
    };
    let n = q.iter().map(|v| v * v).sum::<f32>().sqrt();
    q.map(|v| v / n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: [f32; 3], b: [f32; 3]) -> bool {
        a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-4)
    }

    #[test]
    fn a_pose_moves_a_point_scale_then_turn_then_move() {
        let q = Pose::turned([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
        let m = Pose { t: [10.0, 0.0, 0.0], r: q, s: [2.0, 2.0, 2.0] }.matrix();
        assert!(near(m.point([1.0, 0.0, 0.0]), [10.0, 2.0, 0.0]), "{:?}", m.point([1.0, 0.0, 0.0]));
    }

    #[test]
    fn mul_applies_the_right_one_first() {
        let up = Pose::at([0.0, 1.0, 0.0]).matrix();
        let turn = Pose { r: Pose::turned([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2), ..Pose::default() }.matrix();
        assert!(near(turn.mul(&up).point([0.0; 3]), [-1.0, 0.0, 0.0]), "up, then turn");
        assert!(near(up.mul(&turn).point([0.0; 3]), [0.0, 1.0, 0.0]), "turn, then up");
    }

    #[test]
    fn the_inverse_undoes_and_the_pose_comes_back() {
        let p = Pose { t: [1.0, -2.0, 3.0], r: Pose::turned([0.0, 1.0, 0.0], 0.7), s: [1.0, 2.0, 3.0] };
        let m = p.matrix();
        let id = m.mul(&m.inverse().unwrap());
        assert!((0..4).all(|j| (0..4).all(|i| (id.0[j][i] - WorldMatrix::IDENTITY.0[j][i]).abs() < 1e-4)));
        let back = m.pose();
        assert!(near(back.t, p.t) && near(back.s, p.s));
        assert!(back.r.iter().zip(&p.r).all(|(a, b)| (a - b).abs() < 1e-4) || back.r.iter().zip(&p.r).all(|(a, b)| (a + b).abs() < 1e-4));
        assert!(Pose { s: [0.0, 1.0, 1.0], ..Pose::default() }.matrix().inverse().is_none());
    }
}
