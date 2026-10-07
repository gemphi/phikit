//! Spreadsheet — Blender's `space_spreadsheet`: the active object's
//! evaluated geometry as a table, one row per element of the domain the
//! dataset region chose. Structure only; the domain is `set.rs`, the
//! columns `column.rs`, the rows `row.rs` and the dataset region
//! `dataset.rs`.

pub mod column;
pub mod dataset;
pub mod row;
pub mod set;

use crate::v2::state::BState;
use crate::v2::view::prim::{BCard, BColumn};
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

#[derive(Clone, Copy, Debug, Default)]
pub struct BSpreadsheetItem;

impl BSpreadsheetItem {
    pub fn new() -> Self {
        Self
    }
}

impl BViewItem for BSpreadsheetItem {
    fn build(&self, s: &BState) -> BView {
        let Some(object) = set::active_object(s) else {
            // Blender shows an empty table when the active object has no
            // geometry to list.
            return BView::from(BCard::column(BColumn::Body, Vec::new()));
        };
        // The domain the dataset region chose, and its elements.
        let domain = set::BSpreadsheetDomain::now(s);
        let rows = domain.count(&object.mesh).min(set::ROW_MAX);
        let mut list = vec![set::caption(&object, domain), column::header(&object, domain)];
        list.extend((0..rows).map(|index| row::row(&object, domain, index)));
        BView::from(BCard::column(BColumn::Body, list))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BSpreadsheetList;

impl BSpreadsheetList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BSpreadsheetList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BSpreadsheetItem::new().build(s)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::scene::{SceneMesh, SceneModel, SceneNormals, SceneObject};
    use crate::v2::scene::{clear_loaded_scene, set_loaded_scene};
    use crate::v2::state::CtxKey;

    fn cube(name: &str) -> SceneObject {
        SceneObject {
            name: name.to_string(),
            mesh: SceneMesh {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.5, 0.0]],
                indices: vec![0, 1, 2],
                normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
                corners: Default::default(),
            }
            .into(),
            location: [0.0; 3],
            scale: [1.0; 3],
            matrix_world: crate::phiphx::scene::IDENTITY_M4,
            materials: Vec::new(),
            material_indices: Default::default(),
            uvs: Default::default(),
            color: [1.0; 4],
            shadow: Default::default(),
        }
    }

    /// Every domain the dataset region offers, with the columns the mesh
    /// actually carries in it — a UV layer on the Vertex domain (the reader
    /// keeps UVs per vertex, where Blender's are per corner), a
    /// `material_index` on the Face domain when the faces were assigned,
    /// and a colour attribute on the Face Corner domain when the corners
    /// carry one. A domain whose attributes the mesh has none of is an
    /// index column and nothing else, which is what Blender shows too.
    #[test]
    fn each_domain_lists_its_own_elements_and_the_columns_the_mesh_carries() {
        use crate::v2::host::BHostType;
        use crate::v2::state::BPath;
        let mut object = cube("Tri");
        // Two triangles over four vertices: 4 vertices, 2 faces, 6 corners.
        let mesh = SceneMesh {
            vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]; 2]),
            corners: crate::phiphx::scene::SceneCorners { ids: vec![0; 6], normals: Vec::new(), colors: vec![[1.0, 0.5, 0.0]; 6] },
        };
        object.mesh = mesh.into();
        object.uvs = std::sync::Arc::new(vec![[0.25, 0.75], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        object.material_indices = std::sync::Arc::new(vec![0, 3]);
        let mut s = BState::default();
        s.set(CtxKey::Object, "Tri");
        set_loaded_scene(Some(SceneModel { objects: vec![object], ..Default::default() }));
        let rows_of = |s: &BState| -> Vec<Vec<String>> {
            let BView::Card(card) = BSpreadsheetItem::new().build(s) else { panic!("a body card") };
            card.body
                .iter()
                .filter_map(|v| match v {
                    BView::TableRow { cell_list, .. } => Some(
                        cell_list
                            .iter()
                            .filter_map(|c| match c.view.as_ref() {
                                BView::Label { text } => Some(text.clone()),
                                _ => None,
                            })
                            .collect(),
                    ),
                    _ => None,
                })
                .collect()
        };
        let pick = |s: &mut BState, slug: &str| s.set(BPath::owned(vec![BHostType::Spreadsheet.slug(), "domain"]), slug.to_string());

        // Vertex: position and the UV layer.
        let vertex = rows_of(&s);
        assert_eq!(vertex.len(), 4, "one row per vertex");
        assert_eq!(vertex[0], ["0", "0.000", "0.000", "0.000", "0.250", "0.750"], "index, position, UV");

        // Face: one row per triangle, `material_index` only.
        pick(&mut s, "FACE");
        let face = rows_of(&s);
        assert_eq!(face.len(), 2, "one row per triangle");
        assert_eq!(face[1], ["1", "3"], "the second face's material slot");

        // Face Corner: one row per corner, the colour attribute.
        pick(&mut s, "CORNER");
        let corner = rows_of(&s);
        assert_eq!(corner.len(), 6, "three corners a triangle");
        assert_eq!(corner[0], ["0", "1.000", "0.500", "0.000"]);

        // A domain the reader cannot offer falls back to Vertex rather than
        // showing an empty or a wrong table: the mesh is triangulated, so
        // it carries no edge list (`BSpreadsheetDomain::OFFERED`).
        pick(&mut s, "EDGE");
        assert_eq!(rows_of(&s).len(), 4, "Edge is not offered; the Vertex domain stands");

        // The dataset region names each domain with its count, and the
        // chosen one is the active row.
        pick(&mut s, "FACE");
        let BView::Card(tree) = dataset::BDataset::new().build(&s) else { panic!("a body card") };
        let labels: Vec<(String, bool)> = tree
            .body
            .iter()
            .filter_map(|v| match v {
                BView::Button(c) => Some((c.label.clone().unwrap_or_default(), c.active)),
                _ => None,
            })
            .collect();
        assert_eq!(labels, [("Vertex  4".into(), false), ("Face  2".into(), true), ("Face Corner  6".into(), false)]);
        clear_loaded_scene();
    }

    #[test]
    fn the_table_lists_the_active_objects_vertices_and_nothing_without_one() {
        let mut s = BState::default();
        s.set(CtxKey::Object, "Tri");
        set_loaded_scene(Some(SceneModel { objects: vec![cube("Other"), cube("Tri")], ..Default::default() }));
        let BView::Card(card) = BSpreadsheetItem::new().build(&s) else { panic!("a body card") };
        let rows: Vec<&BView> = card.body.iter().filter(|v| matches!(v, BView::TableRow { .. })).collect();
        assert_eq!(rows.len(), 3, "one row per vertex of the active object");
        assert!(card.body.iter().any(|v| matches!(v, BView::TableHeader { .. })), "a header row");
        assert!(card.body.iter().any(|v| matches!(v, BView::Label { text } if text.starts_with("Tri") && text.ends_with('3'))), "the dataset line names the object and the count");
        clear_loaded_scene();
        let BView::Card(card) = BSpreadsheetItem::new().build(&s) else { panic!("a body card") };
        assert!(card.body.iter().all(|v| !matches!(v, BView::TableRow { .. })), "no scene, no rows");
    }
}
