//! The Spreadsheet's content — Blender's `spreadsheet_main_region_draw`:
//! the active object's evaluated geometry, one row per element of the
//! chosen domain, the row's index in the left column and its attributes
//! across. The Vertex domain and its `position` attribute are what a fresh
//! Spreadsheet opens on; the columns each domain carries are `column.rs`
//! and the rows `row.rs`, which is how Blender splits it
//! (`spreadsheet_column.cc`, `spreadsheet_layout.cc`).

use crate::phiphx::scene::SceneObject;
use crate::v2::state::{BState, CtxKey};
use crate::v2::view::prim::BTableCell;
use crate::v2::view::BView;

/// Blender draws the rows the region can show; a table this long scrolls,
/// and past this many the scroller, not the data, is what is being tested.
pub const ROW_MAX: usize = 512;
/// The index column, `SPREADSHEET_ROW_INDEX_WIDTH`-ish at scale 1.
pub const INDEX_WIDTH: f32 = 60.0;
/// One float column of a vector attribute.
pub const FLOAT_WIDTH: f32 = 96.0;

/// A geometry domain, as the dataset region lists them
/// (`spreadsheet_dataset_draw.cc`: Vertex, Edge, Face, Face Corner for a
/// mesh).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BSpreadsheetDomain {
    #[default]
    Vertex,
    Edge,
    Face,
    FaceCorner,
}

impl BSpreadsheetDomain {
    pub fn label(self) -> &'static str {
        match self {
            Self::Vertex => "Vertex",
            Self::Edge => "Edge",
            Self::Face => "Face",
            Self::FaceCorner => "Face Corner",
        }
    }

    /// The domains the dataset region offers for a mesh here. **Edge is
    /// not among them**: the reader triangulates, so `SceneMesh` carries
    /// `indices` and no edge list — the edges of a triangulated mesh are
    /// not the file's edges (a quad's diagonal is not one), and a count
    /// derived from the triangles would be wrong rather than missing.
    /// `.edge_verts` is read for creases (`phiphx`'s `mesh/crease.rs`) but
    /// does not reach the scene, which is what would have to change.
    pub const OFFERED: &'static [Self] = &[Self::Vertex, Self::Face, Self::FaceCorner];

    /// The identifier the host key holds (`SpaceSpreadsheet.attribute_
    /// domain`).
    pub fn slug(self) -> &'static str {
        match self {
            Self::Vertex => "POINT",
            Self::Edge => "EDGE",
            Self::Face => "FACE",
            Self::FaceCorner => "CORNER",
        }
    }

    /// The domain that slug names; anything else is the Vertex domain, as
    /// a fresh Spreadsheet opens on.
    pub fn of(slug: &str) -> Self {
        Self::OFFERED.iter().copied().find(|d| d.slug() == slug).unwrap_or_default()
    }

    /// The domain the Spreadsheet is showing.
    pub fn now(s: &BState) -> Self {
        Self::of(s.host(crate::v2::host::BHostType::Spreadsheet).get("domain").unwrap_or(""))
    }

    /// How many elements the mesh has in this domain: a vertex each, a
    /// triangle each for Face, and three corners a triangle.
    pub fn count(self, mesh: &crate::phiphx::scene::SceneMesh) -> usize {
        match self {
            Self::Vertex => mesh.vertices.len(),
            Self::Edge => 0,
            Self::Face => mesh.indices.len() / 3,
            Self::FaceCorner => mesh.indices.len(),
        }
    }
}

/// The active object of the loaded scene, by name.
pub fn active_object(s: &BState) -> Option<SceneObject> {
    let active = s.get(CtxKey::Object)?.to_string();
    crate::v2::scene::source::with_loaded(|loaded| {
        loaded.and_then(|model| model.objects.iter().find(|object| object.name == active).cloned())
    })
}

/// Blender prints spreadsheet floats to three decimals.
pub fn number(value: f32) -> String {
    format!("{value:.3}")
}

pub fn cell(text: impl Into<String>, width: f32) -> BTableCell {
    BTableCell::fixed_end(BView::label(text), width)
}

/// The dataset line above the table: the object and the chosen domain with
/// its element count, as the dataset region's tree reads.
pub fn caption(object: &SceneObject, domain: BSpreadsheetDomain) -> BView {
    BView::label(format!("{}  ·  Mesh  ·  {}  {}", object.name, domain.label(), domain.count(&object.mesh)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_print_to_three_decimals_and_a_domain_counts_its_elements() {
        assert_eq!(number(1.0), "1.000");
        assert_eq!(number(-0.25), "-0.250");
        // Two triangles over four vertices: four vertices, two faces, six
        // corners. Edge is not offered, so it counts nothing.
        let mesh = crate::phiphx::scene::SceneMesh {
            vertices: vec![[0.0; 3]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            normals: crate::phiphx::scene::SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]; 2]),
            corners: Default::default(),
        };
        let count = |d: BSpreadsheetDomain| d.count(&mesh);
        assert_eq!((count(BSpreadsheetDomain::Vertex), count(BSpreadsheetDomain::Face), count(BSpreadsheetDomain::FaceCorner)), (4, 2, 6));
        assert_eq!(count(BSpreadsheetDomain::Edge), 0, "the reader triangulates and keeps no edge list");
        assert_eq!(BSpreadsheetDomain::OFFERED.len(), 3, "Vertex, Face, Face Corner");
        assert_eq!(BSpreadsheetDomain::of("CORNER"), BSpreadsheetDomain::FaceCorner);
        assert_eq!(BSpreadsheetDomain::of("EDGE"), BSpreadsheetDomain::Vertex, "a domain not offered falls back to the one a fresh Spreadsheet opens on");
        assert_eq!(BSpreadsheetDomain::of(""), BSpreadsheetDomain::Vertex);
    }
}
