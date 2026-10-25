//! The match's phases and the moves a player may make next, each with the
//! states it may lead to — BOT-anist's `AnimationState` (`validNextStates`,
//! `transition(to:)`), so a match cannot jump from a tip-off to a
//! celebration without a score between.

use crate::pose::moves::BMove;

/// Where a match is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPhase {
    /// Lined up, nothing started.
    Lineup,
    /// The restart: a tip-off, a centre pass, a serve, a kick-off.
    Restart,
    /// The ball in play.
    Live,
    /// A score: the scorers celebrate, the ball comes back.
    Scored,
    /// Out of play: a dead ball before the restart.
    Dead,
    /// The end.
    Over,
}

impl BPhase {
    pub fn valid_next(self) -> &'static [BPhase] {
        use BPhase::*;
        match self {
            Lineup => &[Restart],
            Restart => &[Live],
            Live => &[Scored, Dead, Over],
            Scored => &[Restart, Over],
            Dead => &[Restart, Over],
            Over => &[],
        }
    }

    /// Moves to `next` if the phase allows it; `false` and unchanged if not.
    pub fn transition(&mut self, next: BPhase) -> bool {
        if *self != next && self.valid_next().contains(&next) {
            *self = next;
            true
        } else {
            false
        }
    }
}

impl BMove {
    /// Whether the move is an action that runs its course (a pass, a shot)
    /// rather than a way of moving that lasts as long as it is wanted.
    pub fn is_action(self) -> bool {
        !matches!(self, BMove::Idle | BMove::Run | BMove::Dribble | BMove::Defend | BMove::Pivot | BMove::Clap | BMove::Cheer)
    }

    /// How long the action runs, in frames at 24 a second.
    pub fn frames(self) -> usize {
        match self {
            BMove::Pass => 13,
            BMove::Catch => 12,
            BMove::JumpShot => 31,
            BMove::Layup => 33,
            BMove::Contest | BMove::Block => 26,
            BMove::PickUp => 14,
            BMove::Celebrate => 36,
            BMove::Shoot => 30,
            BMove::Serve => 30,
            BMove::Dig | BMove::Set => 16,
            BMove::Spike => 30,
            BMove::Kick => 16,
            BMove::Header => 18,
            BMove::Save => 22,
            _ => 1,
        }
    }

    /// The moves that may follow this one.
    pub fn valid_next(self) -> &'static [BMove] {
        use BMove::*;
        match self {
            Idle | Run | Defend => &[Idle, Run, Defend, Catch, PickUp, Contest, Block, Celebrate, Dig, Set, Spike, Kick, Header, Save, Serve, Pass, Dribble, Pivot, JumpShot, Layup, Shoot],
            Dribble => &[Dribble, Pass, JumpShot, Layup, Run, Idle, Kick],
            Pivot => &[Pivot, Pass, Shoot],
            Catch | PickUp | Save => &[Dribble, Pivot, Pass, JumpShot, Shoot, Run, Idle, Kick, Layup],
            Pass | Kick | Header | Serve | Dig | Set | Spike | Block | Contest | JumpShot | Layup | Shoot => &[Idle, Run, Defend, Celebrate, Catch, Dig, Set, Block],
            Celebrate => &[Idle, Run, Defend],
            Clap | Cheer => &[Clap, Cheer],
        }
    }

    pub fn may_follow(self, next: BMove) -> bool {
        self == next || self.valid_next().contains(&next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_match_goes_only_where_its_phase_allows() {
        let mut phase = BPhase::Lineup;
        assert!(!phase.transition(BPhase::Scored), "no score before the restart");
        assert!(phase.transition(BPhase::Restart) && phase.transition(BPhase::Live) && phase.transition(BPhase::Scored));
        assert!(phase.transition(BPhase::Restart));
        assert!(!BMove::Celebrate.may_follow(BMove::JumpShot), "no shot from a celebration");
        assert!(BMove::Catch.may_follow(BMove::JumpShot), "a catch and shoot");
    }
}
