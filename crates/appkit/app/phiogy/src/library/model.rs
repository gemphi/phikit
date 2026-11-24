//! A model of the library: a USD file's meshes as parts, each its
//! triangles where the file puts them and its own colour, read with
//! phiusd. Phiogy shows only USD: anything else is turned into USD
//! first (`data/convert.tsv`).

use std::path::Path;

use appkit::pui::core::PPiece;

/// A part of a model: its name (the last step of its mesh's path) and
/// its piece, in the file's own colour or the one given for none.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Part {
    pub name: String,
    pub piece: PPiece,
}

/// A model: its parts, and how many triangles they have.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Model {
    pub part: Vec<Part>,
    pub count: usize,
}

impl Model {
    /// The model of a USD file; one with no parts where it cannot be
    /// read. A part the file gives no colour is in `colour`.
    pub fn read(path: &Path, colour: [f32; 3]) -> Model {
        let scene = phiusd::read(path);
        let part: Vec<Part> = scene.mesh.iter().map(|mesh| {
            let name = mesh.path.rsplit('/').next().unwrap_or(&mesh.path).to_string();
            Part { name, piece: PPiece::flat(mesh.triangle().collect(), mesh.colour.unwrap_or(colour)) }
        }).collect();
        Model { count: scene.count(), part }
    }

    /// Its pieces, each part in the colour given for it where one is.
    pub fn dressed(&self, colour: impl Fn(usize) -> Option<[f32; 3]>) -> Vec<PPiece> {
        let one = |(index, part): (usize, &Part)| PPiece { colour: colour(index).unwrap_or(part.piece.colour), ..part.piece.clone() };
        self.part.iter().enumerate().map(one).collect()
    }
}
