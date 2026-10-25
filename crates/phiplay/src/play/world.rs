//! The match as a phiecs world: a word for each player ("Ember 7"), the
//! ball and each side's goal; aspects on the players — where they are and
//! where they are going (`BBody`), who they are (`BRole`), what they are
//! doing (`BAct`); and the match itself as a resource (`BGame`) — the
//! phase, the score, the ball, the tape being written.
//!
//! The systems shared by every sport live here: steering the players toward
//! their targets (Post-couple) and writing the frame down (Readout).

use super::ball::BBallState;
use super::state::BPhase;
use super::{BEvent, BStep, BTape, BTrack, FPS, heading_to, unwrap};
use crate::choice::BChoice;
use crate::look::roster::roster;
use crate::pose::BMoment;
use crate::pose::moves::BMove;
use crate::sport::BSportSpec;
use glam::Vec2;
use phiecs::prelude::*;

/// Where a player is and is heading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBody {
    pub at: Vec2,
    pub vel: Vec2,
    pub heading: f32,
    /// Where she is going, and how fast she may go there.
    pub target: Vec2,
    pub pace: f32,
    /// The heading she wants when standing (toward the ball, the basket).
    pub face: Option<f32>,
}
impl Aspect for BBody {}

/// Who a player is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRole {
    pub team: usize,
    pub slot: usize,
    pub roster: usize,
}
impl Aspect for BRole {}

/// What a player is doing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAct {
    pub mv: BMove,
    /// The frame it began.
    pub start: usize,
    /// Its stride's phase and the dribble's, running on.
    pub phase: f32,
    pub dphase: f32,
    /// The move before and its moment, for the blend into this one.
    pub was: Option<(BMove, BMoment)>,
}
impl Aspect for BAct {}

/// The relation kinds the play reads.
#[derive(Clone, Copy, Debug)]
pub struct BKinds {
    /// Attacker ⇒ teammate (or goal): how good a pass (or shot) is.
    pub open_to: Relation,
    /// Defender ⇒ the attacker she marks.
    pub guards: Relation,
}

/// The match: a resource of the world.
pub struct BGame {
    pub sport: &'static BSportSpec,
    pub choice: BChoice,
    pub frame: usize,
    pub phase: BPhase,
    pub score: [u16; 2],
    /// The side with the ball (invasion) or the serve (rally).
    pub attack: usize,
    pub ball: BBallState,
    pub players: Vec<Word>,
    pub goals: [Word; 2],
    pub kinds: BKinds,
    pub tape: BTape,
    /// Frames until the restart after a score or a dead ball.
    pub wait: usize,
}

/// Blends between moves over this many frames (make_demo's `BLEND`).
pub const BLEND: usize = 5;

/// A world lined up for `choice`: `side` players a team, in their positions.
pub fn build(choice: &BChoice) -> World {
    let sport = choice.sport.spec();
    let mut w = World::new();
    w.register::<BBody>();
    w.register::<BRole>();
    w.register::<BAct>();
    let kinds = BKinds { open_to: w.declare("open_to").expect("a kind"), guards: w.declare("guards").expect("a kind") };
    let mut players = Vec::new();
    let mut tracks = Vec::new();
    for team in 0..2 {
        let team_kind = if team == 0 { choice.home } else { choice.away };
        let people = roster(team_kind);
        for slot in 0..usize::from(sport.side) {
            let at = home_spot(sport, team, slot);
            let word = w.coin(&format!("{} {}", team_kind.spec().name, people[slot].number)).word();
            w.bind(word, BBody { at, vel: Vec2::ZERO, heading: heading_to(Vec2::new(-attack_dir(team), 0.0)) + std::f32::consts::PI, target: at, pace: 3.0, face: None });
            w.bind(word, BRole { team, slot, roster: slot });
            w.bind(word, BAct { mv: BMove::Idle, start: 0, phase: 0.0, dphase: 0.0, was: None });
            players.push(word);
            tracks.push(BTrack { team, roster: slot, slot, steps: Vec::new() });
        }
    }
    let goals = [w.coin_bare("goal west"), w.coin_bare("goal east")];
    let tape = BTape { players: tracks, ball: Vec::new(), flying: Vec::new(), score: Vec::new(), events: Vec::new() };
    w.resources.insert(BGame { sport, choice: *choice, frame: 0, phase: BPhase::Lineup, score: [0, 0], attack: 0, ball: BBallState::Dead { at: glam::Vec3::new(0.0, 0.0, sport.ball.radius) }, players, goals, kinds, tape, wait: 0 });
    w
}

/// The way a team attacks along x: home toward +x, away toward −x.
pub fn attack_dir(team: usize) -> f32 {
    if team == 0 { 1.0 } else { -1.0 }
}

/// Where a team's slot lines up: the sport's position, turned to face the
/// end the team attacks.
pub fn home_spot(sport: &BSportSpec, team: usize, slot: usize) -> Vec2 {
    let p = sport.positions[slot].at;
    let d = attack_dir(team);
    Vec2::new(p[0] * sport.field.half[0] * d, p[1] * sport.field.half[1] * d)
}

/// The game resource, read.
pub fn game(w: &World) -> &BGame {
    w.resources.get::<BGame>().expect("the game")
}

