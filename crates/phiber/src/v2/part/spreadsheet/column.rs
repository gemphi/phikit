//! The columns a domain carries — Blender's `spreadsheet_column.cc` and
//! `GeometryDataSource::foreach_default_column_ids`: one column per
//! attribute of the chosen domain, under the attribute's own name, with a
//! vector attribute spread over one column per component.
//!
//! Blender lists the attributes the geometry **has**. A name beginning with
//! a dot is internal (`allow_procedural_attribute_access`) and is hidden
//! unless the Filter panel asks for it, which is why `.corner_vert` and
//! `.select_vert` are not here — and why a mesh with no corner attribute
//! shows a Face Corner table of indices and nothing else, in Blender too.
//!
//! What the reader carries decides the rest: `position` always, the UV
//! layer when the mesh has one, `material_index` when the faces were
//! assigned per face, and the active colour attribute when the corners
//! carry one.

use super::set::{BSpreadsheetDomain, FLOAT_WIDTH, INDEX_WIDTH, cell};
use crate::phiphx::scene::SceneObject;
use crate::v2::view::BView;
use crate::v2::view::prim::BTableCell;

/// One attribute as the table shows it: its name, and the components it
/// spreads over (`["x", "y", "z"]` for a vector, `[]` for a scalar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BColumnSpec {
    pub name: &'static str,
    pub parts: &'static [&'static str],
}

impl BColumnSpec {
    /// How many table columns it takes.
    pub fn width(self) -> usize {
        self.parts.len().max(1)
    }
}

const POSITION: BColumnSpec = BColumnSpec { name: "position", parts: &["x", "y", "z"] };
/// The reader keeps UVs **per vertex**, where Blender's UV layer is per
/// corner — so the column sits on the Vertex domain here and on the Face
/// Corner domain in Blender. Said out loud because it is a difference, not
/// an oversight: `SceneObject::uvs` is `len == vertices.len()`.
const UVMAP: BColumnSpec = BColumnSpec { name: "UVMap", parts: &["u", "v"] };
const MATERIAL_INDEX: BColumnSpec = BColumnSpec { name: "material_index", parts: &[] };
const COLOR: BColumnSpec = BColumnSpec { name: "Color", parts: &["r", "g", "b"] };

/// The columns `object` has in `domain`, in Blender's order: the geometry's
/// own attributes, each only when the mesh carries it.
pub fn columns(object: &SceneObject, domain: BSpreadsheetDomain) -> Vec<BColumnSpec> {
    let mut out = Vec::new();
    match domain {
        BSpreadsheetDomain::Vertex => {
            out.push(POSITION);
            if !object.uvs.is_empty() {
                out.push(UVMAP);
            }
        }
        BSpreadsheetDomain::Face => {
            if !object.material_indices.is_empty() {
                out.push(MATERIAL_INDEX);
            }
        }
        BSpreadsheetDomain::FaceCorner => {
            if !object.mesh.corners.colors.is_empty() {
                out.push(COLOR);
            }
        }
        // Not offered (`BSpreadsheetDomain::OFFERED`).
        BSpreadsheetDomain::Edge => {}
    }
    out
}

/// The header row: the index column, then each attribute — its name over
/// the first component and the component letters over the rest, as Blender
/// writes `position x`, `y`, `z`.
pub fn header(object: &SceneObject, domain: BSpreadsheetDomain) -> BView {
    let mut list = vec![cell("", INDEX_WIDTH)];
    for spec in columns(object, domain) {
        match spec.parts.split_first() {
            Some((first, rest)) => {
                list.push(cell(format!("{} {first}", spec.name), FLOAT_WIDTH));
                list.extend(rest.iter().map(|p| cell(*p, FLOAT_WIDTH)));
            }
            None => list.push(cell(spec.name, FLOAT_WIDTH)),
        }
    }
    list.push(BTableCell::flex(BView::empty()));
    BView::table_header(list)
}
