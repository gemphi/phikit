//! World Netball's numbers and the seven positions' areas.

use crate::colour::Rgb;
use crate::field::{BFieldSpec, BMark, BPaint};
use crate::sport::{BBallCut, BBallSpec, BGoal, BPositionSpec, BRulesSpec, BScoring, BSportSpec, BZone};
use crate::theme::BRole;
use crate::kit::{BCut, BFootwear};
use crate::turf::BTurf;

/// The transverse lines, a third of 30.5 m apart.
const THIRD: f32 = 30.5 / 6.0;

static MARKS: &[BMark] = &[
    BMark::Frame { min: [-15.25, -7.625], max: [15.25, 7.625] },
    BMark::Line { a: [THIRD, -7.625], b: [THIRD, 7.625] },
    BMark::Line { a: [-THIRD, -7.625], b: [-THIRD, 7.625] },
    // The centre circle, 0.9 m across.
    BMark::Arc { c: [0.0, 0.0], r: 0.45, from: 0.0, to: 360.0 },
    // The goal circles: semicircles of 4.9 m about each goal line's middle.
    BMark::Arc { c: [15.25, 0.0], r: 4.9, from: 90.0, to: 270.0 },
    BMark::Arc { c: [-15.25, 0.0], r: 4.9, from: -90.0, to: 90.0 },
];

static PAINT: &[BPaint] = &[
    BPaint { min: [-15.25, -7.625], max: [-THIRD, 7.625], role: BRole::Primary },
    BPaint { min: [THIRD, -7.625], max: [15.25, 7.625], role: BRole::Primary },
];

use BZone::*;
static POSITIONS: &[BPositionSpec] = &[
    BPositionSpec { code: "GS", name: "Goal shooter", at: [0.8, 0.0], zones: &[AttackThird, AttackCircle] },
    BPositionSpec { code: "GA", name: "Goal attack", at: [0.5, 0.3], zones: &[CentreThird, AttackThird, AttackCircle] },
    BPositionSpec { code: "WA", name: "Wing attack", at: [0.3, -0.5], zones: &[CentreThird, AttackThird] },
    BPositionSpec { code: "C", name: "Centre", at: [0.0, 0.0], zones: &[OwnThird, CentreThird, AttackThird] },
    BPositionSpec { code: "WD", name: "Wing defence", at: [-0.3, 0.5], zones: &[OwnThird, CentreThird] },
    BPositionSpec { code: "GD", name: "Goal defence", at: [-0.5, -0.3], zones: &[OwnThird, CentreThird, OwnCircle] },
    BPositionSpec { code: "GK", name: "Goal keeper", at: [-0.8, 0.0], zones: &[OwnThird, OwnCircle] },
];

pub static SPEC: BSportSpec = BSportSpec {
    label: "Netball",
    icon: "game_netball",
    body: "World Netball",
    field: BFieldSpec { half: [15.25, 7.625], runoff: 3.05, line: 0.05, marks: MARKS, paint: PAINT },
    // A size 5: 69–71 cm round, 400–450 g, made in panels like a volleyball.
    ball: BBallSpec {
        radius: 0.1114,
        mass: 0.425,
        bounce: 0.75,
        cut: BBallCut::Strips,
        panels: &[Rgb::hex(0xF4F1E8), Rgb::hex(0xF2C94C), Rgb::hex(0x2F80ED)],
        seam: Rgb::hex(0x3A3A40),
    },
    // The ring's inside is 380 mm across, 150 mm out from the post.
    goal: BGoal::Ring { height: 3.05, rim: 0.19, reach: 0.15 },
    side: 7,
    roster: 12,
    positions: POSITIONS,
    rules: BRulesSpec {
        scoring: BScoring::Goals,
        dribble: false,
        steps: Some(1),
        hold: Some(3.0),
        touches: None,
        periods: 4,
        minutes: 15.0,
    },
    turf: BTurf::Polymer,
    indoor: true,
    cut: BCut::Vest,
    footwear: BFootwear::Shoes,
};
