//! `BLI_tridiagonal_solve` and `tridiagonal_solve_with_limits`
//! (`curve.cc`): the tridiagonal system the Continuous Acceleration
//! smoothing solves for a run of handle heights (`smooth.rs`), in doubles as
//! Blender solves it, then held inside each height's limits.

/// One run's equations: `a[i]·h[i-1] + b[i]·h[i] + c[i]·h[i+1] = d[i]`.
#[derive(Clone)]
pub(super) struct BSystem {
    pub(super) a: Vec<f32>,
    pub(super) b: Vec<f32>,
    pub(super) c: Vec<f32>,
    pub(super) d: Vec<f32>,
}

impl BSystem {
    pub(super) fn new(n: usize) -> Self {
        Self { a: vec![0.0; n], b: vec![0.0; n], c: vec![0.0; n], d: vec![0.0; n] }
    }

    pub(super) fn set(&mut self, i: usize, (a, b, c, d): (f32, f32, f32, f32)) {
        (self.a[i], self.b[i], self.c[i], self.d[i]) = (a, b, c, d);
    }

    /// `bezier_lock_unknown`.
    pub(super) fn lock(&mut self, i: usize, value: f32) {
        self.set(i, (0.0, 1.0, 0.0, value));
    }

    /// `BLI_tridiagonal_solve`: the Thomas algorithm, in doubles.
    fn solve(&self) -> Option<Vec<f32>> {
        let n = self.b.len();
        let (mut c1, mut d1) = (vec![0.0f64; n], vec![0.0f64; n]);
        c1[0] = f64::from(self.c[0]) / f64::from(self.b[0]);
        d1[0] = f64::from(self.d[0]) / f64::from(self.b[0]);
        for i in 1..n {
            let denum = f64::from(self.b[i]) - f64::from(self.a[i]) * c1[i - 1];
            c1[i] = f64::from(self.c[i]) / denum;
            d1[i] = (f64::from(self.d[i]) - f64::from(self.a[i]) * d1[i - 1]) / denum;
        }
        let mut x = vec![0.0f32; n];
        let mut prev = d1[n - 1];
        x[n - 1] = prev as f32;
        for i in (0..n - 1).rev() {
            prev = d1[i] - c1[i] * prev;
            x[i] = prev as f32;
        }
        prev.is_finite().then_some(x)
    }

    /// `bezier_relax_direction`: which way unlocking `i` would move it.
    fn relax(&self, h: &[f32], i: usize) -> f32 {
        let n = h.len();
        let state = self.a[i] * h[(i + n - 1) % n] + self.b[i] * h[i] + self.c[i] * h[(i + 1) % n] - self.d[i];
        -state * self.b[i]
    }

    /// `tridiagonal_solve_with_limits`: solve, lock every height past its
    /// limit at the limit and solve again, and release a lock that would
    /// move back inside — at most twice each.
    pub(super) fn solve_within(mut self, hmin: &[f32], hmax: &[f32]) -> Option<Vec<f32>> {
        let original = self.clone();
        let n = hmin.len();
        let (mut locked_at, mut unlocks) = (vec![false; n], vec![0u8; n]);
        loop {
            let h = self.solve()?;
            let (mut overshoot, mut unlocked, mut locked, mut all) = (false, false, false, false);
            loop {
                for i in 0..n {
                    if h[i] >= hmin[i] && h[i] <= hmax[i] {
                        continue;
                    }
                    overshoot = true;
                    let target = if h[i] > hmax[i] { hmax[i] } else { hmin[i] };
                    // Lock the ones heading the right way first, if there are any.
                    if target != 0.0 || all {
                        locked_at[i] = true;
                        self.lock(i, target);
                        locked = true;
                    }
                }
                all = true;
                if !(overshoot && !locked) {
                    break;
                }
            }
            if !locked {
                for i in 0..n {
                    if !locked_at[i] || unlocks[i] >= 2 {
                        continue;
                    }
                    let relax = original.relax(&h, i);
                    if (relax > 0.0 && h[i] < hmax[i]) || (relax < 0.0 && h[i] > hmin[i]) {
                        self.set(i, (original.a[i], original.b[i], original.c[i], original.d[i]));
                        locked_at[i] = false;
                        unlocks[i] += 1;
                        unlocked = true;
                    }
                }
            }
            if !(overshoot || unlocked) {
                return Some(h);
            }
        }
    }
}
