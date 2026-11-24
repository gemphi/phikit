//! A model of the library: a USD file's meshes as parts, each its
//! triangles where the file puts them and its own colour, read with
//! phiusd. Phiogy shows only USD: anything else is turned into USD
//! first (`data/convert.tsv`).

use std::path::Path;

use appkit::pui::core::PPiece;

/// A part of a model: its name (the last step of its mesh's path), the
/// path itself (where it stands in the scene, inside what), and its
/// piece, in the file's own colour or the one given for none.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Part {
    pub name: String,
    pub path: String,
    pub piece: PPiece,
}

/// A line of a scene's tree: the part it stands for (the first under it,
/// for what holds parts), its name, its depth, and whether it holds any.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub part: usize,
    pub name: String,
    pub depth: usize,
    pub holds: bool,
}

/// A model: its parts, and how many triangles they have.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Model {
    pub part: Vec<Part>,
    pub count: usize,
}

impl Model {
    /// The model of a USD file; one with no parts where it cannot be
    /// read. A part the file gives no colour is in `colour`; a file's
    /// colours are linear, as USD keeps them, and are shown as seen.
    pub fn read(path: &Path, colour: [f32; 3]) -> Model {
        let scene = phiusd::read(path);
        let part: Vec<Part> = scene.mesh.iter().map(|mesh| {
            let name = mesh.path.rsplit('/').next().unwrap_or(&mesh.path).to_string();
            Part { name, path: mesh.path.clone(), piece: PPiece::flat(mesh.triangle().collect(), mesh.colour.map(seen).unwrap_or(colour)) }
        }).collect();
        Model { count: scene.count(), part }
    }

    /// Its scene as a tree, as Reality Composer Pro's navigator shows it:
    /// every step of every part's path once, in the order met, each set
    /// in by how deep it is, what holds parts before them.
    pub fn tree(&self) -> Vec<Node> {
        let mut out: Vec<(String, Node)> = Vec::new();
        for (index, part) in self.part.iter().enumerate() {
            let step: Vec<&str> = part.path.split('/').filter(|step| !step.is_empty()).collect();
            for depth in 0..step.len() {
                let at = step[..=depth].join("/");
                if out.iter().all(|(known, _)| *known != at) {
                    out.push((at, Node { part: index, name: step[depth].to_string(), depth, holds: depth + 1 < step.len() }));
                }
            }
        }
        out.into_iter().map(|(_, node)| node).collect()
    }

    /// Its pieces, each part in the colour given for it where one is.
    pub fn dressed(&self, colour: impl Fn(usize) -> Option<[f32; 3]>) -> Vec<PPiece> {
        let one = |(index, part): (usize, &Part)| PPiece { colour: colour(index).unwrap_or(part.piece.colour), ..part.piece.clone() };
        self.part.iter().enumerate().map(one).collect()
    }
}

/// A linear colour as it is seen (sRGB), as the picture is drawn.
fn seen(colour: [f32; 3]) -> [f32; 3] {
    colour.map(|c| match c <= 0.003_130_8 {
        true => c * 12.92,
        false => 1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055,
    })
}
