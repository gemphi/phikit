//! Invasion games — basketball, netball, soccer: a side carries the ball
//! toward the other's goal, the other marks and waits for it back.
//!
//! Perception (Pre-couple) writes who is open as relations in the world:
//! the holder ⇒ each teammate, weighted by how free and how far forward
//! she is; each attacker ⇒ the goal, weighted by her shot. The holder's
//! choice (Couple) is the world's own plan — the heaviest chain of
//! `open_to` from her to the goal (`World::plan`): straight there is a
//! shot, through a teammate is a pass. The rules shape it: netball's three
//! seconds and zones, basketball's dribble, soccer's feet.

use super::ball::{self, BAim, BBallState, in_hands};
use super::state::BPhase;
use super::world::{self, BAct, BBody, BRole, act, attack_dir, done, game, game_mut, home_spot, progress, scored};
use super::{BTape, FPS, dice};
use crate::choice::BChoice;
use crate::pose::moves::{BMove, release, take};
use crate::sport::{BGoal, BScoring, BSportSpec, BZone};
use glam::{Vec2, Vec3};
use phiecs::prelude::*;

/// A throw decided and waiting for its release frame.
#[derive(Clone, Copy, Debug)]
struct BPending {
    by: Word,
    to: Vec3,
    frames: usize,
    aim: BAim,
}

/// The holder's clock: when she got the ball, when she last chose.
#[derive(Clone, Copy, Debug, Default)]
struct BClock {
    since: usize,
    chose: usize,
}

pub fn play(choice: &BChoice, frames: usize) -> BTape {
    let mut w = world::build(choice);
    w.resources.insert(Option::<BPending>::None);
    w.resources.insert(BClock::default());
    w.add_system(FnSystem::new("ball", ball::advance).at(Stage::Ingest));
    w.add_system(FnSystem::new("perceive", perceive).at(Stage::PreCouple));
    w.add_system(FnSystem::new("decide", decide).at(Stage::Couple));
    w.add_system(FnSystem::new("steer", world::steer).at(Stage::PostCouple));
    w.add_system(FnSystem::new("record", world::record).at(Stage::Readout));
    for _ in 0..frames {
        w.step(1.0 / FPS);
    }
    w.resources.remove::<world::BGame>().expect("the game").tape
}

/// Where team `team`'s goal is aimed at: the rim, the ring, the goal mouth.
fn target_of(sport: &BSportSpec, team: usize) -> Vec3 {
    let d = attack_dir(team);
    let hx = sport.field.half[0];
    match sport.goal {
        BGoal::Hoop { height, out, .. } => Vec3::new(d * (hx - out), 0.0, height),
        BGoal::Ring { height, rim, reach } => Vec3::new(d * (hx - reach - rim), 0.0, height),
        BGoal::Goal { height, .. } => Vec3::new(d * hx, 0.0, height * 0.5),
        BGoal::Net { .. } => Vec3::ZERO,
    }
}

fn players(w: &World) -> Vec<Word> {
    game(w).players.clone()
}

fn body(w: &World, p: Word) -> BBody {
    *w.read::<BBody>(p).expect("a body")
}

fn role(w: &World, p: Word) -> BRole {
    *w.read::<BRole>(p).expect("a role")
}

/// How good a shot `p` has from where she stands, 0–1.
fn shot(w: &World, p: Word) -> f32 {
    let g = game(w);
    let r = role(w, p);
    let at = body(w, p).at;
    let goal = target_of(g.sport, r.team).truncate();
    let d = at.distance(goal);
    // A defender within 2 m takes up to a third off.
    let free = (nearest_foe(w, p) / 2.0).min(1.0) * 0.3 + 0.7;
    // Open-play rates: about 45% from mid-range and 36% from beyond the arc
    // (the NBA's), 80–90% for a netball shooter in the circle, a striker's
    // chance falling from about two in five at six metres.
    let q = match g.sport.goal {
        BGoal::Hoop { .. } => (0.66 - 0.03 * d).max(0.0),
        BGoal::Ring { .. } => {
            let shooter = g.sport.positions[r.slot].zones.contains(&BZone::AttackCircle);
            if shooter && d <= 4.9 { 0.95 - 0.05 * d } else { 0.0 }
        }
        BGoal::Goal { .. } => if d < 20.0 { 0.55 - 0.022 * d } else { 0.0 },
        BGoal::Net { .. } => 0.0,
    };
    q * free
}

