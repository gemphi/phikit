//! The FIVB's beach numbers.

use crate::colour::Rgb;
use crate::field::{BFieldSpec, BMark};
use crate::sport::{BBallCut, BBallSpec, BGoal, BPositionSpec, BRulesSpec, BScoring, BSportSpec};
use crate::kit::{BCut, BFootwear};
use crate::turf::BTurf;

/// Only the boundary: a beach court has no centre or attack lines, its
/// boundary a band 5–8 cm wide.
static MARKS: &[BMark] = &[BMark::Frame { min: [-8.0, -4.0], max: [8.0, 4.0] }];

static POSITIONS: &[BPositionSpec] = &[
    BPositionSpec { code: "B", name: "Blocker", at: [-0.45, 0.45], zones: &[] },
    BPositionSpec { code: "D", name: "Defender", at: [-0.7, -0.45], zones: &[] },
];

pub static SPEC: BSportSpec = BSportSpec {
    label: "Beach volleyball",
    icon: "game_beach",
    body: "FIVB",
    field: BFieldSpec { half: [8.0, 4.0], runoff: 5.0, line: 0.07, marks: MARKS, paint: &[] },
    // 66–68 cm, 260–280 g; sand takes most of a bounce.
    ball: BBallSpec {
        radius: 0.1066,
        mass: 0.27,
        bounce: 0.2,
        cut: BBallCut::Strips,
        panels: &[Rgb::hex(0xFFD400), Rgb::hex(0x0057B8), Rgb::hex(0xF7F7F2)],
        seam: Rgb::hex(0x2A2A30),
    },
    goal: BGoal::Net { height: 2.24, post: 0.8 },
    side: 2,
    roster: 2,
    positions: POSITIONS,
    rules: BRulesSpec {
        scoring: BScoring::Rally { to: 21, sets: 3 },
        dribble: false,
        steps: None,
        hold: None,
        touches: Some(3),
        periods: 3,
        minutes: 0.0,
    },
    turf: BTurf::Sand,
    indoor: false,
    cut: BCut::Crop,
    footwear: BFootwear::Barefoot,
};
