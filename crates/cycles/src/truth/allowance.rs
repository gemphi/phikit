// SPDX-License-Identifier: Apache-2.0
//! Ported from `tests/python/modules/render_report.py:287-288` (the
//! default: a channel fails past 0.016, a render when more than 1 per cent
//! of its pixels do) and `tests/python/cycles_render_tests.py:337-357` (the
//! allowances Blender itself sets for some test categories). A category is
//! the folder a scene is in, `test_dir_name`.

/// `Report.fail_threshold` and `Report.fail_percent`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Allowance {
    /// `fail_threshold`: the most a channel may differ.
    pub threshold: f32,
    /// `fail_percent`: the most pixels, in per cent, that may differ more.
    pub percent: f64,
}

impl Default for Allowance {
    /// `render_report.py:287-288`.
    fn default() -> Self {
        Self { threshold: 0.016, percent: 1.0 }
    }
}

impl Allowance {
    /// What `cycles_render_tests.py` sets for `category`, in the order it
    /// sets it (a later line overrides an earlier one, so `light` ends at
    /// 0.02 and 4 per cent). `osl_all` is `args.osl == 'all'`, which this
    /// port does not run but whose allowances are kept (lines 338, 343).
    pub fn of_category(category: &str, osl_all: bool) -> Self {
        let mut allowance = Self::default();
        let is = |names: &[&str]| names.contains(&category);
        // Lines 337-339.
        if is(&["motion_blur", "integrator", "displacement"]) || (osl_all && is(&["shader", "hair"])) {
            allowance.threshold = 0.032;
        }
        // Lines 343-344.
        if osl_all && category == "principled_bsdf" {
            allowance.threshold = 0.06;
        }
        // Lines 347-348.
        if is(&["shadow_catcher", "light"]) {
            allowance.threshold = 0.038;
        }
        // Lines 349-351.
        if is(&["light", "camera"]) {
            allowance.threshold = 0.02;
            allowance.percent = 4.0;
        }
        // Lines 352-354.
        if is(&["volume", "openvdb"]) {
            allowance.threshold = 0.048;
            allowance.percent = 3.0;
        }
        // Lines 356-357.
        if category == "colorspace" {
            allowance.threshold = 0.05;
        }
        allowance
    }
}
