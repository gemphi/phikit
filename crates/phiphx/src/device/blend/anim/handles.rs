//! Keyframe insertion and the handles Blender gives keys — `insert_vert_fcurve`
//! then `BKE_fcurve_handles_recalc` (`fcurve.cc`), `calchandleNurb_intern`
//! and `BKE_nurb_handle_smooth_fcurve` (`curve.cc`): auto and auto-clamped
//! handles a third of the way to each neighbour, flat at an extreme and at
//! the first and last key, and between them the smoothest curve through
//! the keys (Continuous Acceleration, Blender's default smoothing), solved
//! as Blender solves it — a tridiagonal system whose handles are held
//! inside their neighbours' values (`tridiagonal_solve_with_limits`).
//! Blender keeps the handles it wrote; it does not recompute them on load,
//! so a key phiphx writes must carry Blender's own.

use super::fcurve::{BFCurve, BKey, COLOR_AUTO_RAINBOW, COLOR_AUTO_RGB, COLOR_AUTO_YRGB, FLAG_SELECTED, HANDLE_AUTO, HANDLE_AUTO_ANIM, HANDLE_VECT, IPO_BEZ};

/// `FCURVE_VISIBLE`: the curve is drawn in the Graph editor.
pub const FLAG_VISIBLE: u32 = 1;

/// `HD_AUTOTYPE_NORMAL` / `HD_AUTOTYPE_LOCKED_FINAL`: whether the smoothing
/// pass may still move a key's auto handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BHandleLock {
    Normal,
    Locked,
}

/// `BEZT_BINARYSEARCH_THRESH`: keys closer than this share a frame.
const SAME_FRAME: f32 = 0.01;
/// `calchandleNurb_intern`'s F-Curve length with smoothing: `6.0 / 2.5614 × 2.5614`.
const SMOOTH_LEN: f32 = 6.0;

fn is_auto(h: u8) -> bool {
    h == HANDLE_AUTO || h == HANDLE_AUTO_ANIM
}

impl BFCurve {
    /// A curve as `I` makes one (`ED_action_fcurve_ensure`): shown and
    /// selected, coloured by its property — X, Y, Z red, green, blue for a
    /// vector, W yellow first for a rotation of four (`FCURVE_COLOR_AUTO_RGB`
    /// / `_YRGB`), a rainbow for anything else.
    pub fn keyed(path: impl Into<String>, index: u32) -> Self {
        let mut curve = Self::new(path, index, Vec::new());
        curve.flag = FLAG_VISIBLE | FLAG_SELECTED;
        curve.color_mode = match curve.property() {
            "location" | "rotation_euler" | "scale" | "delta_location" | "delta_rotation_euler" | "delta_scale" => COLOR_AUTO_RGB,
            "rotation_quaternion" | "rotation_axis_angle" | "delta_rotation_quaternion" => COLOR_AUTO_YRGB,
            _ => COLOR_AUTO_RAINBOW,
        };
        curve
    }

    /// `insert_vert_fcurve`: a key at `frame` with `value` — a new Bézier key
    /// with auto-clamped handles, selected, or the value of the key already
    /// there — then the handles recomputed.
    pub fn insert(&mut self, frame: f32, value: f32) {
        match self.keys.iter().position(|k| (k.co[0] - frame).abs() < SAME_FRAME) {
            Some(i) => {
                let dy = value - self.keys[i].co[1];
                let key = &mut self.keys[i];
                key.co[1] = value;
                key.h1[1] += dy;
                key.h2[1] += dy;
            }
            None => {
                let mut key = BKey::new([frame - 1.0, value], [frame, value], [frame + 1.0, value], IPO_BEZ);
                (key.selected, key.h1_selected, key.h2_selected) = (true, true, true);
                let at = self.keys.iter().position(|k| k.co[0] > frame).unwrap_or(self.keys.len());
                self.keys.insert(at, key);
            }
        }
        self.recalc_handles();
    }

