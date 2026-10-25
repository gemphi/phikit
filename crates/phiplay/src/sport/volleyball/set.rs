//! The FIVB's numbers, and the six rotation places.
//!
//! Each side keeps its own half (x < 0 for the side the positions are
//! written for), facing the net at x = 0: place 4 is front left, 3 front
//! middle, 2 front right, 1 back right (the server), 6 back middle, 5 back
//! left. "Left" from a player facing +x is +y.

use crate::colour::Rgb;
use crate::field::{BFieldSpec, BMark, BPaint};
use crate::sport::{BBallCut, BBallSpec, BGoal, BPositionSpec, BRulesSpec, BScoring, BSportSpec};
use crate::theme::BRole;
use crate::kit::{BCut, BFootwear};
use crate::turf::BTurf;

static MARKS: &[BMark] = &[
    BMark::Frame { min: [-9.0, -4.5], max: [9.0, 4.5] },
    BMark::Line { a: [0.0, -4.5], b: [0.0, 4.5] },
    // The attack lines, 3 m either side of the centre line.
    BMark::Line { a: [3.0, -4.5], b: [3.0, 4.5] },
    BMark::Line { a: [-3.0, -4.5], b: [-3.0, 4.5] },
];

static PAINT: &[BPaint] = &[
    BPaint { min: [-9.0, -4.5], max: [9.0, 4.5], role: BRole::Primary },
    BPaint { min: [-3.0, -4.5], max: [3.0, 4.5], role: BRole::Secondary },
];

static POSITIONS: &[BPositionSpec] = &[
    BPositionSpec { code: "1", name: "Right back, the server", at: [-0.8, -0.6], zones: &[] },
    BPositionSpec { code: "2", name: "Right front", at: [-0.25, -0.6], zones: &[] },
    BPositionSpec { code: "3", name: "Middle front", at: [-0.25, 0.0], zones: &[] },
    BPositionSpec { code: "4", name: "Left front", at: [-0.25, 0.6], zones: &[] },
    BPositionSpec { code: "5", name: "Left back", at: [-0.8, 0.6], zones: &[] },
    BPositionSpec { code: "6", name: "Middle back", at: [-0.8, 0.0], zones: &[] },
];

pub static SPEC: BSportSpec = BSportSpec {
    label: "Volleyball",
    icon: "game_volleyball",
    body: "FIVB",
    // The free zone is 5 m past the sidelines and 6.5 m past the end lines
    // in FIVB competition; 5.5 all round keeps the floor one rectangle.
    field: BFieldSpec { half: [9.0, 4.5], runoff: 5.5, line: 0.05, marks: MARKS, paint: PAINT },
    // 65–67 cm round, 260–280 g, in the yellow and blue of the FIVB ball.
    ball: BBallSpec {
        radius: 0.1050,
        mass: 0.27,
        bounce: 0.7,
        cut: BBallCut::Strips,
        panels: &[Rgb::hex(0xF2C230), Rgb::hex(0x1F4FA8), Rgb::hex(0xF5F5F0)],
        seam: Rgb::hex(0x2A2A30),
    },
    goal: BGoal::Net { height: 2.24, post: 1.0 },
    side: 6,
    roster: 14,
    positions: POSITIONS,
    rules: BRulesSpec {
        scoring: BScoring::Rally { to: 25, sets: 5 },
        dribble: false,
        steps: None,
        hold: None,
        touches: Some(3),
        periods: 5,
        minutes: 0.0,
    },
    turf: BTurf::Polymer,
    indoor: true,
    cut: BCut::Fitted,
    footwear: BFootwear::Shoes,
};
