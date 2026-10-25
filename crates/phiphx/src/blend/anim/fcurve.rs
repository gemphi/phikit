//! One F-Curve and its evaluation — Blender `fcurve.cc::fcurve_eval_keyframes`
//! for every interpolation a saved file carries: constant, linear, Bézier
//! and the ten easing kinds (`anim/easing.rs`). Outside the keyed range the
//! curve holds its end values (`FCURVE_EXTRAPOLATE_CONSTANT`, the default)
//! or runs on along its end slope (`FCURVE_EXTRAPOLATE_LINEAR`).

/// One keyframe: `BezTriple.vec[0..3]` as (x, y) — left handle, point, right
/// handle — and its interpolation to the next key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BKey {
    pub h1: [f32; 2],
    pub co: [f32; 2],
    pub h2: [f32; 2],
    /// `BEZT_IPO_CONST` 0, `_LIN` 1, `_BEZ` 2.
    pub ipo: u8,
    /// `BezTriple.f2 & SELECT`: the key point is selected in the editors.
    pub selected: bool,
    /// `BezTriple.h1 / h2` (`eBezTriple_Handle`): each handle's type, which
    /// picks its `TH_HANDLE_*` colour.
    pub h1_type: u8,
    pub h2_type: u8,
    /// `BezTriple.f1 / f3 & SELECT`: each handle's own selection.
    pub h1_selected: bool,
    pub h2_selected: bool,
    /// `BezTriple.easing` (`eBezTriple_Easing`), for the easing interpolations.
    pub easing: u8,
    /// `BezTriple.back`, `amplitude`, `period`: the Back and Elastic shapes.
    pub back: f32,
    pub amplitude: f32,
    pub period: f32,
}

/// `FCurve.extend`.
pub const EXTRAPOLATE_CONSTANT: u8 = 0;
pub const EXTRAPOLATE_LINEAR: u8 = 1;
/// `BEZT_IPO_CONST` / `_LIN`.
pub const IPO_CONST: u8 = 0;
pub const IPO_LIN: u8 = 1;

/// `eBezTriple_Handle`.
pub const HANDLE_FREE: u8 = 0;
pub const HANDLE_AUTO: u8 = 1;
pub const HANDLE_VECT: u8 = 2;
pub const HANDLE_ALIGN: u8 = 3;
pub const HANDLE_AUTO_ANIM: u8 = 4;
/// `BEZT_IPO_BEZ`: the interpolation that has handles.
pub const IPO_BEZ: u8 = 2;

impl BKey {
    /// A key with Blender's defaults around its points: auto-clamped
    /// handles (`HD_AUTO_ANIM`, what `insert_vert_fcurve` gives a new key),
    /// nothing selected.
    pub fn new(h1: [f32; 2], co: [f32; 2], h2: [f32; 2], ipo: u8) -> Self {
        Self {
            h1,
            co,
            h2,
            ipo,
            selected: false,
            h1_type: HANDLE_AUTO_ANIM,
            h2_type: HANDLE_AUTO_ANIM,
            h1_selected: false,
            h2_selected: false,
            // `DNA_curve_defaults.h`: the Back overshoot and the Elastic
            // amplitude and period a new key carries.
            easing: 0,
            back: 1.70158,
            amplitude: 0.8,
            period: 4.1,
        }
    }

    /// `BEZT_ISSEL_ANY`: the point or either handle is selected.
    pub fn any_selected(&self) -> bool {
        self.selected || self.h1_selected || self.h2_selected
    }
}

/// `FCurve.color_mode`: how the Graph editor colours the curve.
pub const COLOR_AUTO_RAINBOW: u32 = 0;
pub const COLOR_AUTO_RGB: u32 = 1;
pub const COLOR_CUSTOM: u32 = 2;
pub const COLOR_AUTO_YRGB: u32 = 3;
/// `FCurve.flag` bits the editors read.
pub const FLAG_SELECTED: u32 = 1 << 1;
pub const FLAG_ACTIVE: u32 = 1 << 2;
pub const FLAG_PROTECTED: u32 = 1 << 3;
pub const FLAG_MUTED: u32 = 1 << 4;

