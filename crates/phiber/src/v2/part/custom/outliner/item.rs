//! Outliner scene tree — Blender's View Layer rows, backed by the runtime
//! Scene_Source and BState.
//!
//! Rows come from [`tree::flatten`], which reads [`crate::v2::scene::scene_source`]
//! — the same single Scene_Source the Viewport reads — so the Outliner never
//! keeps an independent object list (Requirement 5.2). One row per object in
//! enumeration order (5.1); a zero-object scene emits zero object rows (5.3).
//! Selection is shared with the Viewport via `K_OBJECT` (5.4, 5.5).
//!
//! Row geometry matches `outliner_draw_tree_element`: indent by depth, a
//! reserved disclosure slot, the data-type icon, the name, then a flexible
//! gap and the right-aligned restriction column. Alternating bands come from
//! the palette's `row_alt`; the active object gets the selected fill.

use crate::v2::state::{
    BState,
    key::K_OBJECT,
    val::{V_FALSE, V_TRUE},
};
use crate::v2::view::prim::{BAction, BRowShade};
use crate::v2::view::{BView, BViewItem};

use super::set::TOGGLES;
use super::tree::{self, BOutlineRow, BRowKind};

#[derive(Clone, Copy, Debug, Default)]
pub struct BOutlinerItem;

impl BOutlinerItem {
    pub fn new() -> Self {
        Self
    }

    /// Disclosure action for a row that has children.
    fn toggle_action(row: &BOutlineRow) -> Option<BAction> {
        let open = row.expanded?;
        let next = if open { V_FALSE } else { V_TRUE };
        Some(BAction::Set(tree::k_expanded(row.id), next.to_string()))
    }

    /// One restriction toggle. Blender draws these as icon-only buttons in a
    /// fixed column at the row's right edge; the flag lives per object.
    fn restriction(s: &BState, object: &str, field: &'static str, icon: &'static str) -> BView {
        let key = tree::k_restrict(object, field);
        let on = s.get(&key) == Some(V_TRUE);
        let next = if on { V_FALSE } else { V_TRUE };
        // `eye` / `eye_off` is the one pair where the icon itself carries the
        // state; the rest dim via the button's own disabled-ish styling.
        let shown = if field == "hide_get" && on {
            "eye_off"
        } else {
            icon
        };
        BView::button_action(shown, "", BAction::Set(key, next.to_string()))
    }

    /// The restriction cluster for an object row. `tree_row` pins this to the
    /// row's right edge and stops it shrinking, so the toggles stay in a
    /// clean column however long the object names get.
    ///
    /// Only the columns Blender actually shows are drawn — the eye and the
    /// camera by default — with the rest opt-in via `ctx.outliner.col.<flag>`.
    fn restrictions(s: &BState, object: &str, view_layer: bool) -> Vec<BView> {
        let mut out = Vec::new();
        for t in TOGGLES {
            if t.kind.view_layer_only() && !view_layer {
                continue;
            }
            let shown = match s.get(&tree::k_column(t.state_key)) {
                Some(V_TRUE) => true,
                Some(V_FALSE) => false,
                _ => t.default_on,
            };
            if !shown {
                continue;
            }
            out.push(Self::restriction(s, object, t.state_key, t.icon));
        }
        out
    }

    fn row_view(
        s: &BState,
        row: &BOutlineRow,
        index: usize,
        active: &str,
        restrictions: bool,
    ) -> BView {
        let selected = row.object.as_deref() == Some(active);
        let shade = if selected {
            BRowShade::Selected
        } else if index % 2 == 1 {
            BRowShade::Alt
        } else {
            BRowShade::Flat
        };

        // The data-type icon is its own grid column, owned by `tree_row`;
        // only the name (and the search-hit marker) live in the main cells.
        let mut cells = Vec::new();
        match (&row.object, row.kind) {
            // Object rows are clickable — they set the active object.
            (Some(name), BRowKind::Object) => {
                cells.push(BView::button_action(
                    "",
                    row.label.clone(),
                    BAction::Set(K_OBJECT, name.clone()),
                ));
            }
            _ => cells.push(BView::label(row.label.clone())),
        }
        if row.matched {
            cells.push(BView::icon("search"));
        }

        let trail = match (&row.object, restrictions) {
            (Some(name), true) => Self::restrictions(s, name, true),
            _ => Vec::new(),
        };

        BView::tree_row(
            row.depth,
            row.expanded,
            Self::toggle_action(row),
            row.icon,
            cells,
            trail,
            shade,
        )
    }
}

