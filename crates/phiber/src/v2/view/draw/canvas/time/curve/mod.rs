//! The F-curves the Graph editor draws — `graph_draw_curves` over the
//! channels `ANIM_animdata_filter` lists, each in the colour
//! `graph_refresh_fcurve_colors` gives it and at the alpha its selection
//! earns. Structure: the curve list and the value axis; painting is `set.rs`.

pub mod handle;
pub mod line;
pub mod set;

use crate::phiphx::blend::{BFCurve, COLOR_AUTO_RGB, COLOR_AUTO_YRGB, COLOR_CUSTOM, FLAG_ACTIVE, FLAG_MUTED};
use crate::v2::state::{BState, CtxKey};
use crate::v2::theme::BAnimColors;

/// `U.fcu_inactive_alpha`: an unselected curve is drawn at a quarter.
pub const INACTIVE_ALPHA: f32 = 0.25;
/// `HSV_BANDWIDTH` of `getcolor_fcurve_rainbow`.
const HSV_BANDWIDTH: f32 = 0.3;

/// The theme's axis colours (`tui.xaxis` … `waxis`, and the one past them)
/// for `AUTO_RGB` / `AUTO_YRGB`, as the curve painter takes them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAxisColors {
    pub x: [f32; 3],
    pub y: [f32; 3],
    pub z: [f32; 3],
    pub w: [f32; 3],
    pub other: [f32; 3],
}

impl BAxisColors {
    pub fn of(colors: &BAnimColors) -> Self {
        Self {
            x: BAnimColors::rgb(colors.axis_x),
            y: BAnimColors::rgb(colors.axis_y),
            z: BAnimColors::rgb(colors.axis_z),
            w: BAnimColors::rgb(colors.axis_w),
            other: BAnimColors::rgb(colors.axis_other),
        }
    }
}

/// One curve as drawn: its keys, colour and weight.
#[derive(Clone, Debug, PartialEq)]
pub struct BDrawnCurve {
    pub curve: BFCurve,
    pub color: [f32; 3],
    pub alpha: f32,
    /// `FCURVE_ACTIVE`: drawn 2.5 px wide where the rest are 1 px.
    pub active: bool,
    /// `FCURVE_MUTED`: drawn in a grey, whatever its colour.
    pub muted: bool,
}

/// The Graph editor's curves and the value range they span.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BCurveSet {
    pub list: Vec<BDrawnCurve>,
    /// The lowest and highest key value, or none without keys.
    pub range: Option<(f32, f32)>,
}

impl BCurveSet {
    /// The curves of every shown object's actions (Only Show Selected, plus
    /// the active object), coloured as Blender colours them.
    pub fn now(s: &BState, axis: BAxisColors) -> Self {
        let active = s.get(CtxKey::Object).map(str::to_string);
        crate::v2::scene::source::with_loaded(|loaded| {
            let Some(model) = loaded else { return Self::default() };
            let mut shown: Vec<&str> = model.selection.selected.iter().map(String::as_str).collect();
            shown.extend(active.as_deref());
            let curves: Vec<BFCurve> = model
                .actions
                .iter()
                .filter(|action| shown.iter().any(|object| action.used_by(object)))
                .flat_map(|action| action.curves.iter().cloned())
                .collect();
            Self::of(curves, axis)
        })
    }

    pub fn of(curves: Vec<BFCurve>, axis: BAxisColors) -> Self {
        let total = curves.len();
        let mut range: Option<(f32, f32)> = None;
        for value in curves.iter().flat_map(|c| c.keys.iter().map(|k| k.co[1])) {
            range = Some(range.map_or((value, value), |(lo, hi)| (lo.min(value), hi.max(value))));
        }
        let list = curves
            .into_iter()
            .enumerate()
            .map(|(index, curve)| BDrawnCurve {
                color: Self::color(&curve, index, total, axis),
                alpha: if curve.selected() { 1.0 } else { INACTIVE_ALPHA },
                active: curve.flag & FLAG_ACTIVE != 0,
                muted: curve.flag & FLAG_MUTED != 0,
                curve,
            })
            .collect();
        Self { list, range }
    }

