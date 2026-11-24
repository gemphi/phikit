//! The ordinary shapes every asset library starts with — cube, dice,
//! sphere, cylinder, cone, torus, plane, capsule, pyramid and the regular
//! solids — made from `data/shape.tsv`. The table's rows are the shapes:
//! their names, measures, parts and looks; code only knows how each form
//! is made. `export::usd` writes them as USDA (`Shape::to_usda`).

mod lathe;
pub mod part;
pub mod pip;
mod solid;
#[cfg(test)]
mod tests;

pub use part::{ShapeMesh, ShapePart};
pub use pip::PipFace;

use crate::table::{TsvRow, TsvTable};
use part::Frame;

/// How a shape is made (the table's `form` column).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeForm {
    Box,
    Sphere,
    Cylinder,
    Cone,
    Torus,
    Plane,
    Capsule,
    Pyramid,
    Tetrahedron,
    Octahedron,
    Icosahedron,
}

impl ShapeForm {
    /// The form a table word names.
    pub fn from_word(word: &str) -> Option<Self> {
        match word {
            "box" => Some(Self::Box),
            "sphere" => Some(Self::Sphere),
            "cylinder" => Some(Self::Cylinder),
            "cone" => Some(Self::Cone),
            "torus" => Some(Self::Torus),
            "plane" => Some(Self::Plane),
            "capsule" => Some(Self::Capsule),
            "pyramid" => Some(Self::Pyramid),
            "tetrahedron" => Some(Self::Tetrahedron),
            "octahedron" => Some(Self::Octahedron),
            "icosahedron" => Some(Self::Icosahedron),
            _ => None,
        }
    }

    /// Whether it encloses a volume (every edge shared by two faces).
    pub fn closed(self) -> bool {
        !matches!(self, Self::Plane)
    }
}

/// A part's name and look, from the table.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeLook {
    pub name: String,
    pub colour: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
}

/// One row of `data/shape.tsv`: a shape's name, form, measures (metres)
/// and the looks of its parts, in the order the form makes them.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub name: String,
    pub form: ShapeForm,
    pub size: f32,
    pub height: f32,
    pub segment: usize,
    pub ring: usize,
    pub bevel: f32,
    pub pip: f32,
    pub pip_depth: f32,
    pub spread: f32,
    pub look: Vec<ShapeLook>,
}

impl Shape {
    /// Every shape the table lists, in table order.
    pub fn all() -> Vec<Shape> {
        let table = TsvTable::parse(include_str!("../../data/shape.tsv"));
        table.row().filter_map(Self::from_row).collect()
    }

    /// The shape named `name`, if the table has one.
    pub fn named(name: &str) -> Option<Shape> {
        Self::all().into_iter().find(|s| s.name == name)
    }

    fn from_row(row: TsvRow<'_>) -> Option<Shape> {
        let form = ShapeForm::from_word(row.get("form"))?;
        let (colour, rough, metal) = (row.number_list("colour"), row.number_list("roughness"), row.number_list("metallic"));
        let look = row
            .word_list("part")
            .iter()
            .enumerate()
            .map(|(i, name)| ShapeLook {
                name: name.to_string(),
                colour: [0, 1, 2].map(|c| colour.get(i * 3 + c).copied().unwrap_or(0.0)),
                roughness: rough.get(i).copied().unwrap_or(0.0),
                metallic: metal.get(i).copied().unwrap_or(0.0),
            })
            .collect();
        Some(Shape {
            name: row.get("name").to_string(),
            form,
            size: row.number("size"),
            height: row.number("height"),
            segment: row.count("segment"),
            ring: row.count("ring"),
            bevel: row.number("bevel"),
            pip: row.number("pip"),
            pip_depth: row.number("pip_depth"),
            spread: row.number("spread"),
            look,
        })
    }

    /// Its parts, each named and coloured as the table says: the body
    /// first, then (a dice) the pips.
    pub fn part(&self) -> Vec<ShapePart> {
        let mut body = ShapeMesh::default();
        let (r, id) = (self.size / 2.0, &Frame::IDENTITY);
        match self.form {
            ShapeForm::Box => solid::rounded_box(&mut body, self.size, self.bevel, self.ring),
            ShapeForm::Plane => solid::plane(&mut body, self.size),
            ShapeForm::Sphere => lathe::sweep(&mut body, &lathe::sphere(r, self.ring), self.segment, id),
            ShapeForm::Cylinder => lathe::sweep(&mut body, &lathe::cylinder(r, self.height), self.segment, id),
            ShapeForm::Cone => lathe::sweep(&mut body, &lathe::cone(r, self.height), self.segment, id),
            ShapeForm::Capsule => lathe::sweep(&mut body, &lathe::capsule(r, self.height, self.ring), self.segment, id),
            ShapeForm::Torus => lathe::sweep(&mut body, &lathe::torus(self.size, self.height, self.ring), self.segment, id),
            ShapeForm::Pyramid => solid::hull(&mut body, &solid::pyramid(self.size, self.height)),
            ShapeForm::Tetrahedron => solid::hull(&mut body, &solid::regular(4, self.size)),
            ShapeForm::Octahedron => solid::hull(&mut body, &solid::regular(6, self.size)),
            ShapeForm::Icosahedron => solid::hull(&mut body, &solid::regular(12, self.size)),
        }
        let mut part = Vec::new();
        let mut look = self.look.iter();
        if let Some(l) = look.next() {
            part.push(body.part(l));
        }
        match (self.pip > 0.0, look.next()) {
            (true, Some(l)) => {
                let mut pip = ShapeMesh::default();
                for face in PipFace::all() {
                    face.disc(&mut pip, self.size, self.spread, self.pip, self.pip_depth, self.segment);
                }
                part.push(pip.part(l));
            }
            _ => {}
        }
        part
    }
}
