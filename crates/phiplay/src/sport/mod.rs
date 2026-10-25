//! A sport: its field and markings, its ball, its goals, how many play a
//! side and where, and the rules the play is refereed by.
//!
//! Each sport owns a folder — `sport/<name>/mod.rs` says what it is, its
//! `set.rs` holds the numbers, taken from its governing body's rulebook
//! (FIBA, World Netball, FIVB, IFAB) — and `sport/set.rs` is the index.

mod set;
pub mod basketball;
pub mod beach;
pub mod netball;
pub mod soccer;
pub mod volleyball;

use crate::colour::Rgb;
use crate::field::BFieldSpec;
use crate::kit::{BCut, BFootwear};
use crate::turf::BTurf;

/// The five sports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSport {
    Basketball,
    Netball,
    Volleyball,
    Beach,
    Soccer,
}

impl BSport {
    pub const ALL: [BSport; 5] = [BSport::Basketball, BSport::Netball, BSport::Volleyball, BSport::Beach, BSport::Soccer];
}

/// What a sport scores into.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BGoal {
    /// A basket on a backboard: rim `height` up, `out` metres in from the
    /// end line, rim radius `rim` (inside).
    Hoop { height: f32, out: f32, rim: f32 },
    /// A netball ring on a post standing on the goal line, no board.
    Ring { height: f32, rim: f32, reach: f32 },
    /// A net across the centre line, `height` at its top, posts `post`
    /// metres outside the sidelines.
    Net { height: f32, post: f32 },
    /// A goal on each goal line, `width` between the posts and `height` to
    /// the crossbar's underside.
    Goal { width: f32, height: f32, depth: f32 },
}

/// How a ball's cover is cut, for the ball's texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BBallCut {
    /// Eight panels: two crossing great circles and two curved seams.
    Basketball,
    /// Eighteen panels in six strips of three, as volleyballs and netballs
    /// are made.
    Strips,
    /// The truncated icosahedron: 12 pentagons, 20 hexagons.
    Soccer,
}

/// The ball.
#[derive(Clone, Copy, Debug)]
pub struct BBallSpec {
    pub radius: f32,
    pub mass: f32,
    /// How much of its speed a bounce on this sport's floor keeps.
    pub bounce: f32,
    pub cut: BBallCut,
    /// The cover's colours, panel by panel, and the seams'.
    pub panels: &'static [Rgb],
    pub seam: Rgb,
}

/// Where a position may stand, for the sports that say (netball).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BZone {
    /// The third a team defends, the centre third, the third it attacks.
    OwnThird,
    CentreThird,
    AttackThird,
    /// The goal circles.
    OwnCircle,
    AttackCircle,
}

/// One position on a side.
#[derive(Clone, Copy, Debug)]
pub struct BPositionSpec {
    pub code: &'static str,
    pub name: &'static str,
    /// Where it lines up, as a share of the half-field: x from −1 (the end
    /// it defends) to 1 (the end it attacks), y from −1 to 1 across.
    pub at: [f32; 2],
    /// Where it may go; empty is anywhere on the field.
    pub zones: &'static [BZone],
}

/// How a match is scored and won.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BScoring {
    /// Points per basket: from the field, from beyond the arc, a free throw.
    Baskets { field: u8, far: u8, free: u8 },
    /// One a goal.
    Goals,
    /// Rally scoring: a point every rally, a set won at `to` by two.
    Rally { to: u8, sets: u8 },
}

/// The rules the play is refereed by.
#[derive(Clone, Copy, Debug)]
pub struct BRulesSpec {
    pub scoring: BScoring,
    /// The ball may be bounced while moving (basketball, soccer's feet).
    pub dribble: bool,
    /// Steps allowed with the ball in hand; `None` for sports the ball is
    /// not held in.
    pub steps: Option<u8>,
    /// Seconds the ball may be held; netball's three.
    pub hold: Option<f32>,
    /// Touches a side may make before the ball crosses (volleyball's three).
    pub touches: Option<u8>,
    /// Periods and their length, minutes of game clock.
    pub periods: u8,
    pub minutes: f32,
}

/// Everything one sport is.
#[derive(Clone, Copy, Debug)]
pub struct BSportSpec {
    pub label: &'static str,
    pub icon: &'static str,
    /// The governing body whose rulebook the numbers come from.
    pub body: &'static str,
    pub field: BFieldSpec,
    pub ball: BBallSpec,
    pub goal: BGoal,
    /// Players on the field per side, and the roster a side brings.
    pub side: u8,
    pub roster: u8,
    pub positions: &'static [BPositionSpec],
    pub rules: BRulesSpec,
    /// What the field is made of unless the picker says otherwise.
    pub turf: BTurf,
    /// Whether the venue is roofed for it (an indoor court) or open.
    pub indoor: bool,
    /// How its uniform is cut, and what is worn on the feet.
    pub cut: BCut,
    pub footwear: BFootwear,
}

impl BSportSpec {
    /// Played over a net (volleyball, beach) rather than into a goal.
    pub fn goal_is_net(&self) -> bool {
        matches!(self.goal, BGoal::Net { .. })
    }
}

#[cfg(test)]
mod tests;
