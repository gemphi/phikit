//! FIBA's numbers. The three-point line is an arc of 6.75 m about the
//! basket, run into straight lines 0.9 m in from the sidelines; they meet
//! 2.99 m in from the end line (`sqrt(6.75² − 6.6²)` short of the basket),
//! which is where the arc's ends sit, at ±102.1° about the basket.

use crate::colour::{INK, Rgb};
use crate::field::{BFieldSpec, BMark, BPaint};
use crate::sport::{BBallCut, BBallSpec, BGoal, BPositionSpec, BRulesSpec, BScoring, BSportSpec};
use crate::theme::BRole;
use crate::kit::{BCut, BFootwear};
use crate::turf::BTurf;

/// The basket's centre, in from the end line.
const BASKET: f32 = 14.0 - 1.575;
/// Where the three-point arc meets its straight lines.
const ARC_END: f32 = 11.0099;
const FT: f32 = 14.0 - 5.8;

static MARKS: &[BMark] = &[
    BMark::Frame { min: [-14.0, -7.5], max: [14.0, 7.5] },
    BMark::Line { a: [0.0, -7.5], b: [0.0, 7.5] },
    BMark::Arc { c: [0.0, 0.0], r: 1.8, from: 0.0, to: 360.0 },
    // The keys (the restricted areas), 5.8 m long and 4.9 m wide, and the
    // free-throw circles on their lines.
    BMark::Frame { min: [FT, -2.45], max: [14.0, 2.45] },
    BMark::Frame { min: [-14.0, -2.45], max: [-FT, 2.45] },
    BMark::Arc { c: [FT, 0.0], r: 1.8, from: 0.0, to: 360.0 },
    BMark::Arc { c: [-FT, 0.0], r: 1.8, from: 0.0, to: 360.0 },
    // The three-point lines.
    BMark::Arc { c: [BASKET, 0.0], r: 6.75, from: 102.1015, to: 257.8985 },
    BMark::Line { a: [14.0, 6.6], b: [ARC_END, 6.6] },
    BMark::Line { a: [14.0, -6.6], b: [ARC_END, -6.6] },
    BMark::Arc { c: [-BASKET, 0.0], r: 6.75, from: -77.8985, to: 77.8985 },
    BMark::Line { a: [-14.0, 6.6], b: [-ARC_END, 6.6] },
    BMark::Line { a: [-14.0, -6.6], b: [-ARC_END, -6.6] },
    // The no-charge semicircles under the baskets.
    BMark::Arc { c: [BASKET, 0.0], r: 1.25, from: 90.0, to: 270.0 },
    BMark::Arc { c: [-BASKET, 0.0], r: 1.25, from: -90.0, to: 90.0 },
];

static PAINT: &[BPaint] = &[
    BPaint { min: [FT, -2.45], max: [14.0, 2.45], role: BRole::Primary },
    BPaint { min: [-14.0, -2.45], max: [-FT, 2.45], role: BRole::Secondary },
];

static POSITIONS: &[BPositionSpec] = &[
    BPositionSpec { code: "PG", name: "Point guard", at: [0.25, 0.0], zones: &[] },
    BPositionSpec { code: "SG", name: "Shooting guard", at: [0.45, 0.65], zones: &[] },
    BPositionSpec { code: "SF", name: "Small forward", at: [0.55, -0.7], zones: &[] },
    BPositionSpec { code: "PF", name: "Power forward", at: [0.75, 0.35], zones: &[] },
    BPositionSpec { code: "C", name: "Centre", at: [0.85, -0.15], zones: &[] },
];

pub static SPEC: BSportSpec = BSportSpec {
    label: "Basketball",
    icon: "game_basketball",
    body: "FIBA",
    field: BFieldSpec { half: [14.0, 7.5], runoff: 2.5, line: 0.05, marks: MARKS, paint: PAINT },
    // A size 6, the women's ball: 72 cm round, about 530 g, and lively —
    // dropped from 1.8 m it comes back to 1.3 (√(1.3/1.8) ≈ 0.85).
    ball: BBallSpec { radius: 0.1146, mass: 0.53, bounce: 0.85, cut: BBallCut::Basketball, panels: &[Rgb::hex(0xD8662B)], seam: INK },
    goal: BGoal::Hoop { height: 3.05, out: 1.575, rim: 0.225 },
    side: 5,
    roster: 12,
    positions: POSITIONS,
    rules: BRulesSpec {
        scoring: BScoring::Baskets { field: 2, far: 3, free: 1 },
        dribble: true,
        steps: Some(2),
        hold: None,
        touches: None,
        periods: 4,
        minutes: 10.0,
    },
    turf: BTurf::Maple,
    indoor: true,
    cut: BCut::Vest,
    footwear: BFootwear::Shoes,
};
