//! Spreadsheet — Blender's `space_spreadsheet`: the active object's
//! evaluated geometry as a table. Structure only; the columns, the rows and
//! the domain line are `set.rs`.

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
            return BView::from(BCard::column(BColumn::Body, vec![set::header()]));
        };
        let mut list = vec![set::caption(&object), set::header()];
        list.extend(object.mesh.vertices.iter().take(set::ROW_MAX).enumerate().map(|(index, position)| set::row(index, *position)));
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
        }
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
