//! The robot in 3D (RobotCharacter, JointPinSystem): each part's meshes
//! from its binary USD, placed where the world puts the part. Apple made
//! the parts in one rig's space, so the head and the backpack stand on
//! the body where they are: the head's foot meets the body's top. The
//! sample pins them to the body's joints of their names, which moves
//! them only as far as a joint moves from where it rests, and no clip
//! plays yet. Each mesh is drawn in its role's colour: the part's
//! material, the lights', the screen's (`paint.tsv`).

use phiecs::prelude::*;
use phiusd::draw::{Library, placed, placed_but};

use super::Data;
use super::look::{Look, MODEL};

pub use super::shape::{Piece, coarse, moved};
use super::shape::piece;

/// How sharp an edge stays sharp, from the game table (degrees).
fn crease(data: &Data) -> f32 {
    data.game.num("smooth.crease")
}

/// The role of a mesh, by the first row of a table (`paint.tsv` for
/// the robot, `scenery.tsv` for the greenhouse) its path holds; a path
/// none holds has the role given.
fn role<'a>(path: &str, table: &'a crate::table::Table, none: &'a str) -> &'a str {
    let lower = path.to_lowercase();
    table.row.iter().find(|row| row.first().is_some_and(|piece| lower.contains(piece.as_str()))).and_then(|row| row.get(1)).map_or(none, String::as_str)
}

/// A role's colour, for a part.
fn colour(role: &str, part: &str, look: &Look, data: &Data) -> [f32; 3] {
    let three = |key: &str| {
        let v: Vec<f32> = data.game.get(key).split_whitespace().filter_map(|n| n.parse().ok()).collect();
        [v.first().copied().unwrap_or_default(), v.get(1).copied().unwrap_or_default(), v.get(2).copied().unwrap_or_default()]
    };
    match role {
        "light" => {
            let row = data.light.row.get(look.light);
            let cell = |i: usize| row.and_then(|r| r.get(i)).and_then(|v| v.parse().ok()).unwrap_or_default();
            [cell(2), cell(3), cell(4)]
        }
        "part" => look.colour(part, data).map(|c| c as f32 / 255.0),
        other => three(&format!("{other}.colour")),
    }
}

/// The robot's pieces in the world, as the world now places it.
pub fn figure(world: &World, look: &Look, data: &Data, library: &mut Library) -> Vec<Piece> {
    let mut out = Vec::new();
    for (part, _) in &look.part {
        let Some(word) = world.lexicon.get(&format!("{MODEL}/{part}")) else { continue };
        for p in placed(world, word, library) {
            let scene = library.scene(&p.asset).clone();
            for mesh in &scene.mesh {
                out.push(piece(mesh, &p.matrix.then(&mesh.world), colour(role(&mesh.path, &data.paint, "part"), part, look, data), crease(data)));
            }
        }
    }
    out
}

/// Everything under a word but for what is under the words skipped, a
/// piece a mesh in its role's colour: the greenhouse, without the robot
/// walking in it.
pub fn scenery(world: &World, under: Word, skip: &[Word], look: &Look, data: &Data, library: &mut Library) -> Vec<Piece> {
    let mut out = Vec::new();
    for p in placed_but(world, under, skip, library) {
        for mesh in &library.scene(&p.asset).mesh {
            out.push(piece(mesh, &p.matrix.then(&mesh.world), colour(role(&mesh.path, &data.scenery, "scenery"), "", look, data), crease(data)));
        }
    }
    out
}