    /// `graph_refresh_fcurve_colors`: a custom colour is kept, `AUTO_RGB`
    /// maps the array index to the X/Y/Z axis colours, `AUTO_YRGB` puts W
    /// first, and everything else takes the rainbow.
    pub fn color(curve: &BFCurve, index: usize, total: usize, axis: BAxisColors) -> [f32; 3] {
        match curve.color_mode {
            COLOR_CUSTOM => curve.color,
            COLOR_AUTO_RGB => match curve.index {
                0 => axis.x,
                1 => axis.y,
                2 => axis.z,
                _ => axis.other,
            },
            COLOR_AUTO_YRGB => match curve.index {
                0 => axis.w,
                1 => axis.x,
                2 => axis.y,
                3 => axis.z,
                _ => axis.other,
            },
            _ => Self::rainbow(index, total.max(1)),
        }
    }

    /// `getcolor_fcurve_rainbow`: triplets or quartets of hues a bandwidth
    /// apart, drifting with the curve's position in the list.
    pub fn rainbow(current: usize, total: usize) -> [f32; 3] {
        let grouping = 4 - (total % 2);
        let mut hue = HSV_BANDWIDTH * (current % grouping) as f32;
        let fac = (current as f32 / total as f32) * 0.7;
        hue += fac * HSV_BANDWIDTH;
        if hue > 1.0 {
            hue %= 1.0;
        }
        let sat = if hue > 0.5 && hue < 0.8 { 0.5 } else { 0.6 };
        hsv_to_rgb(hue, sat, 1.0)
    }

    /// `graph_view_all`: the value range with a tenth of margin either side,
    /// and a unit either side of a flat line.
    pub fn framed(&self) -> (f32, f32) {
        let (lo, hi) = self.range.unwrap_or((0.0, 1.0));
        let span = hi - lo;
        if span < 1e-6 {
            return (lo - 1.0, hi + 1.0);
        }
        (lo - span * 0.1, hi + span * 0.1)
    }
}

/// `hsv_to_rgb_v`.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    match (i as i32).rem_euclid(6) {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::{BKey, COLOR_AUTO_RAINBOW, FLAG_SELECTED};

    fn key(frame: f32, value: f32) -> BKey {
        BKey::new([frame - 1.0, value], [frame, value], [frame + 1.0, value], 1)
    }

    const AXIS: BAxisColors = BAxisColors { x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], z: [0.0, 0.0, 1.0], w: [1.0, 1.0, 0.0], other: [0.0, 1.0, 1.0] };

    #[test]
    fn colours_follow_blenders_modes_and_the_range_frames_the_keys() {
        let mut rgb = BFCurve::new("location", 2, vec![key(1.0, -2.0), key(10.0, 6.0)]);
        rgb.color_mode = COLOR_AUTO_RGB;
        rgb.flag = FLAG_SELECTED;
        let mut yrgb = BFCurve::new("rotation_quaternion", 0, vec![key(5.0, 1.0)]);
        yrgb.color_mode = COLOR_AUTO_YRGB;
        let mut custom = BFCurve::new("scale", 0, vec![]);
        custom.color_mode = COLOR_CUSTOM;
        custom.color = [0.1, 0.2, 0.3];
        let mut rainbow = BFCurve::new("value", 0, vec![]);
        rainbow.color_mode = COLOR_AUTO_RAINBOW;
        let set = BCurveSet::of(vec![rgb, yrgb, custom, rainbow], AXIS);
        assert_eq!(set.list[0].color, AXIS.z);
        assert_eq!(set.list[0].alpha, 1.0, "a selected curve is drawn solid");
        assert_eq!(set.list[1].color, AXIS.w);
        assert_eq!(set.list[1].alpha, INACTIVE_ALPHA);
        assert_eq!(set.list[2].color, [0.1, 0.2, 0.3]);
        assert_eq!(set.list[3].color, BCurveSet::rainbow(3, 4));
        assert_eq!(set.range, Some((-2.0, 6.0)));
        assert_eq!(set.framed(), (-2.8, 6.8));
        assert_eq!(BCurveSet::default().framed(), (-0.1, 1.1), "no keys: a unit range, framed");
    }

    #[test]
    fn the_rainbow_is_blenders_magic_function() {
        // Four curves: quartets, so the first hue is 0 and the second 0.3.
        let first = BCurveSet::rainbow(0, 4);
        assert!(first.iter().zip([1.0, 0.4, 0.4]).all(|(got, want)| (got - want).abs() < 1e-6), "hue 0, saturation 0.6, value 1: {first:?}");
        // The second drifts to hue 0.3525 — a green — at saturation 0.6.
        let second = BCurveSet::rainbow(1, 4);
        assert!((second[0] - 0.4).abs() < 1e-6 && (second[1] - 1.0).abs() < 1e-6 && (second[2] - 0.469).abs() < 1e-3, "{second:?}");
    }
}