fn nearest_foe(w: &World, p: Word) -> f32 {
    let (team, at) = (role(w, p).team, body(w, p).at);
    players(w).iter().filter(|&&o| role(w, o).team != team).map(|&o| body(w, o).at.distance(at)).fold(f32::MAX, f32::min)
}

/// Pre-couple: who marks whom and who is open, as relations.
fn perceive(w: &mut World, _dt: f32) {
    let kinds = game(w).kinds;
    let all = players(w);
    // The relations are this frame's: drop last frame's.
    let ends: Vec<Word> = all.iter().copied().chain(game(w).goals).collect();
    for &a in &all {
        for &b in &ends {
            w.unrelate(a, kinds.open_to, b);
            w.unrelate(a, kinds.guards, b);
        }
    }
    let attack = game(w).attack;
    let side = usize::from(game(w).sport.side);
    let attackers: Vec<Word> = all.iter().copied().filter(|&p| role(w, p).team == attack).collect();
    let defenders: Vec<Word> = all.iter().copied().filter(|&p| role(w, p).team != attack).collect();
    for &a in &attackers {
        let goal = game(w).goals[usize::from(attack_dir(attack) > 0.0)];
        let q = shot(w, a);
        if q > 0.05 {
            w.relate_by(a, kinds.open_to, goal, f64::from(q), 0.0);
        }
        for &b in attackers.iter().filter(|&&p| p != a) {
            let free = (nearest_foe(w, b) / 4.0).min(1.0);
            let ahead = (body(w, b).at.x - body(w, a).at.x) * attack_dir(attack);
            let weight = free * (0.6 + (ahead / 10.0).clamp(-0.3, 0.4));
            if weight > 0.15 {
                w.relate_by(a, kinds.open_to, b, f64::from(weight), 0.0);
            }
        }
    }
    // Marking: slot for slot (netball's are mirrored: the keeper marks the
    // shooter); a soccer side marks zonally (`decide`).
    for &d in &defenders {
        let slot = role(w, d).slot;
        let mark = if matches!(game(w).sport.goal, BGoal::Ring { .. }) { side - 1 - slot } else { slot };
        if let Some(&a) = attackers.iter().find(|&&p| role(w, p).slot == mark) {
            w.relate(d, kinds.guards, a);
        }
    }
}

/// Couple: the phase moved on, the holder's choice, everyone's target.
fn decide(w: &mut World, _dt: f32) {
    let frame = game(w).frame;
    match game(w).phase {
        BPhase::Lineup => {
            game_mut(w).phase.transition(BPhase::Restart);
            restart(w, None);
        }
        BPhase::Restart => {
            if game(w).wait > 0 {
                game_mut(w).wait -= 1;
            } else {
                game_mut(w).phase.transition(BPhase::Live);
            }
        }
        BPhase::Scored | BPhase::Dead => {
            if game(w).wait > 0 {
                game_mut(w).wait -= 1;
            } else {
                // Out of play: taken in where it went out; a score: from the
                // middle (the end line in basketball).
                let out = matches!(game(w).phase, BPhase::Dead).then(|| game(w).ball.at().truncate());
                let g = game_mut(w);
                g.attack = 1 - g.attack;
                g.phase.transition(BPhase::Restart);
                restart(w, out);
            }
        }
        BPhase::Live => live(w, frame),
        BPhase::Over => {}
    }
    position(w);
    locomotion(w);
}

