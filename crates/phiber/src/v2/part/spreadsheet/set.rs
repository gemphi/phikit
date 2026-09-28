//! The Spreadsheet's content — Blender's `spreadsheet_main_region_draw`:
//! the active object's evaluated geometry, one row per element of the
//! chosen domain, the row's index in the left column and its attributes
//! across. The Vertex domain and its `position` attribute are what a fresh
//! Spreadsheet opens on, and what is drawn here.

use crate::phiphx::scene::SceneObject;
use crate::v2::state::{BState, CtxKey};
use crate::v2::view::prim::{BRowShade, BTableCell};
use crate::v2::view::BView;

/// Blender draws the rows the region can show; a table this long scrolls,
/// and past this many the scroller, not the data, is what is being tested.
pub const ROW_MAX: usize = 512;
/// The index column, `SPREADSHEET_ROW_INDEX_WIDTH`-ish at scale 1.
pub const INDEX_WIDTH: f32 = 60.0;
/// One float column of a vector attribute.
pub const FLOAT_WIDTH: f32 = 96.0;

/// A geometry domain, as the dataset region lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BSpreadsheetDomain {
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

fn cell(text: impl Into<String>, width: f32) -> BTableCell {
    BTableCell::fixed_end(BView::label(text), width)
}

/// The header row: the index column, then `position` as x, y, z.
pub fn header() -> BView {
    BView::table_header(vec![
        cell("", INDEX_WIDTH),
        cell("position x", FLOAT_WIDTH),
        cell("y", FLOAT_WIDTH),
        cell("z", FLOAT_WIDTH),
        BTableCell::flex(BView::empty()),
    ])
}

/// One vertex's row.
pub fn row(index: usize, position: [f32; 3]) -> BView {
    let shade = if index % 2 == 0 { BRowShade::Flat } else { BRowShade::Alt };
    BView::table_row(
        vec![
            cell(index.to_string(), INDEX_WIDTH),
            cell(number(position[0]), FLOAT_WIDTH),
            cell(number(position[1]), FLOAT_WIDTH),
            cell(number(position[2]), FLOAT_WIDTH),
            BTableCell::flex(BView::empty()),
        ],
        shade,
        None,
    )
}

/// The dataset line above the table: the object and the domain with its
/// element count, as the dataset region's tree reads.
pub fn caption(object: &SceneObject) -> BView {
    BView::label(format!(
        "{}  ·  Mesh  ·  {}  {}",
        object.name,
        BSpreadsheetDomain::Vertex.label(),
        object.mesh.vertices.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_prints_its_index_and_three_decimals() {
        assert_eq!(number(1.0), "1.000");
        assert_eq!(number(-0.25), "-0.250");
        let BView::TableRow { cell_list, shade, .. } = row(3, [1.0, 2.5, -3.0]) else { panic!("a table row") };
        assert_eq!(cell_list.len(), 5);
        assert_eq!(shade, BRowShade::Alt, "odd rows take the alternate shade");
        assert!(matches!(cell_list[0].view.as_ref(), BView::Label { text } if text == "3"));
        assert!(matches!(cell_list[3].view.as_ref(), BView::Label { text } if text == "-3.000"));
    }
}
