//! `IK_QJacobian`: the task Jacobian and its selectively damped least
//! squares inverse (`InvertSDLS`, Buss and Kim), over a thin SVD. The SVD
//! is a one-sided Jacobi here (Eigen's `JacobiSVD` in Blender); the SDLS
//! step does not depend on the singular vectors' signs or order.

use glam::DVec3;
use std::f64::consts::FRAC_PI_4;

#[derive(Clone, Debug, Default)]
pub struct BJacobian {
    pub dof: usize,
    pub task_size: usize,
    /// Row-major `task_size × dof`.
    jacobian: Vec<f64>,
    beta: Vec<f64>,
    d_theta: Vec<f64>,
    d_theta_tmp: Vec<f64>,
    d_norm_weight: Vec<f64>,
    norm: Vec<f64>,
    weight: Vec<f64>,
    weight_sqrt: Vec<f64>,
    /// `J = U·diag(w)·Vᵀ`: `u` is `task × r`, `v` is `dof × r`, row-major.
    svd_u: Vec<f64>,
    svd_w: Vec<f64>,
    svd_v: Vec<f64>,
    rank: usize,
}

impl BJacobian {
    /// `ArmMatrices`.
    pub fn arm(&mut self, dof: usize, task_size: usize) {
        *self = Self {
            dof,
            task_size,
            jacobian: vec![0.0; task_size * dof],
            beta: vec![0.0; task_size],
            d_theta: vec![0.0; dof],
            d_theta_tmp: vec![0.0; dof],
            d_norm_weight: vec![0.0; dof],
            norm: vec![0.0; dof],
            weight: vec![1.0; dof],
            weight_sqrt: vec![1.0; dof],
            ..Default::default()
        };
    }

    pub fn set_dof_weight(&mut self, dof: usize, weight: f64) {
        self.weight[dof] = weight;
        self.weight_sqrt[dof] = weight.sqrt();
    }

    pub fn set_betas(&mut self, id: usize, v: DVec3) {
        self.beta[id..id + 3].copy_from_slice(&v.to_array());
    }

    pub fn set_derivatives(&mut self, id: usize, dof_id: usize, v: DVec3, norm_weight: f64) {
        let w = self.weight_sqrt[dof_id];
        for k in 0..3 {
            self.jacobian[(id + k) * self.dof + dof_id] = v[k] * w;
        }
        self.d_norm_weight[dof_id] = norm_weight;
    }

    pub fn angle_update(&self, dof_id: usize) -> f64 {
        self.d_theta[dof_id]
    }

    /// `AngleUpdateNorm`.
    pub fn angle_update_norm(&self) -> f64 {
        self.d_theta.iter().zip(&self.d_norm_weight).map(|(d, w)| (d * w).abs()).fold(0.0, f64::max)
    }

    /// `Lock`: the joint's change fixed at `delta`, taken out of the task.
    pub fn lock(&mut self, dof_id: usize, delta: f64) {
        for i in 0..self.task_size {
            self.beta[i] -= self.jacobian[i * self.dof + dof_id] * delta;
            self.jacobian[i * self.dof + dof_id] = 0.0;
        }
        self.norm[dof_id] = 0.0;
        self.d_theta[dof_id] = 0.0;
    }

    /// `Invert`: the SVD, then `InvertSDLS`.
    pub fn invert(&mut self) {
        self.svd();
        self.invert_sdls();
    }