#[derive(Clone, Debug, PartialEq)]
pub struct BFCurve {
    /// `rna_path`, e.g. `location` or `pose.bones["chest"].rotation_quaternion`.
    pub path: String,
    pub index: u32,
    pub keys: Vec<BKey>,
    /// `FCurve.flag`: selected / active / protected / muted.
    pub flag: u32,
    /// `FCurve.color_mode` and the `color` a custom mode keeps.
    pub color_mode: u32,
    pub color: [f32; 3],
    /// A Generator modifier in expanded-polynomial mode
    /// (`FMod_Generator.coefficients`, lowest power first) — what a fresh
    /// driver draws as its `y = x` line.
    pub generator: Option<Vec<f32>>,
    /// `FCurve.extend`: how the curve continues past its keys.
    pub extend: u8,
    /// `FCurve.grp → bActionGroup.name`: the channel group the editors list
    /// it under — a bone's name for its pose curves, `Object Transforms` for
    /// the object's own. `None` for an ungrouped curve.
    pub group: Option<String>,
}

impl BFCurve {
    /// A curve with Blender's defaults for everything but its keys:
    /// unselected, rainbow-coloured.
    pub fn new(path: impl Into<String>, index: u32, keys: Vec<BKey>) -> Self {
        Self { path: path.into(), index, keys, flag: 0, color_mode: COLOR_AUTO_RAINBOW, color: [0.0; 3], generator: None, extend: EXTRAPOLATE_CONSTANT, group: None }
    }

    /// The generator polynomial at `x`, when the curve carries one
    /// (`fcm_generator_evaluate`, `FCM_GENERATOR_POLYNOMIAL`).
    pub fn generated(&self, x: f32) -> Option<f32> {
        let coefficients = self.generator.as_ref()?;
        Some(coefficients.iter().enumerate().map(|(power, c)| c * x.powi(power as i32)).sum())
    }

    /// The bone this curve drives, when its path is a pose-bone path.
    pub fn bone(&self) -> Option<&str> {
        let rest = self.path.strip_prefix("pose.bones[\"")?;
        rest.split("\"]").next()
    }

    pub fn selected(&self) -> bool {
        self.flag & FLAG_SELECTED != 0
    }

    /// The property at the end of the path: `location`, `rotation_euler`, …
    pub fn property(&self) -> &str {
        self.path.rsplit("].").next().unwrap_or(&self.path).rsplit('.').next().unwrap_or(&self.path)
    }

    /// The owner's own property this curve drives (`location`,
    /// `rotation_euler`, …), or `None` for a nested path — a bone's channel
    /// (`pose.bones["arm"].location`) or a constraint's setting — which
    /// drives something inside the owner, not the owner.
    pub fn own_property(&self) -> Option<&str> {
        (!self.path.contains(['.', '['])).then_some(self.path.as_str())
    }

    /// `evaluate_fcurve`: the keys' value — nothing keyed evaluates to 0 —
    /// then the modifiers: a Generator replaces it.
    pub fn evaluate(&self, x: f32) -> f32 {
        self.generated(x).unwrap_or_else(|| self.eval(x).unwrap_or(0.0))
    }

    /// `FCurve.flag & FCURVE_MUTED`: the curve is switched off in the editors.
    pub fn muted(&self) -> bool {
        self.flag & FLAG_MUTED != 0
    }

    /// `fcurve_eval_keyframes`: the keyed value at `frame`, none without keys.
    pub fn eval(&self, frame: f32) -> Option<f32> {
        let keys = &self.keys;
        let (first, last) = (keys.first()?, keys.last()?);
        if frame <= first.co[0] {
            return Some(self.before(first, frame));
        }
        if frame >= last.co[0] {
            return Some(self.after(last, frame));
        }
        // The segment [a, b] that holds `frame`.
        let i = keys.partition_point(|k| k.co[0] <= frame);
        let (a, b) = (&keys[i - 1], &keys[i]);
        if (b.co[0] - a.co[0]).abs() < 1e-9 {
            return Some(b.co[1]);
        }
        Some(match a.ipo {
            IPO_CONST => a.co[1],
            IPO_LIN => {
                let t = (frame - a.co[0]) / (b.co[0] - a.co[0]);
                a.co[1] + (b.co[1] - a.co[1]) * t
            }
            IPO_BEZ => bezier(a, b, frame),
            eased => super::easing::eased(
                eased,
                a.easing,
                frame - a.co[0],
                a.co[1],
                b.co[1] - a.co[1],
                b.co[0] - a.co[0],
                a.back,
                a.amplitude,
                a.period,
            ),
        })
    }