/// The restart: everyone to their spots, the ball to the side restarting —
/// from the middle, or from the line where it went `out`.
fn restart(w: &mut World, out: Option<Vec2>) {
    let sport = game(w).sport;
    let attack = game(w).attack;
    for p in players(w) {
        let r = role(w, p);
        let mut b = body(w, p);
        b.at = home_spot(sport, r.team, r.slot) * if r.team == attack { 0.6 } else { 0.8 };
        b.target = b.at;
        b.vel = Vec2::ZERO;
        w.bind(p, b);
        let a = *w.read::<BAct>(p).expect("acting");
        w.bind(p, BAct { mv: BMove::Idle, start: game(w).frame, was: Some((a.mv, Default::default())), ..a });
    }
    // The restarter: the point guard, the centre, the striker.
    let code = match sport.goal {
        BGoal::Hoop { .. } => "PG",
        BGoal::Ring { .. } => "C",
        _ => "ST",
    };
    let [hx, hy] = sport.field.half;
    let starter = match out {
        // From the line: whoever of the side is nearest it.
        Some(at) => players(w).into_iter().filter(|&p| role(w, p).team == attack).min_by(|&a, &b| body(w, a).at.distance(at).total_cmp(&body(w, b).at.distance(at))),
        None => players(w).into_iter().find(|&p| role(w, p).team == attack && sport.positions[role(w, p).slot].code == code),
    }
    .unwrap_or(players(w)[0]);
    let mut b = body(w, starter);
    b.at = match (out, sport.goal) {
        (Some(at), _) => Vec2::new(at.x.clamp(-hx - 0.3, hx + 0.3), at.y.clamp(-hy - 0.3, hy + 0.3)),
        (None, BGoal::Hoop { .. }) => Vec2::new(-attack_dir(attack) * hx * 0.5, 0.0),
        (None, _) => Vec2::new(-attack_dir(attack) * 0.3, 0.0),
    };
    b.target = b.at;
    w.bind(starter, b);
    let carried = if sport.rules.dribble { BBallState::Dribbled { by: starter, at: Vec3::ZERO } } else { BBallState::Held { by: starter, at: in_hands(w, starter) } };
    let g = game_mut(w);
    g.ball = carried;
    g.wait = (FPS * 0.6) as usize;
    let frame = g.frame;
    w.resources.insert(BClock { since: frame, chose: frame });
}

/// The ball in play.
fn live(w: &mut World, frame: usize) {
    let sport = game(w).sport;
    // A throw waiting for its release.
    if let Some(p) = *w.resources.get::<Option<BPending>>().expect("pending") {
        let mv = w.read::<BAct>(p.by).expect("acting").mv;
        if release(mv).is_none_or(|r| progress(w, p.by) >= r) {
            ball::throw(w, p.to, p.frames, p.aim);
            w.resources.insert(Option::<BPending>::None);
        }
    }
    match game(w).ball {
        BBallState::Held { by, .. } | BBallState::Dribbled { by, .. } => hold(w, by, frame),
        BBallState::Flight { aim: BAim::To(p), end, .. } => {
            let lead = (take(BMove::Catch).unwrap_or(0.36) * BMove::Catch.frames() as f32) as usize;
            if end.saturating_sub(frame) <= lead && sport.rules.steps.is_some() {
                act(w, p, BMove::Catch);
            }
        }
        BBallState::Arrived { aim, at } => match aim {
            BAim::To(p) => {
                game_mut(w).attack = role(w, p).team;
                game_mut(w).ball = if sport.rules.dribble && sport.rules.steps.is_none() { BBallState::Dribbled { by: p, at } } else { BBallState::Held { by: p, at } };
                w.resources.insert(BClock { since: frame, chose: frame });
            }
            BAim::Shot { by, team, made: true, points } => {
                scored(w, team, points, if points == 3 { "three" } else { "score" });
                game_mut(w).ball = BBallState::Loose { vel: Vec3::new(0.0, 0.0, -1.0), at };
                act(w, by, BMove::Idle);
                act(w, by, BMove::Celebrate);
            }
            BAim::Shot { team, .. } => {
                // Off the rim (or wide): loose, out toward the court.
                let out = Vec3::new(-attack_dir(team) * (1.5 + 2.0 * dice(frame, 3)), (dice(frame, 5) - 0.5) * 4.0, 2.5);
                game_mut(w).ball = BBallState::Loose { vel: out, at };
            }
            BAim::Court { .. } => {}
        },
        BBallState::Loose { at, .. } => {
            // Over a line: dead, to the other side.
            let [hx, hy] = sport.field.half;
            if at.x.abs() > hx + sport.ball.radius || at.y.abs() > hy + sport.ball.radius {
                let g = game_mut(w);
                g.ball = BBallState::Dead { at };
                g.phase.transition(BPhase::Dead);
                g.wait = FPS as usize;
                return;
            }
            // The nearest chases it (`position`); within reach and low, she
            // picks it up.
            if let Some(p) = chaser(w) {
                let b = body(w, p);
                if b.at.distance(at.truncate()) < 0.7 && at.z < 1.3 {
                    act(w, p, if sport.rules.steps.is_none() { BMove::Run } else { BMove::PickUp });
                    game_mut(w).attack = role(w, p).team;
                    game_mut(w).ball = if sport.rules.dribble && sport.rules.steps.is_none() { BBallState::Dribbled { by: p, at } } else { BBallState::Held { by: p, at } };
                    w.resources.insert(BClock { since: frame, chose: frame });
                }
            }
        }
        _ => {}
    }
}

