//! Where a thing stands relative to its parent: a move, a turn and a
//! size, as USD writes them (`xformOp:translate`, `xformOp:orient`,
//! `xformOp:scale`, in that order), and the matrix they make.

use crate::aspect::Aspect;

/// A turn, as a unit quaternion: its real part first, as USD writes it.
pub type Turn = [f32; 4];

/// A thing's place relative to its parent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub translate: [f32; 3],
    pub orient: Turn,
    pub scale: [f32; 3],
}

impl Default for Pose {
    fn default() -> Self {
        Self { translate: [0.0; 3], orient: [1.0, 0.0, 0.0, 0.0], scale: [1.0; 3] }
    }
}

impl Aspect for Pose {}

/// A 4x4 matrix, by rows; a point is a column on its right.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix(pub [[f32; 4]; 4]);

impl Default for Matrix {
    fn default() -> Self {
        let mut m = [[0.0; 4]; 4];
        (0..4).for_each(|i| m[i][i] = 1.0);
        Self(m)
    }
}

impl Matrix {
    /// This matrix, then `other` inside it: `self * other`.
    pub fn then(&self, other: &Matrix) -> Matrix {
        let mut out = [[0.0; 4]; 4];
        for (r, row) in out.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
                *cell = (0..4).map(|k| self.0[r][k] * other.0[k][c]).sum();
            }
        }
        Matrix(out)
    }

    /// Where a point lands.
    pub fn apply(&self, p: [f32; 3]) -> [f32; 3] {
        let m = &self.0;
        let row = |r: usize| m[r][0] * p[0] + m[r][1] * p[1] + m[r][2] * p[2] + m[r][3];
        [row(0), row(1), row(2)]
    }

    /// The matrix that undoes this one; none where it cannot be undone.
    pub fn inverse(&self) -> Option<Matrix> {
        let m = &self.0;
        let mut a = [[0.0f64; 8]; 4];
        for (r, row) in a.iter_mut().enumerate() {
            for c in 0..4 {
                row[c] = m[r][c] as f64;
            }
            row[4 + r] = 1.0;
        }
        for c in 0..4 {
            let pivot = (c..4).max_by(|&x, &y| a[x][c].abs().total_cmp(&a[y][c].abs()))?;
            if a[pivot][c].abs() < 1e-12 {
                return None;
            }
            a.swap(c, pivot);
            let lead = a[c][c];
            a[c].iter_mut().for_each(|v| *v /= lead);
            for r in (0..4).filter(|&r| r != c) {
                let f = a[r][c];
                let row = a[c];
                a[r].iter_mut().zip(row).for_each(|(v, w)| *v -= f * w);
            }
        }
        let mut out = [[0.0f32; 4]; 4];
        for (r, row) in out.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
                *cell = a[r][4 + c] as f32;
            }
        }
        Some(Matrix(out))
    }

    /// The same matrix with no move: only its turn and size.
    pub fn still(&self) -> Matrix {
        let mut m = self.0;
        (0..3).for_each(|r| m[r][3] = 0.0);
        Matrix(m)
    }

    /// The point the matrix puts its origin at.
    pub fn origin(&self) -> [f32; 3] {
        [self.0[0][3], self.0[1][3], self.0[2][3]]
    }
}

impl Pose {
    /// A pose that only moves.
    pub fn at(translate: [f32; 3]) -> Pose {
        Pose { translate, ..Pose::default() }
    }

    /// The matrix of the move, then the turn, then the size: a point is
    /// sized first, turned, then moved.
    pub fn matrix(&self) -> Matrix {
        let [w, x, y, z] = normal(self.orient);
        let r = [
            [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - w * z), 2.0 * (x * z + w * y)],
            [2.0 * (x * y + w * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - w * x)],
            [2.0 * (x * z - w * y), 2.0 * (y * z + w * x), 1.0 - 2.0 * (x * x + y * y)],
        ];
        let mut m = Matrix::default().0;
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] = r[i][j] * self.scale[j];
            }
            m[i][3] = self.translate[i];
        }
        Matrix(m)
    }

    /// The turn about the up axis (Y) that faces along `dir` on the
    /// ground: forward is -Z, as RealityKit's characters look.
    pub fn facing(dir: [f32; 2]) -> Turn {
        let angle = (-dir[0]).atan2(-dir[1]);
        [(angle / 2.0).cos(), 0.0, (angle / 2.0).sin(), 0.0]
    }
}

/// The same turn of length one; no turn where it has no length.
fn normal(q: Turn) -> Turn {
    let len = q.iter().map(|v| v * v).sum::<f32>().sqrt();
    match len > f32::EPSILON {
        true => q.map(|v| v / len),
        false => [1.0, 0.0, 0.0, 0.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pose_sizes_then_turns_then_moves() {
        // Half a turn about Y, twice the size, moved one along X.
        let pose = Pose { translate: [1.0, 0.0, 0.0], orient: [0.0, 0.0, 1.0, 0.0], scale: [2.0; 3] };
        let p = pose.matrix().apply([1.0, 0.0, 0.0]);
        assert!((p[0] - -1.0).abs() < 1e-5 && p[1].abs() < 1e-5 && p[2].abs() < 1e-5, "{p:?}");
        // A parent's matrix then a child's: the child moved inside the parent.
        let child = Pose::at([0.0, 0.0, 3.0]).matrix();
        let o = pose.matrix().then(&child).origin();
        assert!((o[0] - 1.0).abs() < 1e-5 && (o[2] - -6.0).abs() < 1e-5, "{o:?}");
    }

    #[test]
    fn a_matrix_and_its_inverse_undo_each_other() {
        let pose = Pose { translate: [1.0, 2.0, 3.0], orient: [0.9, 0.1, 0.3, 0.2], scale: [2.0, 0.5, 3.0] };
        let m = pose.matrix();
        let back = m.inverse().expect("it can be undone").then(&m);
        for (r, row) in back.0.iter().enumerate() {
            for (c, v) in row.iter().enumerate() {
                assert!((v - (r == c) as u8 as f32).abs() < 1e-4, "{back:?}");
            }
        }
    }

    #[test]
    fn facing_forward_is_no_turn() {
        let q = Pose::facing([0.0, -1.0]);
        assert!((q[0] - 1.0).abs() < 1e-6 && q[2].abs() < 1e-6, "{q:?}");
        let along = Pose { orient: Pose::facing([1.0, 0.0]), ..Pose::default() }.matrix().apply([0.0, 0.0, -1.0]);
        assert!((along[0] - 1.0).abs() < 1e-5 && along[2].abs() < 1e-5, "{along:?}");
    }
}