    /// `fcurve_eval_keyframes_extrapolate` before the first key: the value
    /// held, or — `FCURVE_EXTRAPOLATE_LINEAR` — the line through the first
    /// key on its own slope: its left handle's, or the next key's for a
    /// linear key (a constant key still holds).
    fn before(&self, first: &BKey, frame: f32) -> f32 {
        if self.extend != EXTRAPOLATE_LINEAR || first.ipo == IPO_CONST {
            return first.co[1];
        }
        let dx = first.co[0] - frame;
        let slope = if first.ipo == IPO_LIN {
            match self.keys.get(1) {
                Some(next) => slope_of(first.co, next.co),
                None => 0.0,
            }
        } else {
            slope_of(first.h1, first.co)
        };
        first.co[1] - slope * dx
    }

    /// The same past the last key, on its right handle or the previous key.
    fn after(&self, last: &BKey, frame: f32) -> f32 {
        if self.extend != EXTRAPOLATE_LINEAR || last.ipo == IPO_CONST {
            return last.co[1];
        }
        let dx = frame - last.co[0];
        let n = self.keys.len();
        let slope = if last.ipo == IPO_LIN {
            if n >= 2 { slope_of(self.keys[n - 2].co, last.co) } else { 0.0 }
        } else {
            slope_of(last.co, last.h2)
        };
        last.co[1] + slope * dx
    }
}

impl BKey {
    /// `BKE_fcurve_correct_bezpart`: the segment `a → b` with its handles
    /// pulled in so neither reaches past the other key in time — the
    /// segment stays a function of time. Returns `a`'s right handle and
    /// `b`'s left handle as corrected.
    pub fn corrected_handles(a: &BKey, b: &BKey) -> ([f32; 2], [f32; 2]) {
        let (v1, mut v2, mut v3, v4) = (a.co, a.h2, b.h1, b.co);
        let h1 = [v1[0] - v2[0], v1[1] - v2[1]];
        let h2 = [v4[0] - v3[0], v4[1] - v3[1]];
        let len = v4[0] - v1[0];
        let (len1, len2) = (h1[0].abs(), h2[0].abs());
        if len1 + len2 == 0.0 {
            return (v2, v3);
        }
        if len1 > len {
            let fac = len / len1;
            v2 = [v1[0] - fac * h1[0], v1[1] - fac * h1[1]];
        }
        if len2 > len {
            let fac = len / len2;
            v3 = [v4[0] - fac * h2[0], v4[1] - fac * h2[1]];
        }
        (v2, v3)
    }

    /// Cubic Bézier position at parameter `t` (`berekeny`).
    pub fn cubic(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
        let c1 = 3.0 * (p1 - p0);
        let c2 = 3.0 * (p0 - 2.0 * p1 + p2);
        let c3 = p3 - p0 + 3.0 * (p1 - p2);
        p0 + t * c1 + t * t * c2 + t * t * t * c3
    }
}

/// `dy / dx` from `from` to `to`; flat when they share a time.
fn slope_of(from: [f32; 2], to: [f32; 2]) -> f32 {
    let dx = to[0] - from[0];
    if dx.abs() < 1e-9 { 0.0 } else { (to[1] - from[1]) / dx }
}

