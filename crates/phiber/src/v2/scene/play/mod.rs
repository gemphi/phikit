//! A match as a scene: the Play host's choice (`phiplay::BChoice`) built
//! into a [`SceneModel`] the viewport, the timeline and the renderer take
//! like any opened file.
//!
//! phiplay plays the match (`phiplay::play`, a tape of every frame) and
//! generates everything that is not a body (`phiplay::build`); this module
//! puts the two together:
//!
//! - the arena — field, goals, the venue's bowl, shell, roof and boards,
//!   the seats and the crowd, the scoreboard — as objects, the seats merged
//!   per colour and the fans sharing their meshes (`convert`);
//! - the players — each the demo's figure adopted with her own rig and
//!   posed from her track, dressed in garments cut from that figure and
//!   skinned to her rig (`figure`, `motion`);
//! - the ball, its glow and the scoreboard's digits, moved by the tape;
//! - the venue's lamps as lights and a broadcast camera on the halfway line.

pub mod convert;
pub mod figure;
pub mod motion;
mod stage;

pub use stage::{demo, open};
use stage::{broadcast, light, report, tint};

use crate::phiphx::blend::BPlayRef;
use crate::phiphx::scene::build::BSkinnedMesh;
use crate::phiphx::scene::SceneCollection;
use crate::phiphx::scene::SceneModel;
use figure::BFigure;
use glam::Mat4;
use phiplay::BChoice;
use phiplay::build::{ball, board, dress, field, goal, venue};
use phiplay::play::FPS;
use std::collections::HashMap;
use std::sync::Arc;

/// How long a match plays, seconds.
pub const SECONDS: f32 = 60.0;

/// The façade's three strengths and the levels of the show each stands
/// for: lit above 0.6, dim above 0.2, dark below.
const FACADE_STEPS: [(f32, f32); 3] = [(0.6, f32::INFINITY), (0.2, 0.6), (f32::NEG_INFINITY, 0.2)];

/// The glow's scale over the ball in flight, and inside it at rest.
const GLOW: (f32, f32) = (1.025, 0.96);

