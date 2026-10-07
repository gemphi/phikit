//! One row of the table — Blender's `spreadsheet_layout.cc`: the element's
//! index in the left column, then its value under each of the domain's
//! columns (`column.rs`), alternate rows shaded.
//!
//! The values come from the one place they can: whichever of `SceneObject`'s
//! arrays the domain indexes. A vertex indexes `vertices` and `uvs`, a face
//! indexes `material_indices`, a corner indexes `corners.colors`. An index
//! past the end of its array prints nothing rather than a zero, because a
//! blank cell says "no value here" and a zero says "zero".

use super::column::columns;
use super::set::{BSpreadsheetDomain, FLOAT_WIDTH, INDEX_WIDTH, cell, number};
use crate::phiphx::scene::SceneObject;
use crate::v2::view::BView;
use crate::v2::view::prim::{BRowShade, BTableCell};

/// The cells one element fills under `name`'s components.
fn values(object: &SceneObject, domain: BSpreadsheetDomain, name: &str, index: usize) -> Vec<String> {
    let three = |v: Option<&[f32; 3]>| v.map_or_else(|| vec![String::new(); 3], |v| v.iter().copied().map(number).collect());
    match (domain, name) {
        (_, "position") => three(object.mesh.vertices.get(index)),
        (_, "UVMap") => object
            .uvs
            .get(index)
            .map_or_else(|| vec![String::new(); 2], |uv| uv.iter().copied().map(number).collect()),
        (_, "material_index") => vec![object.material_indices.get(index).map(u16::to_string).unwrap_or_default()],
        (_, "Color") => three(object.mesh.corners.colors.get(index)),
        _ => Vec::new(),
    }
}

/// One element's row: its index, its values, and the shade its parity gives
/// it.
pub fn row(object: &SceneObject, domain: BSpreadsheetDomain, index: usize) -> BView {
    let mut list = vec![cell(index.to_string(), INDEX_WIDTH)];
    for spec in columns(object, domain) {
        let mut got = values(object, domain, spec.name, index);
        got.resize(spec.width(), String::new());
        list.extend(got.into_iter().map(|text| cell(text, FLOAT_WIDTH)));
    }
    list.push(BTableCell::flex(BView::empty()));
    let shade = if index.is_multiple_of(2) { BRowShade::Flat } else { BRowShade::Alt };
    BView::table_row(list, shade, None)
}