/// The holder: back to carrying after an action, and every few frames a
/// choice — the plan's — or, out of time, any pass.
fn hold(w: &mut World, by: Word, frame: usize) {
    let sport = game(w).sport;
    if w.resources.get::<Option<BPending>>().expect("pending").is_some() {
        return;
    }
    let mv = w.read::<BAct>(by).expect("acting").mv;
    if mv.is_action() && !done(w, by) {
        return;
    }
    let clock = *w.resources.get::<BClock>().expect("clock");
    if frame < clock.chose + 6 {
        return;
    }
    w.resources.insert(BClock { chose: frame, ..clock });
    let held = (frame - clock.since) as f32 / FPS;
    let attack = role(w, by).team;
    let goal_word = game(w).goals[usize::from(attack_dir(attack) > 0.0)];
    let goal = target_of(sport, attack);
    let at = body(w, by).at;
    let kinds = game(w).kinds;
    let plan = w.plan(by, kinds.open_to, goal_word, 3);
    let urgent = sport.rules.hold.is_some_and(|h| held > h - 0.6);
    let roll = dice(frame, by.id() as usize);
    // Close in: lay it up.
    if matches!(sport.goal, BGoal::Hoop { .. }) && at.distance(goal.truncate()) < 1.8 {
        return shoot(w, by, BMove::Layup, frame);
    }
    match plan.as_deref() {
        Some([_, g]) if *g == goal_word && (roll < 0.45 || urgent) => {
            let mv = match sport.goal {
                BGoal::Hoop { .. } => BMove::JumpShot,
                BGoal::Ring { .. } => BMove::Shoot,
                _ => BMove::Kick,
            };
            shoot(w, by, mv, frame);
        }
        Some([_, mate, ..]) if *mate != goal_word && (roll < 0.35 || urgent || held > 2.5) => pass(w, by, *mate),
        _ if urgent => {
            let mate = w.calls_for(by, kinds.open_to).into_iter().map(|(m, _)| m).find(|m| *m != goal_word);
            if let Some(mate) = mate {
                pass(w, by, mate);
            }
        }
        _ => {}
    }
}

