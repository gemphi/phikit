//! Continuous Acceleration smoothing — `BKE_nurb_handle_smooth_fcurve` and
//! `bezier_handle_calc_smooth_fcurve` (`curve.cc`), non-cyclic: each run of
//! free auto keys between two fixed ones gets the handle heights that keep
//! the curve's second derivative continuous through it. With every handle a
//! third of its interval wide, a handle's height `h` is the unknown of a
//! tridiagonal system — continuity at each free key, a fixed handle or zero
//! curvature at each end — solved and then held inside the neighbours'
//! values (`tridiagonal_solve_with_limits`, the auto-clamp).

use super::fcurve::{BKey, HANDLE_AUTO, HANDLE_AUTO_ANIM, HANDLE_VECT};
use super::tridiagonal::BSystem;

/// The smoothing pass over one curve's keys.
pub struct BSmooth;

impl BSmooth {
    fn auto(h: u8) -> bool {
        h == HANDLE_AUTO || h == HANDLE_AUTO_ANIM
    }

    /// `BKE_nurb_handle_smooth_fcurve`, non-cyclic: every run from one key
    /// that is not free (`free[i]`: both handles auto and not locked by the
    /// first pass) to the next.
    pub fn fcurve(keys: &mut [BKey], free: &[bool]) {
        let total = keys.len();
        let is_free = |keys: &[BKey], i: usize| free[i] && Self::auto(keys[i].h1_type) && Self::auto(keys[i].h2_type);
        let (mut start, mut count) = (0usize, 1usize);
        for j in 1..total {
            if !is_free(keys, j) {
                Self::run(keys, start, count + 1, free);
                start = j;
                count = 1;
            } else {
                count += 1;
            }
        }
        if count > 1 {
            Self::run(keys, start, count, free);
        }
    }

    /// `bezier_check_solve_end_handle`.
    fn solves_end(htype: u8, end: bool, free: bool) -> bool {
        htype == HANDLE_VECT || (end && Self::auto(htype) && free)
    }

    /// `bezier_calc_handle_adj`: a fixed handle wider than a third of its
    /// interval, scaled to fit, as the correction it adds.
    fn handle_adj(hsize: &mut [f32; 2], dx: f32) -> f32 {
        let fac = dx / (hsize[0] + dx / 3.0);
        if fac < 1.0 {
            hsize[0] *= fac;
            hsize[1] *= fac;
        }
        1.0 - 3.0 * hsize[0] / dx
    }

    /// `bezier_clamp`: auto-clamp keeps a key's handle from reversing and its
    /// neighbour's from overshooting.
    fn clamp(hmax: &mut [f32], hmin: &mut [f32], i: usize, dy: f32, no_reverse: bool, no_overshoot: bool) {
        if dy > 0.0 {
            if no_overshoot {
                hmax[i] = hmax[i].min(dy);
            }
            if no_reverse {
                hmin[i] = 0.0;
            }
        } else if dy < 0.0 {
            if no_reverse {
                hmax[i] = 0.0;
            }
            if no_overshoot {
                hmin[i] = hmin[i].max(dy);
            }
        } else if no_reverse || no_overshoot {
            hmax[i] = 0.0;
            hmin[i] = 0.0;
        }
    }

    /// `bezier_output_handle`: one handle `dy` above its key; at the curve's
    /// ends the other handle mirrors it.
    fn output(key: &mut BKey, right: bool, dy: f32, endpoint: bool) {
        let (mine, other) = if right { (key.h2_type, key.h1_type) } else { (key.h1_type, key.h2_type) };
        if !(Self::auto(mine) || mine == HANDLE_VECT) {
            return;
        }
        let co = key.co;
        let handle = if right { &mut key.h2 } else { &mut key.h1 };
        handle[1] = co[1] + dy;
        let set = *handle;
        if endpoint && (Self::auto(other) || other == HANDLE_VECT) {
            let mirrored = [2.0 * co[0] - set[0], 2.0 * co[1] - set[1]];
            if right { key.h1 = mirrored } else { key.h2 = mirrored }
        }
    }

    /// `bezier_handle_calc_smooth_fcurve` over `keys[start .. start + count]`.
    fn run(keys: &mut [BKey], start: usize, count: usize, free: &[bool]) {
        if count < 2 {
            return;
        }
        let total = keys.len();
        let (first, last) = (start, start + count - 1);
        let solve_first = Self::solves_end(keys[first].h2_type, start == 0, free[first]);
        let solve_last = Self::solves_end(keys[last].h1_type, start + count == total, free[last]);
        if count == 2 && solve_first == solve_last {
            return;
        }
        let (mut dx, mut dy) = (vec![0.0f32; count], vec![0.0f32; count]);
        for i in 1..count {
            let j = start + i;
            dx[i] = keys[j].co[0] - keys[j - 1].co[0];
            dy[i] = keys[j].co[1] - keys[j - 1].co[1];
        }
        let mut l = vec![1.0f32; count];
        for i in 1..count - 1 {
            l[i] = dx[i + 1] / dx[i];
        }
        let (mut hmax, mut hmin) = (vec![f32::MAX; count], vec![-f32::MAX; count]);
        let clamped = |k: &BKey| k.h1_type == HANDLE_AUTO_ANIM || k.h2_type == HANDLE_AUTO_ANIM;
        let mut clamped_cur = clamped(&keys[first]);
        for i in 1..count {
            let clamped_prev = clamped_cur;
            clamped_cur = clamped(&keys[start + i]);
            Self::clamp(&mut hmax, &mut hmin, i - 1, dy[i], clamped_prev, clamped_prev);
            Self::clamp(&mut hmax, &mut hmin, i, dy[i] * l[i], clamped_cur, clamped_cur);
        }
        let mut sys = BSystem::new(count);
        let continuous = |i: usize| (l[i] * l[i], 2.0 * (l[i] + 1.0), 1.0 / l[i + 1], dy[i] * l[i] * l[i] + dy[i + 1]);
        let (mut first_adj, mut last_adj) = (0.0f32, 0.0f32);
        if !solve_first {
            let k = &keys[first];
            let mut tmp = [k.h2[0] - k.co[0], k.h2[1] - k.co[1]];
            first_adj = Self::handle_adj(&mut tmp, dx[1]);
            sys.lock(0, tmp[1]);
        } else {
            sys.set(0, (0.0, 2.0, 1.0 / l[1], dy[1]));
        }
        if !solve_last {
            let k = &keys[last];
            let mut tmp = [k.co[0] - k.h1[0], k.co[1] - k.h1[1]];
            last_adj = Self::handle_adj(&mut tmp, dx[count - 1]);
            sys.lock(count - 1, tmp[1]);
        } else {
            let i = count - 1;
            sys.set(i, (l[i] * l[i], 2.0 * l[i], 0.0, dy[i] * l[i] * l[i]));
        }
        for i in 1..count - 1 {
            sys.set(i, continuous(i));
        }
        if count > 2 || solve_last {
            sys.b[1] += l[1] * first_adj;
        }
        if count > 2 || solve_first {
            sys.b[count - 2] += last_adj;
        }
        let Some(h) = sys.solve_within(&hmin, &hmax) else { return };
        for i in 1..count - 1 {
            let j = start + i;
            let end = j == total - 1;
            Self::output(&mut keys[j], false, -h[i] / l[i], end);
            Self::output(&mut keys[j], true, h[i], end);
        }
        if solve_first {
            Self::output(&mut keys[first], true, h[0], start == 0);
        }
        if solve_last {
            Self::output(&mut keys[last], false, -h[count - 1] / l[count - 1], start + count == total);
        }
    }
}