/// The value of the segment `a → b` at time `frame`: solve the x cubic for
/// `t` (`findzero`), then read the y cubic there. After the handles are
/// corrected the x cubic is monotonic on [0, 1], so a bisection finds the
/// one root.
fn bezier(a: &BKey, b: &BKey, frame: f32) -> f32 {
    let (v2, v3) = BKey::corrected_handles(a, b);
    let (x0, x1, x2, x3) = (a.co[0], v2[0], v3[0], b.co[0]);
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if BKey::cubic(x0, x1, x2, x3, mid) < frame { lo = mid } else { hi = mid }
    }
    let t = 0.5 * (lo + hi);
    BKey::cubic(a.co[1], v2[1], v3[1], b.co[1], t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(x: f32, y: f32, ipo: u8) -> BKey {
        BKey::new([x - 1.0, y], [x, y], [x + 1.0, y], ipo)
    }

    #[test]
    fn holds_its_ends_and_interpolates_between() {
        let c = BFCurve::new("location", 0, vec![key(1.0, 0.0, 1), key(11.0, 10.0, 1)]);
        assert_eq!(c.eval(-5.0), Some(0.0));
        assert_eq!(c.eval(50.0), Some(10.0));
        assert!((c.eval(6.0).unwrap() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn a_flat_bezier_eases_in_and_out() {
        // Flat handles: the curve leaves 0 slowly and arrives at 10 slowly,
        // so it is below the line early and above it late.
        let c = BFCurve::new("location", 0, vec![key(0.0, 0.0, 2), key(10.0, 10.0, 2)]);
        let early = c.eval(2.0).unwrap();
        let late = c.eval(8.0).unwrap();
        assert!(early < 2.0 && early > 0.0, "{early}");
        assert!(late > 8.0 && late < 10.0, "{late}");
        assert!((c.eval(5.0).unwrap() - 5.0).abs() < 1e-4);
    }

    #[test]
    fn evaluation_is_zero_without_keys_and_the_generator_replaces_the_keys() {
        let empty = BFCurve::new("influence", 0, vec![]);
        assert_eq!(empty.evaluate(0.3), 0.0, "`evaluate_fcurve` on a keyless driver");
        let mut inverted = BFCurve::new("mute", 0, vec![key(0.0, 5.0, 1), key(1.0, 5.0, 1)]);
        inverted.generator = Some(vec![1.0, -1.0]);
        assert!((inverted.evaluate(0.25) - 0.75).abs() < 1e-6, "the Generator's `1 − x`, not the keys' 5");
        let keyed = BFCurve::new("location", 0, vec![key(1.0, 0.0, 1), key(11.0, 10.0, 1)]);
        assert!((keyed.evaluate(6.0) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn paths_split_into_bone_and_property() {
        let c = BFCurve::new("pose.bones[\"chest\"].rotation_quaternion", 1, vec![]);
        assert_eq!(c.bone(), Some("chest"));
        assert_eq!(c.property(), "rotation_quaternion");
        let o = BFCurve::new("location", 0, vec![]);
        assert_eq!(o.bone(), None);
        assert_eq!(o.property(), "location");
    }

    #[test]
    fn an_easing_key_eases_into_the_next_and_a_muted_curve_is_switched_off() {
        let mut a = key(0.0, 0.0, super::super::easing::IPO_QUAD);
        a.easing = super::super::easing::EASE_IN;
        let c = BFCurve::new("location", 0, vec![a, key(10.0, 10.0, 1)]);
        assert!((c.eval(5.0).unwrap() - 2.5).abs() < 1e-5, "quad in: a quarter of the way at half time");
        assert_eq!(c.eval(10.0), Some(10.0));
        let mut muted = c.clone();
        muted.flag |= FLAG_MUTED;
        assert!(muted.muted() && !c.muted());
    }

    #[test]
    fn linear_extrapolation_runs_on_along_the_end_slopes() {
        // A Bézier key's handles give the slope; a linear key takes the next key's.
        let mut c = BFCurve::new(
            "location",
            0,
            vec![BKey::new([-1.0, -2.0], [0.0, 0.0], [1.0, 2.0], 2), BKey::new([9.0, 10.0], [10.0, 10.0], [11.0, 10.0], 2)],
        );
        assert_eq!(c.eval(-5.0), Some(0.0), "constant extrapolation holds");
        c.extend = EXTRAPOLATE_LINEAR;
        assert!((c.eval(-5.0).unwrap() + 10.0).abs() < 1e-5, "slope 2 from the left handle: {:?}", c.eval(-5.0));
        assert!((c.eval(15.0).unwrap() - 10.0).abs() < 1e-5, "a flat right handle holds the value");
        let mut lin = BFCurve::new("location", 0, vec![key(0.0, 0.0, 1), key(10.0, 10.0, 1)]);
        lin.extend = EXTRAPOLATE_LINEAR;
        assert!((lin.eval(20.0).unwrap() - 20.0).abs() < 1e-5, "a linear key runs on through the previous key");
        assert!((lin.eval(-10.0).unwrap() + 10.0).abs() < 1e-5);
    }
}