fn shoot(w: &mut World, by: Word, mv: BMove, frame: usize) {
    let sport = game(w).sport;
    let team = role(w, by).team;
    let q = if mv == BMove::Layup { 0.75 } else { shot(w, by) };
    let made = dice(frame, 17 + by.id() as usize) < q;
    let goal = target_of(sport, team);
    let d = body(w, by).at.distance(goal.truncate());
    let points = match sport.rules.scoring {
        BScoring::Baskets { field, far, .. } => if d > 6.75 { u16::from(far) } else { u16::from(field) },
        _ => 1,
    };
    let to = match sport.goal {
        BGoal::Goal { width, height, .. } if made => goal + Vec3::new(0.0, (dice(frame, 9) - 0.5) * width * 0.8, (dice(frame, 11) - 0.5) * height * 0.6),
        BGoal::Goal { width, .. } => goal + Vec3::new(0.0, width * 0.6 * (dice(frame, 9) - 0.5).signum(), 1.0),
        _ => goal + Vec3::Z * 0.05,
    };
    let frames = if mv == BMove::Layup { 9 } else { (8.0 + 1.6 * d).round() as usize };
    act(w, by, mv);
    w.resources.insert(Some(BPending { by, to, frames, aim: BAim::Shot { by, team, made, points } }));
    // Her marker contests.
    let kinds = game(w).kinds;
    if let Some(&guard) = players(w).iter().find(|&&d| w.calls_for(d, kinds.guards).iter().any(|(a, _)| *a == by)) {
        if body(w, guard).at.distance(body(w, by).at) < 2.0 && sport.rules.steps.is_some() {
            act(w, guard, BMove::Contest);
        }
    }
}

fn pass(w: &mut World, by: Word, to: Word) {
    let sport = game(w).sport;
    let target = body(w, to);
    let d = body(w, by).at.distance(target.at);
    let (mv, point, speed) = if sport.rules.steps.is_none() {
        let forward = Vec2::new(target.heading.sin(), -target.heading.cos());
        let p = target.at + forward * 0.45;
        (BMove::Kick, Vec3::new(p.x, p.y, sport.ball.radius), 15.0)
    } else {
        let p = target.at + Vec2::new(target.heading.sin(), -target.heading.cos()) * 0.32;
        (BMove::Pass, Vec3::new(p.x, p.y, 1.12), 9.0)
    };
    let frames = ((d / speed) * FPS).round().max(6.0) as usize;
    act(w, by, mv);
    w.resources.insert(Some(BPending { by, to: point, frames, aim: BAim::To(to) }));
}

/// Whoever is nearest a loose ball: she goes for it.
fn chaser(w: &World) -> Option<Word> {
    let BBallState::Loose { at, .. } = game(w).ball else { return None };
    players(w).into_iter().min_by(|&a, &b| body(w, a).at.distance(at.truncate()).total_cmp(&body(w, b).at.distance(at.truncate())))
}

