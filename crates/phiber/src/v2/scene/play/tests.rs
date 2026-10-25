use super::*;
use glam::Vec3;
use crate::phiphx::scene::SceneMesh;
use std::collections::{HashMap, HashSet};

fn built(choice: &BChoice, seconds: f32) -> (SceneModel, Arc<BTape>) {
    let demo = demo().expect("the embedded demo opens");
    let model = arena(choice, seconds, &demo).expect("the demo has a figure");
    (model, Arc::new(phiplay::play::play(choice, seconds)))
}

fn index(model: &SceneModel, name: &str) -> usize {
    model.objects.iter().position(|o| o.name == name).unwrap_or_else(|| panic!("no {name}"))
}

fn collection<'a>(model: &'a SceneModel, name: &str) -> &'a SceneCollection {
    model.collections[0].children.iter().find(|c| c.name == name).unwrap_or_else(|| panic!("no {name} collection"))
}

use phiplay::play::BTape;

/// The default match — basketball, ten players — built as a scene: every
/// player her own rig with her body, face and kit, the crowd a full house
/// on a few shared meshes, the ball and the clock moving with the tape.
#[test]
fn a_match_is_a_scene_of_rigged_dressed_players_in_a_full_arena() {
    let choice = BChoice::default();
    let (model, tape) = built(&choice, 10.0);
    assert_eq!(model.timeline.frame_end as usize, tape.frames());
    assert_eq!(model.motion.rigs.rigs.len(), 10, "five a side, each her own rig");
    // Every table in step.
    assert_eq!(model.objects.len(), model.object_types.len());
    assert_eq!(model.objects.len(), model.locals.len());
    assert_eq!(model.objects.len(), model.uv_maps.len());
    assert_eq!(model.objects.len(), model.motion.object.len());
    // Each player: her body and face, and everything she wears, skinned to
    // her rig.
    let home = choice.home.spec();
    let first = phiplay::look::roster::roster(choice.home)[0];
    let who = format!("{} {}", home.code, first.number);
    for part in ["Body", "Nose", "Eye.L", "Top", "Shorts", "Hair"] {
        index(&model, &format!("{who} {part}"));
    }
    assert!(model.objects.iter().any(|o| o.name.starts_with(&who) && o.name.contains("Shoes") || o.name.starts_with(&who) && o.name.contains("Sneakers")), "{who} wears shoes");
    let skinned: HashSet<usize> = model.motion.rigs.skins.iter().map(|s| s.object).collect();
    assert!(skinned.contains(&index(&model, &format!("{who} Top"))), "the kit is skinned");
    // Her skin is her look's.
    let body = &model.objects[index(&model, &format!("{who} Body"))];
    let [r, g, b] = first.look.spec().skin.linear();
    assert!(body.materials.iter().any(|m| m.base_color == [r, g, b, 1.0]), "{who}'s skin");
    // The crowd: a full house, one object a palette.
    let crowd = collection(&model, "Crowd");
    assert!(!crowd.object_indices.is_empty() && crowd.object_indices.len() <= 24, "{} crowd objects", crowd.object_indices.len());
    let fans: usize = venue::venue(&choice).crowd.iter().map(|l| l.at.len() * l.mesh.triangle_count()).sum();
    let drawn: usize = crowd.object_indices.iter().map(|&i| model.objects[i].mesh.indices.len() / 3).sum();
    assert_eq!(drawn, fans, "every fan drawn");
    // Lit and watched.
    assert!(!model.lights.is_empty() && model.cameras.len() == 1);
}

/// Played: the players move and bend, their kit with them; the ball flies;
/// the clock ticks.
#[test]
fn the_match_plays_on_the_timeline() {
    let choice = BChoice::default();
    let (model, tape) = built(&choice, 10.0);
    let who = format!("{} {}", choice.home.spec().code, phiplay::look::roster::roster(choice.home)[0].number);
    let world = |frame: f32, i: usize| model.motion.at(frame).object(i).expect("placed");
    // She stands where her track says (her body rides her rig).
    let f = 120usize;
    let step = &tape.players[0].steps[f - 1];
    let m = world(f as f32, index(&model, &format!("{who} Body")));
    assert!((m[3][0] - step.at.x).abs() < 1e-3 && (m[3][1] - step.at.y).abs() < 1e-3, "{:?} vs {:?}", m[3], step.at);
    // Posed: her body and her top move together, frame to frame.
    let posed = |frame: f32| -> HashMap<usize, SceneMesh> { model.motion.posed_at(frame).0.into_iter().collect() };
    let (a, b) = (posed(1.0), posed(f as f32));
    let (body, top) = (index(&model, &format!("{who} Body")), index(&model, &format!("{who} Top")));
    let centre = |m: &SceneMesh| m.vertices.iter().fold(Vec3::ZERO, |s, v| s + Vec3::from(*v)) / m.vertices.len() as f32;
    assert!(a[&body].vertices != b[&body].vertices, "the body bends");
    let (body_moved, top_moved) = (centre(&b[&body]) - centre(&a[&body]), centre(&b[&top]) - centre(&a[&top]));
    assert!((body_moved - top_moved).length() < 0.08, "the top goes with the body: {body_moved} vs {top_moved}");
    // The ball is where the tape has it.
    let ball = index(&model, "Ball");
    let p = world(60.0, ball)[3];
    let q = tape.ball[59];
    assert!((p[0] - q.x).abs() < 1e-3 && (p[2] - q.z).abs() < 1e-3);
    // The clock: one digit of each slot shown, the one the tape names.
    for frame in [1.0f32, 60.0, 239.0] {
        let f = frame as usize - 1;
        for slot in phiplay::build::board::BSlot::ALL {
            let shown: Vec<u8> = (0..10u8).filter(|v| world(frame, index(&model, &format!("Scoreboard {slot:?} {v}")))[3][2] > -50.0).collect();
            let want: Vec<u8> = slot.shows(tape.score[f], f).into_iter().collect();
            assert_eq!(shown, want, "{slot:?} at frame {frame}");
        }
    }
}

/// The façade runs the venue's show: each panel shows one of its three
/// strengths at every frame, and the show changes over the match.
#[test]
fn the_facade_runs_its_light_show() {
    let choice = BChoice { crowd: 10, ..BChoice::default() };
    let (model, _) = built(&choice, 10.0);
    let panel = |k: usize, step: &str| index(&model, &format!("Facade {k:02} {step}"));
    let shown = |frame: f32, i: usize| model.motion.at(frame).object(i).expect("placed")[3][2] > -50.0;
    let mut lit = Vec::new();
    for frame in [1.0f32, 30.0, 90.0, 200.0] {
        for k in 0..usize::from(choice.venue.spec().panels) {
            let on: Vec<bool> = ["Lit", "Dim", "Dark"].iter().map(|s| shown(frame, panel(k, s))).collect();
            assert_eq!(on.iter().filter(|x| **x).count(), 1, "panel {k} at frame {frame}: {on:?}");
            lit.push(on[0]);
        }
    }
    assert!(lit.iter().any(|x| *x) && lit.iter().any(|x| !*x), "the show lights some panels and not others");
}

/// Every sport builds: its own player count, its own field and goals.
#[test]
fn every_sport_builds_its_match() {
    for sport in phiplay::sport::BSport::ALL {
        let choice = BChoice { sport, crowd: 50, ..BChoice::default() };
        let (model, _) = built(&choice, 2.0);
        assert_eq!(model.motion.rigs.rigs.len(), 2 * usize::from(sport.spec().side), "{sport:?}");
    }
}