impl BViewItem for BOutlinerItem {
    fn build(&self, s: &BState) -> BView {
        let active = s.get(&K_OBJECT).unwrap_or("Cube").to_string();
        let restrictions = tree::display_mode(s).shows_restrictions();
        let rows = tree::flatten(s);
        let views = rows
            .iter()
            .enumerate()
            .map(|(i, r)| Self::row_view(s, r, i, &active, restrictions))
            .collect();
        BView::column(views)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::OBJECTS;

    fn rows(state: &BState) -> Vec<BView> {
        let BView::Column { children } = BOutlinerItem::new().build(state) else {
            panic!()
        };
        children
    }

    #[test]
    fn default_scene_emits_one_row_per_object_under_two_group_rows() {
        let state = BState::new();
        let children = rows(&state);
        // Scene Collection + Collection + one row per Default_Scene object.
        assert_eq!(children.len(), 2 + OBJECTS.len());
        for row in &children[2..] {
            assert!(matches!(row, BView::TreeRow { .. }));
        }
    }

    #[test]
    fn group_rows_are_open_by_default_and_collapse_on_first_click() {
        let state = BState::new();
        let children = rows(&state);
        let BView::TreeRow {
            depth,
            expanded,
            toggle,
            ..
        } = &children[0]
        else {
            panic!()
        };
        assert_eq!(*depth, 0);
        assert_eq!(*expanded, Some(true));
        let Some(BAction::Set(_, value)) = toggle else {
            panic!()
        };
        assert_eq!(value, V_FALSE);
    }

    #[test]
    fn rows_indent_by_depth() {
        let state = BState::new();
        let children = rows(&state);
        let depths: Vec<u16> = children
            .iter()
            .map(|r| match r {
                BView::TreeRow { depth, .. } => *depth,
                _ => panic!(),
            })
            .collect();
        assert_eq!(depths[0], 0, "Scene Collection at the root");
        assert_eq!(depths[1], 1, "Collection nested once");
        assert!(depths[2..].iter().all(|d| *d == 2), "objects nested twice");
    }

    #[test]
    fn collapsing_the_scene_collection_hides_everything_below() {
        let mut state = BState::new();
        state.set(&tree::k_expanded("scene_collection"), V_FALSE);
        assert_eq!(rows(&state).len(), 1);
    }

    #[test]
    fn object_row_uses_shape_icon_and_sets_the_active_object() {
        let state = BState::new();
        let children = rows(&state);
        // The data-type icon is a grid column on the row itself, not a cell,
        // so the first main cell is the clickable name.
        let BView::TreeRow {
            children: cells,
            icon: name,
            ..
        } = &children[2]
        else {
            panic!()
        };
        let BView::Button(ctl) = &cells[0] else {
            panic!()
        };
        let Some(BAction::Set(path, value)) = ctl.action.as_ref() else {
            panic!()
        };
        // Rows are sorted alphabetically by default, so the first object row
        // is the alphabetically-first Default_Scene object.
        let mut names: Vec<&str> = OBJECTS.iter().map(|o| o.name).collect();
        names.sort_by_key(|n| n.to_lowercase());
        assert_eq!(ctl.label.as_deref(), Some(names[0]));
        assert_eq!(value, names[0]);
        assert_eq!(path, &K_OBJECT);
        assert!(!name.is_empty());
    }

    #[test]
    fn search_narrows_to_matching_objects_and_force_opens_ancestors() {
        let mut state = BState::new();
        state.set(&tree::k_expanded("scene_collection"), V_FALSE);
        state.set(&tree::k_search(), OBJECTS[0].name);
        let children = rows(&state);
        // Ancestors are force-opened despite the collapsed flag, and only the
        // matching object survives.
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn the_active_object_row_is_the_selected_row() {
        let mut state = BState::new();
        state.set(&K_OBJECT, OBJECTS[0].name);
        let children = rows(&state);
        let selected: Vec<_> = children
            .iter()
            .filter(|r| {
                matches!(
                    r,
                    BView::TreeRow {
                        shade: BRowShade::Selected,
                        ..
                    }
                )
            })
            .collect();
        assert_eq!(selected.len(), 1);
    }
}
