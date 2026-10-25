//! IFAB's markings: the penalty area 16.5 m out from the goal line and
//! 16.5 m either side of the posts (40.32 m across), the goal area 5.5 m
//! by 18.32 m, the penalty mark 11 m out, the centre circle and penalty
//! arcs 9.15 m. The arc outside the area: the mark is 5.5 m behind the
//! area's line, so the arc's ends sit at acos(5.5 / 9.15) = 53.05° either
//! side of the line toward the halfway line.

use crate::colour::Rgb;
use crate::field::{BFieldSpec, BMark};
use crate::sport::{BBallCut, BBallSpec, BGoal, BPositionSpec, BRulesSpec, BScoring, BSportSpec};
use crate::kit::{BCut, BFootwear};
use crate::turf::BTurf;

const X: f32 = 52.5;
const Y: f32 = 34.0;

static MARKS: &[BMark] = &[
    BMark::Frame { min: [-X, -Y], max: [X, Y] },
    BMark::Line { a: [0.0, -Y], b: [0.0, Y] },
    BMark::Arc { c: [0.0, 0.0], r: 9.15, from: 0.0, to: 360.0 },
    BMark::Spot { c: [0.0, 0.0], r: 0.11 },
    BMark::Frame { min: [X - 16.5, -20.16], max: [X, 20.16] },
    BMark::Frame { min: [-X, -20.16], max: [-X + 16.5, 20.16] },
    BMark::Frame { min: [X - 5.5, -9.16], max: [X, 9.16] },
    BMark::Frame { min: [-X, -9.16], max: [-X + 5.5, 9.16] },
    BMark::Spot { c: [X - 11.0, 0.0], r: 0.11 },
    BMark::Spot { c: [-X + 11.0, 0.0], r: 0.11 },
    BMark::Arc { c: [X - 11.0, 0.0], r: 9.15, from: 126.9482, to: 233.0518 },
    BMark::Arc { c: [-X + 11.0, 0.0], r: 9.15, from: -53.0518, to: 53.0518 },
    // The corner arcs, 1 m.
    BMark::Arc { c: [X, Y], r: 1.0, from: 180.0, to: 270.0 },
    BMark::Arc { c: [X, -Y], r: 1.0, from: 90.0, to: 180.0 },
    BMark::Arc { c: [-X, Y], r: 1.0, from: 270.0, to: 360.0 },
    BMark::Arc { c: [-X, -Y], r: 1.0, from: 0.0, to: 90.0 },
];

/// A 4-3-3 lined up for the kick-off, every player in her own half.
static POSITIONS: &[BPositionSpec] = &[
    BPositionSpec { code: "GK", name: "Goalkeeper", at: [-0.95, 0.0], zones: &[] },
    BPositionSpec { code: "RB", name: "Right back", at: [-0.6, -0.65], zones: &[] },
    BPositionSpec { code: "RCB", name: "Right centre back", at: [-0.7, -0.22], zones: &[] },
    BPositionSpec { code: "LCB", name: "Left centre back", at: [-0.7, 0.22], zones: &[] },
    BPositionSpec { code: "LB", name: "Left back", at: [-0.6, 0.65], zones: &[] },
    BPositionSpec { code: "RCM", name: "Right midfield", at: [-0.3, -0.35], zones: &[] },
    BPositionSpec { code: "CM", name: "Centre midfield", at: [-0.4, 0.0], zones: &[] },
    BPositionSpec { code: "LCM", name: "Left midfield", at: [-0.3, 0.35], zones: &[] },
    BPositionSpec { code: "RW", name: "Right wing", at: [-0.08, -0.6], zones: &[] },
    BPositionSpec { code: "ST", name: "Striker", at: [-0.02, 0.0], zones: &[] },
    BPositionSpec { code: "LW", name: "Left wing", at: [-0.08, 0.6], zones: &[] },
];

pub static SPEC: BSportSpec = BSportSpec {
    label: "Soccer",
    icon: "game_soccer",
    body: "IFAB",
    field: BFieldSpec { half: [X, Y], runoff: 6.0, line: 0.12, marks: MARKS, paint: &[] },
    // A size 5: 68–70 cm, 410–450 g; the classic cut of white hexagons and
    // black pentagons.
    ball: BBallSpec {
        radius: 0.1100,
        mass: 0.43,
        bounce: 0.6,
        cut: BBallCut::Soccer,
        panels: &[Rgb::hex(0xF7F7F2), Rgb::hex(0x1B1B1F)],
        seam: Rgb::hex(0x9AA0A6),
    },
    goal: BGoal::Goal { width: 7.32, height: 2.44, depth: 2.0 },
    side: 11,
    roster: 23,
    positions: POSITIONS,
    rules: BRulesSpec {
        scoring: BScoring::Goals,
        dribble: true,
        steps: None,
        hold: None,
        touches: None,
        periods: 2,
        minutes: 45.0,
    },
    turf: BTurf::Stripes,
    indoor: false,
    cut: BCut::Jersey,
    footwear: BFootwear::Boots,
};
