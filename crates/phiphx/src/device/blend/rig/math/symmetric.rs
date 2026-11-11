//! Eigen-decomposition of a symmetric 3x3 matrix, for the polar split.
use glam::{DMat3, DVec3};

/// A symmetric 3×3 eigen-decomposition by Jacobi rotations (double).
pub(super) struct BSymmetric;

impl BSymmetric {
    /// Eigenvalues and the eigenvectors as columns (cyclic Jacobi:
    /// `A ← Jᵀ·A·J` for each off-diagonal pair until it is diagonal).
    pub(super) fn eigen(m: DMat3) -> (DVec3, DMat3) {
        // Row-major: `a[r][c]`; `v`'s columns become the eigenvectors.
        let mut a: [[f64; 3]; 3] = std::array::from_fn(|r| std::array::from_fn(|c| m.col(c)[r]));
        let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        for _ in 0..64 {
            if a[0][1].abs() + a[0][2].abs() + a[1][2].abs() < 1e-15 {
                break;
            }
            for (p, q) in [(0usize, 1usize), (0, 2), (1, 2)] {
                if a[p][q].abs() < 1e-300 {
                    continue;
                }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = if theta == 0.0 { 1.0 } else { theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt()) };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for row in a.iter_mut() {
                    let (kp, kq) = (row[p], row[q]);
                    row[p] = c * kp - s * kq;
                    row[q] = s * kp + c * kq;
                }
                for k in 0..3 {
                    let (pk, qk) = (a[p][k], a[q][k]);
                    a[p][k] = c * pk - s * qk;
                    a[q][k] = s * pk + c * qk;
                }
                for row in v.iter_mut() {
                    let (kp, kq) = (row[p], row[q]);
                    row[p] = c * kp - s * kq;
                    row[q] = s * kp + c * kq;
                }
            }
        }
        (DVec3::new(a[0][0], a[1][1], a[2][2]), DMat3::from_cols_array_2d(&v).transpose())
    }
}