    /// Thin SVD of the Jacobian by one-sided Jacobi rotations on the side
    /// with more rows.
    fn svd(&mut self) {
        let (m, n) = (self.task_size, self.dof);
        let transpose = m < n;
        let (rows, cols) = if transpose { (n, m) } else { (m, n) };
        // `a[c]` is column c (length `rows`) of the matrix worked on.
        let mut a: Vec<Vec<f64>> = (0..cols).map(|c| (0..rows).map(|r| if transpose { self.jacobian[c * n + r] } else { self.jacobian[r * n + c] }).collect()).collect();
        let mut v: Vec<Vec<f64>> = (0..cols).map(|c| (0..cols).map(|r| f64::from(u8::from(r == c))).collect()).collect();
        for _ in 0..60 {
            let mut rotated = false;
            for p in 0..cols {
                for q in p + 1..cols {
                    let alpha: f64 = a[p].iter().map(|x| x * x).sum();
                    let beta: f64 = a[q].iter().map(|x| x * x).sum();
                    let gamma: f64 = a[p].iter().zip(&a[q]).map(|(x, y)| x * y).sum();
                    if gamma.abs() <= 1e-15 * (alpha * beta).sqrt() || gamma == 0.0 {
                        continue;
                    }
                    rotated = true;
                    let zeta = (beta - alpha) / (2.0 * gamma);
                    let t = zeta.signum() / (zeta.abs() + (1.0 + zeta * zeta).sqrt());
                    let t = if zeta == 0.0 { 1.0 } else { t };
                    let c = 1.0 / (1.0 + t * t).sqrt();
                    let s = c * t;
                    for k in 0..rows {
                        let (x, y) = (a[p][k], a[q][k]);
                        a[p][k] = c * x - s * y;
                        a[q][k] = s * x + c * y;
                    }
                    for k in 0..cols {
                        let (x, y) = (v[p][k], v[q][k]);
                        v[p][k] = c * x - s * y;
                        v[q][k] = s * x + c * y;
                    }
                }
            }
            if !rotated {
                break;
            }
        }
        let w: Vec<f64> = a.iter().map(|c| c.iter().map(|x| x * x).sum::<f64>().sqrt()).collect();
        // Left vectors: the columns normalised (`rows × cols`); right: `v`.
        let left: Vec<Vec<f64>> = a.iter().zip(&w).map(|(c, s)| if *s > 0.0 { c.iter().map(|x| x / s).collect() } else { vec![0.0; rows] }).collect();
        let r = cols;
        let (u_cols, v_cols) = if transpose { (&v, &left) } else { (&left, &v) };
        // u: task × r, v: dof × r.
        self.svd_u = (0..m).flat_map(|row| (0..r).map(move |k| (row, k))).map(|(row, k)| u_cols[k][row]).collect();
        self.svd_v = (0..n).flat_map(|row| (0..r).map(move |k| (row, k))).map(|(row, k)| v_cols[k][row]).collect();
        self.svd_w = w;
        self.rank = r;
    }

    /// `InvertSDLS`.
    fn invert_sdls(&mut self) {
        let max_angle_change = FRAC_PI_4;
        let epsilon = 1e-10;
        let (m, n, r) = (self.task_size, self.dof, self.rank);
        self.d_theta.iter_mut().for_each(|d| *d = 0.0);
        for i in 0..n {
            self.norm[i] = (0..m)
                .step_by(3)
                .map(|j| (0..3).map(|k| self.jacobian[(j + k) * n + i].powi(2)).sum::<f64>().sqrt())
                .sum();
        }
        for i in 0..r {
            if self.svd_w[i] <= epsilon {
                continue;
            }
            let w_inv = 1.0 / self.svd_w[i];
            let (mut alpha, mut big_n) = (0.0, 0.0);
            for j in (0..m).step_by(3) {
                let mut tmp = 0.0;
                for k in 0..3 {
                    let u = self.svd_u[(j + k) * r + i];
                    alpha += u * self.beta[j + k];
                    tmp += u * u;
                }
                big_n += tmp.sqrt();
            }
            alpha *= w_inv;
            let (mut big_m, mut max_dtheta) = (0.0, 0.0f64);
            for j in 0..n {
                let v = self.svd_v[j * r + i];
                big_m += v.abs() * self.norm[j];
                self.d_theta_tmp[j] = v * alpha;
                max_dtheta = max_dtheta.max(self.d_theta_tmp[j].abs() * self.weight_sqrt[j]);
            }
            big_m *= w_inv;
            let mut gamma = max_angle_change;
            if big_n < big_m {
                gamma *= big_n / big_m;
            }
            let damp = if gamma < max_dtheta { gamma / max_dtheta } else { 1.0 };
            for j in 0..n {
                let dofdamp = (damp / self.weight[j]).min(1.0);
                self.d_theta[j] += 0.80 * dofdamp * self.d_theta_tmp[j];
            }
        }
        let mut max_angle = 0.0f64;
        for j in 0..n {
            self.d_theta[j] *= self.weight[j];
            max_angle = max_angle.max(self.d_theta[j].abs());
        }
        if max_angle > max_angle_change {
            let damp = max_angle_change / (max_angle_change + max_angle);
            self.d_theta.iter_mut().for_each(|d| *d *= damp);
        }
    }
}
