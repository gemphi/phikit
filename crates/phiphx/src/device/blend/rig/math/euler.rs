//! Euler rotations in Blender's six orders (`math_rotation_c.cc`).
use super::vec::BMat3;
use glam::{Mat3, Mat4, Vec3};

/// A rotation order, `eRotationModes` 1–6 (`ROT_MODE_XYZ` … `ZYX`); the
/// quaternion and axis-angle modes (0, −1) read as XYZ, as Blender's
/// `get_rotation_order_info` does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BEulerOrder(pub i16);

/// `EULER_HYPOT_EPSILON`.
const EULER_HYPOT_EPSILON: f32 = 0.000_037_5;

impl BEulerOrder {
    /// `rotOrders[]`: the axes in order, and the parity.
    fn info(self) -> ([usize; 3], bool) {
        match self.0 {
            2 => ([0, 2, 1], true),
            3 => ([1, 0, 2], true),
            4 => ([1, 2, 0], false),
            5 => ([2, 0, 1], false),
            6 => ([2, 1, 0], true),
            _ => ([0, 1, 2], false),
        }
    }

    /// `eulO_to_mat3` (worked in double, as Blender does).
    pub fn to_mat3(self, e: Vec3) -> Mat3 {
        let ([i, j, k], parity) = self.info();
        let s = if parity { -1.0 } else { 1.0 };
        let (ti, tj, th) = (s * e[i] as f64, s * e[j] as f64, s * e[k] as f64);
        let (ci, cj, ch) = (ti.cos(), tj.cos(), th.cos());
        let (si, sj, sh) = (ti.sin(), tj.sin(), th.sin());
        let (cc, cs, sc, ss) = (ci * ch, ci * sh, si * ch, si * sh);
        let mut m = [[0.0f64; 3]; 3];
        m[i][i] = cj * ch;
        m[j][i] = sj * sc - cs;
        m[k][i] = sj * cc + ss;
        m[i][j] = cj * sh;
        m[j][j] = sj * ss + cc;
        m[k][j] = sj * cs - sc;
        m[i][k] = -sj;
        m[j][k] = cj * si;
        m[k][k] = cj * ci;
        Mat3::from_cols_array_2d(&m.map(|c| c.map(|v| v as f32)))
    }

    /// `mat3_normalized_to_eulo2`: both Euler solutions of a unit matrix.
    fn both(self, m: &Mat3) -> (Vec3, Vec3) {
        let ([i, j, k], parity) = self.info();
        let at = |c: usize, r: usize| m.col(c)[r];
        let cy = at(i, i).hypot(at(i, j));
        let (mut a, mut b) = (Vec3::ZERO, Vec3::ZERO);
        if cy > EULER_HYPOT_EPSILON {
            a[i] = at(j, k).atan2(at(k, k));
            a[j] = (-at(i, k)).atan2(cy);
            a[k] = at(i, j).atan2(at(i, i));
            b[i] = (-at(j, k)).atan2(-at(k, k));
            b[j] = (-at(i, k)).atan2(-cy);
            b[k] = (-at(i, j)).atan2(-at(i, i));
        } else {
            a[i] = (-at(k, j)).atan2(at(j, j));
            a[j] = (-at(i, k)).atan2(cy);
            a[k] = 0.0;
            b = a;
        }
        if parity { (-a, -b) } else { (a, b) }
    }

    /// `mat3_normalized_to_eulO`: the solution with the smaller angles.
    pub fn of_normalized(self, m: &Mat3) -> Vec3 {
        let (a, b) = self.both(m);
        if a.abs().element_sum() > b.abs().element_sum() { b } else { a }
    }

    /// `mat4_to_eulO`.
    pub fn of_mat4(self, m: &Mat4) -> Vec3 {
        self.of_normalized(&Mat3::from_mat4(*m).normalized_cols())
    }

    /// `mat4_to_compatible_eulO`: the solution nearest `old`.
    pub fn compatible_of_mat4(self, old: Vec3, m: &Mat4) -> Vec3 {
        let (a, b) = self.both(&Mat3::from_mat4(*m).normalized_cols());
        let (a, b) = (Self::compatible(a, old), Self::compatible(b, old));
        if (a - old).abs().element_sum() > (b - old).abs().element_sum() { b } else { a }
    }

    /// `compatible_eul`: `eul` wrapped by whole turns towards `old`.
    pub fn compatible(mut eul: Vec3, old: Vec3) -> Vec3 {
        let (pi, pi_x2) = (std::f32::consts::PI, 2.0 * std::f32::consts::PI);
        let mut d = Vec3::ZERO;
        for i in 0..3 {
            d[i] = eul[i] - old[i];
            if d[i] > pi {
                eul[i] -= ((d[i] / pi_x2) + 0.5).floor() * pi_x2;
                d[i] = eul[i] - old[i];
            } else if d[i] < -pi {
                eul[i] += ((-d[i] / pi_x2) + 0.5).floor() * pi_x2;
                d[i] = eul[i] - old[i];
            }
        }
        let (mut i, mut j, mut k) = (0usize, 1usize, 2usize);
        while i < 3 {
            let half = std::f32::consts::FRAC_PI_2;
            if d[i].abs() > pi && d[j].abs() < half && d[k].abs() < half {
                eul[i] += if d[i] > 0.0 { -pi_x2 } else { pi_x2 };
            }
            // `for (i = 0; i < 3; j = k, k = i++)`.
            j = k;
            k = i;
            i += 1;
        }
        eul
    }
}