/// Everyone's target: a pass's receiver to where it lands, a loose ball's
/// chaser to it, the attackers to their spots with cuts, the defenders
/// goal-side of whoever they mark, each held to her zones.
fn position(w: &mut World) {
    let g = game(w);
    let (sport, attack, frame, kinds) = (g.sport, g.attack, g.frame, g.kinds);
    let ball = g.ball.at().truncate();
    let holder = g.ball.holder();
    let receiver = match g.ball {
        BBallState::Flight { aim: BAim::To(p), to, .. } => Some((p, to.truncate())),
        _ => None,
    };
    let chasing = chaser(w);
    for p in players(w) {
        let r = role(w, p);
        let mut b = body(w, p);
        b.face = None;
        let home = home_spot(sport, r.team, r.slot);
        if let Some((_, to)) = receiver.filter(|(q, _)| *q == p) {
            b.target = to;
            b.pace = 6.0;
        } else if Some(p) == chasing {
            b.target = ball;
            b.pace = 6.5;
        } else if Some(p) == holder {
            // The holder drives toward the goal unless she may not move.
            let goal = target_of(sport, r.team).truncate();
            b.target = if sport.rules.steps == Some(1) { b.at } else { b.at + (goal - b.at).normalize_or_zero() * 2.0 };
            b.pace = if sport.rules.steps.is_none() { 5.0 } else { 3.5 };
        } else if r.team == attack {
            let cut = (frame / 60 + r.slot) % 3;
            let swing = Vec2::new(0.0, (dice(frame / 60, r.slot) - 0.5) * 4.0) * f32::from(u8::from(cut == 0));
            let shift = if sport.rules.steps.is_none() { Vec2::new(ball.x * 0.5, ball.y * 0.25) } else { Vec2::ZERO };
            b.target = home + swing + shift;
            b.pace = 4.5;
        } else {
            let marked = w.calls_for(p, kinds.guards).first().map(|(a, _)| *a);
            let own = target_of(sport, 1 - r.team).truncate();
            b.target = match marked {
                Some(a) => {
                    let at = body(w, a).at;
                    let gap = if Some(a) == holder { 0.9 } else if sport.rules.steps.is_none() { 2.5 } else { 1.4 };
                    at + (own - at).normalize_or_zero() * gap
                }
                None => home + Vec2::new(ball.x * 0.4, ball.y * 0.3),
            };
            b.pace = 5.0;
        }
        b.target = keep_in(sport, r, b.target);
        w.bind(p, b);
    }
}

/// `p` held to the field and to the zones her position allows.
fn keep_in(sport: &BSportSpec, r: BRole, p: Vec2) -> Vec2 {
    let [hx, hy] = sport.field.half;
    let mut q = Vec2::new(p.x.clamp(-hx + 0.3, hx - 0.3), p.y.clamp(-hy + 0.3, hy - 0.3));
    let zones = sport.positions[r.slot].zones;
    if zones.is_empty() {
        return q;
    }
    // In her own frame: x from her defended end (−hx) to the end she attacks.
    let d = attack_dir(r.team);
    let third = hx / 3.0;
    let x = q.x * d;
    let lo = if zones.contains(&BZone::OwnThird) { -hx } else if zones.contains(&BZone::CentreThird) { -third } else { third };
    let hi = if zones.contains(&BZone::AttackThird) { hx } else if zones.contains(&BZone::CentreThird) { third } else { -third };
    let mut x = x.clamp(lo + 0.2, hi - 0.2);
    // Out of a goal circle she may not enter.
    for (end, zone) in [(hx, BZone::AttackCircle), (-hx, BZone::OwnCircle)] {
        let c = Vec2::new(end, 0.0);
        let at = Vec2::new(x, q.y);
        if !zones.contains(&zone) && at.distance(c) < 5.1 {
            let out = c + (at - c).normalize_or(Vec2::new(-end.signum(), 0.0)) * 5.1;
            x = out.x;
            q.y = out.y;
        }
    }
    q.x = x * d;
    q
}

/// Everyone not mid-action moving the way her role and pace say.
fn locomotion(w: &mut World) {
    let g = game(w);
    let (sport, attack, holder, phase) = (g.sport, g.attack, g.ball.holder(), g.phase);
    for p in players(w) {
        let a = *w.read::<BAct>(p).expect("acting");
        if a.mv.is_action() && !done(w, p) {
            continue;
        }
        let b = body(w, p);
        let r = role(w, p);
        let moving = b.vel.length() > 0.6 || b.target.distance(b.at) > 0.5;
        let mv = if Some(p) == holder {
            if sport.rules.steps == Some(1) { BMove::Pivot } else if sport.rules.steps.is_none() { BMove::Run } else { BMove::Dribble }
        } else if phase == BPhase::Live && r.team != attack && sport.rules.steps.is_some() && b.target.distance(b.at) < 1.5 {
            BMove::Defend
        } else if moving {
            BMove::Run
        } else {
            BMove::Idle
        };
        if !act(w, p, mv) {
            // A move that may not follow directly goes by way of standing.
            act(w, p, BMove::Idle);
            act(w, p, mv);
        }
    }
}