    /// `BKE_fcurve_handles_recalc` with Continuous Acceleration smoothing
    /// and constant extrapolation.
    pub fn recalc_handles(&mut self) {
        let n = self.keys.len();
        if n < 2 {
            return;
        }
        let mut locks = vec![BHandleLock::Normal; n];
        for i in 0..n {
            let key = &mut self.keys[i];
            key.h1[0] = key.h1[0].min(key.co[0]);
            key.h2[0] = key.h2[0].max(key.co[0]);
            let prev = (i > 0).then(|| self.keys[i - 1].co);
            let next = (i + 1 < n).then(|| self.keys[i + 1].co);
            locks[i] = Self::calc_handle(&mut self.keys[i], prev, next);
            let key = &mut self.keys[i];
            // The first and last auto key hold their value either side.
            if is_auto(key.h1_type) && is_auto(key.h2_type) && (i == 0 || i == n - 1) && self.extend == super::fcurve::EXTRAPOLATE_CONSTANT {
                key.h1[1] = key.co[1];
                key.h2[1] = key.co[1];
                locks[i] = BHandleLock::Locked;
            }
            if i > 0 && self.keys[i - 1].co[0] >= self.keys[i].co[0] {
                locks[i - 1] = BHandleLock::Locked;
                locks[i] = BHandleLock::Locked;
            }
        }
        super::smooth::BSmooth::fcurve(&mut self.keys, &locks.iter().map(|l| *l == BHandleLock::Normal).collect::<Vec<_>>());
    }

    /// `calchandleNurb_intern` for one F-Curve key: its auto handles along
    /// the neighbours' slope, a third of each interval long, clamped flat at
    /// an extreme and inside the neighbours' values.
    fn calc_handle(key: &mut BKey, prev: Option<[f32; 2]>, next: Option<[f32; 2]>) -> BHandleLock {
        let mut lock = BHandleLock::Normal;
        if key.h1_type == 0 && key.h2_type == 0 {
            return lock;
        }
        let p2 = key.co;
        let p1 = prev.unwrap_or_else(|| {
            let p3 = next.unwrap_or(p2);
            [2.0 * p2[0] - p3[0], 2.0 * p2[1] - p3[1]]
        });
        let p3 = next.unwrap_or([2.0 * p2[0] - p1[0], 2.0 * p2[1] - p1[1]]);
        let (dvec_a, dvec_b) = ([p2[0] - p1[0], p2[1] - p1[1]], [p3[0] - p2[0], p3[1] - p2[1]]);
        let len_a = if dvec_a[0] == 0.0 { 1.0 } else { dvec_a[0] };
        let len_b = if dvec_b[0] == 0.0 { 1.0 } else { dvec_b[0] };
        if !(is_auto(key.h1_type) || is_auto(key.h2_type)) {
            return lock;
        }
        let tvec = [dvec_b[0] / len_b + dvec_a[0] / len_a, dvec_b[1] / len_b + dvec_a[1] / len_a];
        let len = SMOOTH_LEN;
        let (mut left_violate, mut right_violate) = (false, false);
        let extreme = |prev: [f32; 2], next: [f32; 2]| {
            let (d1, d2) = (prev[1] - p2[1], next[1] - p2[1]);
            ((d1 <= 0.0 && d2 <= 0.0) || (d1 >= 0.0 && d2 >= 0.0), d1)
        };
        if is_auto(key.h1_type) {
            let a = len_a / len;
            key.h1 = [p2[0] - tvec[0] * a, p2[1] - tvec[1] * a];
            if let (HANDLE_AUTO_ANIM, Some(prev), Some(next)) = (key.h1_type, prev, next) {
                match extreme(prev, next) {
                    (true, _) => {
                        key.h1[1] = p2[1];
                        lock = BHandleLock::Locked;
                    }
                    (false, d1) if (d1 <= 0.0 && prev[1] > key.h1[1]) || (d1 > 0.0 && prev[1] < key.h1[1]) => {
                        key.h1[1] = prev[1];
                        left_violate = true;
                    }
                    _ => {}
                }
            }
        }
        if is_auto(key.h2_type) {
            let b = len_b / len;
            key.h2 = [p2[0] + tvec[0] * b, p2[1] + tvec[1] * b];
            if let (HANDLE_AUTO_ANIM, Some(prev), Some(next)) = (key.h2_type, prev, next) {
                match extreme(prev, next) {
                    (true, _) => {
                        key.h2[1] = p2[1];
                        lock = BHandleLock::Locked;
                    }
                    (false, d1) if (d1 <= 0.0 && next[1] < key.h2[1]) || (d1 > 0.0 && next[1] > key.h2[1]) => {
                        key.h2[1] = next[1];
                        right_violate = true;
                    }
                    _ => {}
                }
            }
        }
        if left_violate || right_violate {
            let (h1_x, h2_x) = (key.h1[0] - p2[0], p2[0] - key.h2[0]);
            if left_violate {
                key.h2[1] = p2[1] + ((p2[1] - key.h1[1]) / h1_x) * h2_x;
            } else {
                key.h1[1] = p2[1] + ((p2[1] - key.h2[1]) / h2_x) * h1_x;
            }
        }
        let _ = HANDLE_VECT;
        lock
    }
}
