//! Rally games over a net — volleyball and beach volleyball. Each side keeps
//! its half (the home side x < 0); a rally is a run of contacts, each a
//! flight from one player's hands to the next's: the serve from behind the
//! end line, then at most three touches a side — a dig to the setter, a set
//! to the hitter, a spike over the net — until a contact fails and the ball
//! falls. Rally scoring: every rally is a point, the winner serves, and a
//! side that wins the serve back rotates a place clockwise (`rotation`).
//!
//! Each contact's move is started so that its moment of contact (`release`)
//! lands on the frame the ball arrives.

use super::ball::{self, BAim, BBallState};
use super::state::BPhase;
use super::world::{self, BAct, BBody, BRole, act, attack_dir, done, game, game_mut, scored};
use super::{BTape, FPS, dice};
use crate::choice::BChoice;
use crate::pose::moves::{BMove, release};
use crate::sport::BGoal;
use glam::{Vec2, Vec3};
use phiecs::prelude::*;

/// The rally's state, beside the game's.
#[derive(Clone, Copy, Debug, Default)]
struct BRally {
    /// Touches the side with the ball has made.
    touches: u8,
    /// The side the ball is on (or is going to).
    side: usize,
    /// Each side's rotation, in places.
    rotation: [usize; 2],
    /// The next contact: who, with what move.
    next: Option<(Word, BMove)>,
    /// The side that served this rally.
    server: usize,
    /// How the ball last came over the net: a serve or an attack.
    over: Option<BMove>,
}

pub fn play(choice: &BChoice, frames: usize) -> BTape {
    let mut w = world::build(choice);
    w.resources.insert(BRally::default());
    w.add_system(FnSystem::new("ball", ball::advance).at(Stage::Ingest));
    w.add_system(FnSystem::new("decide", decide).at(Stage::Couple));
    w.add_system(FnSystem::new("steer", world::steer).at(Stage::PostCouple));
    w.add_system(FnSystem::new("record", world::record).at(Stage::Readout));
    for _ in 0..frames {
        w.step(1.0 / FPS);
    }
    w.resources.remove::<world::BGame>().expect("the game").tape
}

fn rally(w: &World) -> BRally {
    *w.resources.get::<BRally>().expect("the rally")
}

fn set_rally(w: &mut World, r: BRally) {
    w.resources.insert(r);
}

fn body(w: &World, p: Word) -> BBody {
    *w.read::<BBody>(p).expect("a body")
}

fn role(w: &World, p: Word) -> BRole {
    *w.read::<BRole>(p).expect("a role")
}

/// Where a player stands for her side's rotation: her slot moved round
/// the places, `rotation` steps clockwise.
fn spot(w: &World, p: Word) -> Vec2 {
    let g = game(w);
    let r = role(w, p);
    let n = usize::from(g.sport.side);
    let place = (r.slot + n - rally(w).rotation[r.team] % n) % n;
    world::home_spot(g.sport, r.team, place)
}

/// The player of `team` nearest to `to`, but not `not`.
fn nearest(w: &World, team: usize, to: Vec2, not: Option<Word>) -> Word {
    let all = game(w).players.clone();
    all.into_iter()
        .filter(|&p| role(w, p).team == team && Some(p) != not)
        .min_by(|&a, &b| body(w, a).at.distance(to).total_cmp(&body(w, b).at.distance(to)))
        .expect("a side")
}

fn decide(w: &mut World, _dt: f32) {
    let frame = game(w).frame;
    match game(w).phase {
        BPhase::Lineup => {
            game_mut(w).phase.transition(BPhase::Restart);
            serve_up(w);
        }
        BPhase::Restart => {
            if game(w).wait > 0 {
                game_mut(w).wait -= 1;
            } else if let BBallState::Held { by, .. } = game(w).ball {
                act(w, by, BMove::Serve);
                game_mut(w).phase.transition(BPhase::Live);
            }
        }
        BPhase::Scored => {
            if game(w).wait > 0 {
                game_mut(w).wait -= 1;
            } else {
                game_mut(w).phase.transition(BPhase::Restart);
                serve_up(w);
            }
        }
        BPhase::Live => live(w, frame),
        BPhase::Dead | BPhase::Over => {}
    }
    position(w);
}

/// The serving side's server behind her end line with the ball.
fn serve_up(w: &mut World) {
    let serving = game(w).attack;
    let mut r = rally(w);
    r.touches = 0;
    r.server = serving;
    r.next = None;
    set_rally(w, r);
    let sport = game(w).sport;
    let n = usize::from(sport.side);
    let server = game(w).players.clone().into_iter().find(|&p| role(w, p).team == serving && (role(w, p).slot + n - r.rotation[serving] % n) % n == 0).expect("a server");
    for p in game(w).players.clone() {
        let mut b = body(w, p);
        b.at = spot(w, p);
        b.target = b.at;
        b.vel = Vec2::ZERO;
        w.bind(p, b);
        let a = *w.read::<BAct>(p).expect("acting");
        w.bind(p, BAct { mv: BMove::Idle, start: game(w).frame, ..a });
    }
    let mut b = body(w, server);
    b.at = Vec2::new(-attack_dir(serving) * (sport.field.half[0] + 1.0), b.at.y);
    b.target = b.at;
    b.face = Some(super::heading_to(Vec2::new(attack_dir(serving), 0.0)));
    w.bind(server, b);
    let at = ball::in_hands(w, server);
    let g = game_mut(w);
    g.ball = BBallState::Held { by: server, at };
    g.wait = (FPS * 1.2) as usize;
}