/// The match `choice` picks, `seconds` of it, built on the figure in
/// `demo` (the embedded demo court). `None` when `demo` has no rigged body.
pub fn arena(choice: &BChoice, seconds: f32, demo: &SceneModel) -> Option<SceneModel> {
    let figure = BFigure::of(demo)?;
    let tape = Arc::new(phiplay::play::play(choice, seconds));
    let frames = tape.frames() as i32;
    let sport = choice.sport.spec();
    let theme = choice.theme.spec();
    let mut model = SceneModel {
        timeline: crate::phiphx::scene::SceneTimeline { frame_start: 1, frame_end: frames.max(1), frame_step: 1, current_frame: 1, fps: FPS },
        world: demo.world.clone(),
        render: demo.render.clone(),
        playback: demo.playback.clone(),
        ..Default::default()
    };
    let sky = theme.sky.linear();
    model.world.horizon_color = sky;
    model.world.background.color = sky;
    model.world.background.strength = if choice.venue.spec().roof == phiplay::venue::BRoof::Closed { 0.3 } else { 1.0 };
    let mut groups: HashMap<&'static str, Vec<usize>> = HashMap::new();

    // The arena.
    let built = venue::venue(choice);
    let mut parts = vec![field::floor(sport, choice.turf().spec(), theme)];
    parts.extend(goal::goals(sport, theme));
    parts.extend(built.parts.iter().cloned());
    let venue_spec = choice.venue.spec();
    for p in &parts {
        // A façade panel runs the venue's show: drawn at three strengths,
        // the one the show's level falls in at each frame shown.
        if let Some(g) = built.groups.iter().find(|g| g.material == p.name && g.material.starts_with("Facade")) {
            for (k, (lo, hi)) in FACADE_STEPS.iter().enumerate() {
                let strength = g.strength * [1.0, 0.4, 0.06][k];
                let part = phiplay::build::BPart { name: format!("{} {}", p.name, ["Lit", "Dim", "Dark"][k]), material: Arc::new(phiplay::build::BMat { strength, ..(*p.material).clone() }), ..p.clone() };
                let play = BPlayRef(Arc::new(motion::BFacade { frames: frames as usize, show: venue_spec.show, tempo: theme.tempo, at: (g.s, g.t), band: (*lo, *hi) }));
                let i = model.add_object(convert::object(&part), Some(play));
                groups.entry("Arena").or_default().push(i);
            }
            continue;
        }
        let i = model.add_object(convert::object(p), None);
        groups.entry("Arena").or_default().push(i);
    }
    // Seats: each colour one mesh, merged — they never move.
    for list in &built.seats {
        let mut merged = phiplay::build::BMesh::default();
        for at in &list.at {
            merged.merge(&list.mesh, *at);
        }
        let part = phiplay::build::BPart { name: list.name.clone(), mesh: Arc::new(merged), material: list.material.clone(), at: Mat4::IDENTITY };
        let i = model.add_object(convert::object(&part), None);
        groups.entry("Seats").or_default().push(i);
    }
    // The crowd: merged into one object a palette, every fan of that
    // palette in its seat. The tracer keeps a tree of its own for each
    // object (it does not share one between objects drawing one mesh yet),
    // so three thousand objects cost three thousand trees for the same
    // triangles twenty-four do.
    let mut palettes: Vec<(*const phiplay::build::BMat, phiplay::build::BPart)> = Vec::new();
    for list in &built.crowd {
        let at = Arc::as_ptr(&list.material);
        let k = match palettes.iter().position(|(p, _)| *p == at) {
            Some(k) => k,
            None => {
                let name = format!("Crowd {}", list.material.name.trim_start_matches("Fan "));
                palettes.push((at, phiplay::build::BPart { name, mesh: Arc::new(phiplay::build::BMesh::default()), material: list.material.clone(), at: Mat4::IDENTITY }));
                palettes.len() - 1
            }
        };
        let merged = Arc::make_mut(&mut palettes[k].1.mesh);
        for at in &list.at {
            merged.merge(&list.mesh, *at);
        }
    }
    for (_, part) in &palettes {
        let i = model.add_object(convert::object(part), None);
        groups.entry("Crowd").or_default().push(i);
    }
    // The scoreboard: its slab and screens still, its digits played.
    let board = board::board(choice);
    for p in &board.parts {
        let i = model.add_object(convert::object(p), None);
        groups.entry("Scoreboard").or_default().push(i);
    }
    for d in &board.digits {
        let play = BPlayRef(Arc::new(motion::BDigit { tape: tape.clone(), slot: d.slot, value: d.value }));
        let i = model.add_object(convert::object(&d.part), Some(play));
        groups.entry("Scoreboard").or_default().push(i);
    }
    // The ball and its glow.
    let ball_part = ball::ball(&sport.ball, "Ball");
    let i = model.add_object(convert::object(&ball_part), Some(BPlayRef(Arc::new(motion::BBall { tape: tape.clone(), glow: None }))));
    groups.entry("Ball").or_default().push(i);
    let glow = phiplay::build::BPart::new("Ball Glow", phiplay::build::BMesh::sphere(sport.ball.radius, 32, 16), phiplay::build::BMat::glowing("Ball Glow", theme.glow, 2.2), Mat4::IDENTITY);
    let i = model.add_object(convert::object(&glow), Some(BPlayRef(Arc::new(motion::BBall { tape: tape.clone(), glow: Some(GLOW) }))));
    groups.entry("Ball").or_default().push(i);

    // The players.
    for (track, t) in tape.players.iter().enumerate() {
        let team = if t.team == 0 { choice.home } else { choice.away };
        let player = phiplay::look::roster::roster(team)[t.roster];
        let name = format!("{} {}", team.spec().code, player.number);
        let play = BPlayRef(Arc::new(motion::BPlayer { tape: tape.clone(), track, skeleton: figure.skeleton.clone(), height: player.look.spec().height }));
        let adopted = model.adopt_rig(demo, figure.rig, &name, |mesh| figure.keeps(mesh), play);
        groups.entry("Players").or_default().extend(adopted.objects.iter().map(|(_, i)| *i));
        // Her skin and the hair of her brows and lashes in her own colours.
        let look = player.look.spec();
        let hair_colour = look.hair[usize::from(player.hair_colour) % look.hair.len()];
        for &(_, i) in &adopted.objects {
            for m in &mut model.objects[i].materials {
                let colour = match m.name.as_str() {
                    "Skin" => Some(look.skin),
                    "Hair" => Some(hair_colour),
                    _ => None,
                };
                if let Some(c) = colour {
                    tint(m, c);
                }
            }
        }
        for garment in dress::dress(&figure.body, choice, t.team, &player) {
            let mesh = BSkinnedMesh {
                vertices: garment.piece.positions.clone(),
                faces: garment.piece.faces.clone(),
                uvs: garment.piece.uvs.clone(),
                face_materials: garment.piece.face_materials.clone(),
                groups: figure.body.groups.clone(),
                weights: garment.piece.weights.clone(),
                smooth: true,
            };
            let materials = garment.materials.iter().map(convert::material).collect();
            if let Some(i) = model.add_skinned(adopted.rig, &format!("{name} {}", garment.name), mesh, materials) {
                groups.entry("Players").or_default().push(i);
            }
        }
    }

    // The lamps and the camera.
    for (k, lamp) in built.lamps.iter().enumerate() {
        model.lights.push(light(k, lamp));
    }
    if let Some(camera) = broadcast(choice, demo) {
        model.render.camera = camera.name.clone();
        model.extra_locals.push((camera.name.clone(), camera.matrix_world));
        model.cameras.push(camera);
    }
    for l in &model.lights {
        model.extra_locals.push((l.name.clone(), l.matrix_world));
    }

    // The Outliner's collections: one for each kind of thing.
    let root = model.collections.first_mut()?;
    for name in ["Arena", "Seats", "Crowd", "Scoreboard", "Ball", "Players"] {
        let members = groups.remove(name).unwrap_or_default();
        root.object_indices.retain(|i| !members.contains(i));
        root.children.push(SceneCollection { name: name.to_string(), children: Vec::new(), object_indices: members, extra_names: Vec::new(), exclude: false, hide_viewport: false, visibility: Default::default() });
    }
    root.extra_names = model.cameras.iter().map(|c| c.name.clone()).chain(model.lights.iter().map(|l| l.name.clone())).collect();
    report(&mut model, choice);
    Some(model)
}

#[cfg(test)]
mod tests;