pub fn game_mut(w: &mut World) -> &mut BGame {
    w.resources.get_mut::<BGame>().expect("the game")
}

/// Starts `mv` for `player` now (if her current move allows it).
pub fn act(w: &mut World, player: Word, mv: BMove) -> bool {
    let frame = game(w).frame;
    let Some(a) = w.read::<BAct>(player).copied() else { return false };
    if a.mv == mv || !a.mv.may_follow(mv) {
        return a.mv == mv;
    }
    let was = Some((a.mv, moment_of(w, player, frame)));
    w.bind(player, BAct { mv, start: frame, was, ..a });
    true
}

/// How far through her action a player is (0 at its start, 1 at its end);
/// a way of moving is always at 0.
pub fn progress(w: &World, player: Word) -> f32 {
    let frame = game(w).frame;
    let a = w.read::<BAct>(player).expect("acting");
    if a.mv.is_action() { (frame - a.start) as f32 / a.mv.frames().max(2) as f32 } else { 0.0 }
}

/// Whether her action has run its course.
pub fn done(w: &World, player: Word) -> bool {
    let a = w.read::<BAct>(player).expect("acting");
    a.mv.is_action() && game(w).frame >= a.start + a.mv.frames()
}

fn moment_of(w: &World, player: Word, frame: usize) -> BMoment {
    let a = w.read::<BAct>(player).expect("acting");
    let b = w.read::<BBody>(player).expect("a body");
    let speed = b.vel.length();
    let t = if a.mv.is_action() { ((frame - a.start) as f32 / a.mv.frames().max(2) as f32).min(1.0) } else { 0.0 };
    let ball = game(w).ball.at().truncate();
    let to_ball = ball - b.at;
    let look = if to_ball.length() > 0.3 { (unwrap(heading_to(to_ball), b.heading) - b.heading).clamp(-0.9, 0.9) } else { 0.0 };
    BMoment { t, amp: (speed / 5.5).min(1.0).powf(0.7), phase: a.phase, bounce: a.dphase.sin().abs(), bank: 0.0, look }
}

/// Post-couple: every player steered toward her target — accelerating at
/// most 6 m/s², never past her pace — her heading turned toward where she
/// runs (or what she watches standing), her stride's phase advanced by the
/// ground covered so the feet do not skate.
pub fn steer(w: &mut World, dt: f32) {
    let players = game(w).players.clone();
    let ball = game(w).ball.at().truncate();
    for p in players {
        let mut b = *w.read::<BBody>(p).expect("a body");
        let acting = w.read::<BAct>(p).expect("acting").mv;
        let planted = matches!(acting, BMove::Pivot | BMove::JumpShot | BMove::Shoot | BMove::Set | BMove::Dig | BMove::Block | BMove::Contest | BMove::Serve | BMove::Celebrate);
        let to = b.target - b.at;
        let want = if planted || to.length() < 0.15 { Vec2::ZERO } else { to.normalize() * b.pace.min(to.length() * 2.5) };
        let dv = (want - b.vel).clamp_length_max(6.0 * dt);
        b.vel += dv;
        b.at += b.vel * dt;
        let speed = b.vel.length();
        let wanted = if speed > 1.5 { heading_to(b.vel) } else { b.face.unwrap_or_else(|| heading_to(ball - b.at)) };
        let wanted = unwrap(wanted, b.heading);
        b.heading += ((wanted - b.heading) * 0.3).clamp(-0.3, 0.3);
        w.bind(p, b);
        let mut a = *w.read::<BAct>(p).expect("acting");
        let stride = (0.9 + 0.3 * speed).clamp(1.1, 2.6);
        a.phase += std::f32::consts::TAU * speed * dt / stride;
        a.dphase += std::f32::consts::PI * (2.0 * speed / stride).max(1.8) * dt;
        w.bind(p, a);
    }
}

/// Readout: the frame written onto the tape.
pub fn record(w: &mut World, _dt: f32) {
    let frame = game(w).frame;
    let players = game(w).players.clone();
    let steps: Vec<BStep> = players
        .iter()
        .map(|&p| {
            let a = *w.read::<BAct>(p).expect("acting");
            let b = *w.read::<BBody>(p).expect("a body");
            let moment = moment_of(w, p, frame);
            let since = frame - a.start;
            let from = a.was.filter(|_| since < BLEND).map(|(mv, m)| (mv, BMoment { t: 1.0, ..m }, 1.0 - (since + 1) as f32 / (BLEND + 1) as f32));
            BStep { at: b.at, heading: b.heading, mv: a.mv, moment, from }
        })
        .collect();
    let g = game_mut(w);
    for (track, step) in g.tape.players.iter_mut().zip(steps) {
        track.steps.push(step);
    }
    g.tape.ball.push(g.ball.at());
    g.tape.flying.push(g.ball.flying());
    g.tape.score.push(g.score);
    g.frame += 1;
}

/// Notes a score for `team` on the tape.
pub fn scored(w: &mut World, team: usize, points: u16, what: &str) {
    let g = game_mut(w);
    g.score[team] += points;
    let frame = g.frame;
    g.tape.events.push(BEvent { frame, team, what: what.to_string() });
    g.phase.transition(BPhase::Scored);
    g.wait = (FPS * 1.5) as usize;
}