/// Picks who takes the ball next and where, and sends it there.
fn send(w: &mut World, from_side: usize, touch: u8, frame: usize) {
    let sport = game(w).sport;
    let [hx, hy] = sport.field.half;
    let net = match sport.goal {
        BGoal::Net { height, .. } => height,
        _ => 2.24,
    };
    // Over the net when it is the third touch (or a serve); else to a
    // teammate on this side.
    let (side, to, mv, frames, height) = if touch == 0 || touch >= 3 {
        let other = 1 - from_side;
        let d = -attack_dir(other);
        let land = Vec2::new(d * hx * (0.25 + 0.6 * dice(frame, 1)), hy * (dice(frame, 2) * 1.6 - 0.8));
        let frames = if touch == 0 { 26 } else { 13 };
        (other, land, BMove::Dig, frames, 0.9)
    } else if touch == 1 {
        (from_side, Vec2::new(-attack_dir(from_side) * 1.0, 0.0), BMove::Set, 22, net + 0.2)
    } else {
        let y = if dice(frame, 3) < 0.5 { 1.0 } else { -1.0 } * (hy - 1.2);
        (from_side, Vec2::new(-attack_dir(from_side) * 1.2, y), BMove::Spike, 20, net + 0.65)
    };
    let holder = game(w).ball.holder();
    let who = nearest(w, side, to, if side == from_side { holder } else { None });
    let mut b = body(w, who);
    b.target = to;
    w.bind(who, b);
    let mut r = rally(w);
    if side != from_side {
        r.over = Some(if touch == 0 { BMove::Serve } else { BMove::Spike });
    }
    r.side = side;
    r.touches = if side == from_side { touch } else { 0 };
    r.next = Some((who, mv));
    set_rally(w, r);
    // A spike meets the other side's block at the net.
    if mv == BMove::Dig && touch >= 3 {
        let blockers: Vec<Word> = game(w).players.clone().into_iter().filter(|&p| role(w, p).team == side && body(w, p).at.x.abs() < 3.5).take(2).collect();
        for blocker in blockers {
            act(w, blocker, BMove::Block);
        }
    }
    ball::throw(w, Vec3::new(to.x, to.y, height), frames, BAim::To(who));
}

fn live(w: &mut World, frame: usize) {
    // The serve's release.
    if let BBallState::Held { by, .. } = game(w).ball {
        if progress_past_release(w, by) {
            let side = role(w, by).team;
            game_mut(w).ball = BBallState::Held { by, at: ball::in_hands(w, by) };
            send(w, side, 0, frame);
        }
        return;
    }
    let r = rally(w);
    match game(w).ball {
        BBallState::Flight { end, .. } => {
            // The contact's move starts so its contact lands on arrival.
            if let Some((who, mv)) = r.next {
                let lead = (release(mv).unwrap_or(0.5) * mv.frames() as f32) as usize;
                if end.saturating_sub(frame) == lead {
                    act(w, who, mv);
                }
            }
        }
        BBallState::Arrived { aim: BAim::To(who), at } => {
            let touch = r.touches + 1;
            // Does the contact hold? The first touch over the net is the hard
            // one: a serve is received nine times in ten, a spike dug about
            // half the time (an attack's kill rate in the women's game).
            let odds = match (touch, r.next.map(|n| n.1)) {
                (1, _) if r.over == Some(BMove::Serve) => 0.9,
                (1, _) => 0.5,
                (_, Some(BMove::Set)) => 0.97,
                (_, Some(BMove::Spike)) => 0.93,
                _ => 0.9,
            };
            if dice(frame, 31) < odds {
                game_mut(w).ball = BBallState::Held { by: who, at };
                send(w, role(w, who).team, touch, frame);
            } else {
                // It falls: the point to the other side.
                let lost = role(w, who).team;
                game_mut(w).ball = BBallState::Loose { vel: Vec3::new(0.0, 0.0, -1.0), at };
                point(w, 1 - lost);
            }
        }
        _ => {}
    }
}

fn progress_past_release(w: &World, by: Word) -> bool {
    let a = w.read::<BAct>(by).expect("acting");
    a.mv == BMove::Serve && world::progress(w, by) >= release(BMove::Serve).unwrap_or(0.55)
}

/// A rally won by `team`: the point, the serve, the rotation on a side-out,
/// the winners' celebration.
fn point(w: &mut World, team: usize) {
    scored(w, team, 1, "point");
    let mut r = rally(w);
    if r.server != team {
        r.rotation[team] += 1;
    }
    set_rally(w, r);
    game_mut(w).attack = team;
    let winners: Vec<Word> = game(w).players.clone().into_iter().filter(|&p| role(w, p).team == team).collect();
    for p in winners {
        act(w, p, BMove::Idle);
        act(w, p, BMove::Celebrate);
    }
}

/// Everyone not on the ball back to her place, facing the net.
fn position(w: &mut World) {
    let next = rally(w).next.map(|n| n.0);
    for p in game(w).players.clone() {
        let a = *w.read::<BAct>(p).expect("acting");
        let mut b = body(w, p);
        if Some(p) != next && game(w).phase == BPhase::Live {
            b.target = spot(w, p);
        }
        b.pace = 4.5;
        b.face = Some(super::heading_to(Vec2::new(attack_dir(role(w, p).team), 0.0)));
        w.bind(p, b);
        if !a.mv.is_action() || done(w, p) {
            let mv = if b.target.distance(b.at) > 0.5 || b.vel.length() > 0.6 { BMove::Run } else { BMove::Idle };
            if !act(w, p, mv) {
                act(w, p, BMove::Idle);
            }
        }
    }
}
