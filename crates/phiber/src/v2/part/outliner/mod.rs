//! Outliner center content — the scene collection tree.
//! Blender ref: `space_outliner.cc` WINDOW region draw.

pub mod drop;
pub mod key;
pub mod layer;
pub mod object_rows;
pub mod op;
pub mod reveal;
pub mod filter;
pub mod mode;
pub mod row_type;
pub mod set;
pub mod toggle;
pub mod tree;
pub mod type_set;

use crate::v2::state::BKey as _;
use crate::v2::state::CtxKey;
use crate::v2::state::OutlinerKey;
use crate::v2::state::BState;
use crate::v2::state::val::V_FALSE;
use crate::v2::state::val::V_TRUE;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use crate::v2::view::prim::BRowShade;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use self::set::TOGGLES;
use crate::v2::view::BViewList;
pub use self::tree::BOutlineRow;
pub use self::tree::BRowKind;

// Outliner scene tree — Blender's View Layer rows, backed by the runtime
// Scene_Source and BState.
// 
// Rows come from [`tree::flatten`], which reads [`crate::v2::scene::scene_source`]
// — the same single Scene_Source the Viewport reads — so the Outliner never
// keeps an independent object list (Requirement 5.2). One row per object in
// enumeration order (5.1); a zero-object scene emits zero object rows (5.3).
// Selection is shared with the Viewport via `CtxKey::Object.path()` (5.4, 5.5).
// 
// Row geometry matches `outliner_draw_tree_element`: indent by depth, a
// reserved disclosure slot, the data-type icon, the name, then a flexible
// gap and the right-aligned restriction column. Alternating bands come from
// the palette's `row_alt`; the active object gets the selected fill.
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
        Some(BAction::Set(OutlinerKey::Expanded(row.id).path(), next.to_string()))
    }

    /// One restriction toggle. Blender draws these as icon-only buttons in a
    /// fixed column at the row's right edge; the flag lives per object.
    fn restriction(s: &BState, object: &str, field: &'static str, icon: &'static str) -> BView {
        let key = self::tree::k_restrict(object, field);
        let on = s.get(&key) == Some(V_TRUE);
        let next = if on { V_FALSE } else { V_TRUE };
        // `eye` / `eye_off` is the one pair where the icon itself carries the
        // state. The rest were said to "dim via the button's own disabled-ish
        // styling", and nothing set `muted`, so nothing dimmed: nine toggles
        // that read their state and then rendered identically on and off.
        //
        // On takes the fill, off takes the quiet ink — which is how Blender
        // draws a restriction column, and what `muted` is for.
        let shown = if field == "hide_get" && on {
            "eye_off"
        } else {
            icon
        };
        BView::Button(
            BControl::new()
                .with_icon(shown)
                .with_active(on)
                .with_muted(!on)
                .with_action(BAction::Set(key, next.to_string())),
        )
    }

    /// The restriction cluster for an object row. `tree_row` pins this to the
    /// row's right edge and stops it shrinking, so the toggles stay in a
    /// clean column however long the object names get.
    ///
    /// The whole cluster is shown by default (`BOutlineToggleSpec::default_on`,
    /// a recorded divergence from Blender's eye-and-camera-only default); a
    /// column is dropped only when the Filter popover turns it off
    /// (`ctx.outliner.col.<flag>` = false), and the two View-Layer-only columns
    /// appear only in View Layer mode.
    fn restrictions(s: &BState, object: &str, view_layer: bool) -> Vec<BView> {
        let mut out = Vec::new();
        for t in TOGGLES {
            if t.kind.view_layer_only() && !view_layer {
                continue;
            }
            let shown = match s.get(&OutlinerKey::Column(t.state_key).path()) {
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
        selection: &crate::v2::scene::select::BSelection,
        restrictions: bool,
    ) -> BView {
        let selected = row.object.as_deref().is_some_and(|name| selection.contains(name));
        let active = row.object.as_deref().is_some_and(|name| Some(name) == s.get(CtxKey::Object)) && matches!(row.kind, BRowKind::Object);
        let shade = if selected && active {
            BRowShade::Active
        } else if selected {
            BRowShade::Selected
        } else if index % 2 == 1 {
            BRowShade::Alt
        } else {
            BRowShade::Flat
        };

        // The data-type icon is its own grid column, owned by `tree_row`;
        // only the name (and the search-hit marker) live in the main cells.
        let mut cell_list = Vec::new();
        match (&row.object, row.kind) {
            // Object rows are clickable — they set the active object.
            (Some(name), BRowKind::Object) => {
                cell_list.push(BView::Button(
                    BControl::new()
                        .with_label(row.label.clone())
                        .with_action(BAction::Set(CtxKey::Object.path(), name.clone())),
                ));
            }
            _ => cell_list.push(BView::label(row.label.clone())),
        }
        if row.matched {
            cell_list.push(BView::icon("search"));
        }
        // A closed row shows what it holds after its name, a glyph per kind
        // with its count (`outliner_draw_iconrow`).
        for (icon, count) in &row.summary {
            cell_list.push(BView::Badge { icon: (*icon).to_string(), count: *count });
        }

        let trail = match (&row.object, restrictions) {
            (Some(name), true) => Self::restrictions(s, name, true),
            _ => Vec::new(),
        };
        // A collection row leads with its checkbox — checked when the
        // collection is in the view layer (`LAYER_COLLECTION_EXCLUDE` off).
        let lead = match (&row.object, row.kind) {
            (Some(id), BRowKind::Collection) => vec![Self::exclude_box(s, id)],
            _ => Vec::new(),
        };

        let row_view = BView::tree_row(row.depth, row.expanded, Self::toggle_action(row), row.icon, cell_list, trail, shade).with_lead(lead);
        // Objects drag onto collections (move) and onto objects (parent);
        // collections take drops and nest (`drop::BOutlinerDropRule`).
        let payload = match (&row.object, row.kind) {
            (Some(name), BRowKind::Object) => Some(format!("ob:{name}")),
            (Some(id), BRowKind::Collection | BRowKind::SceneCollection) => id.strip_prefix("col/").map(|path| format!("col:{path}")),
            _ => None,
        };
        match payload {
            Some(payload) => row_view.with_drag(crate::v2::view::prim::BTreeDrag { payload, drop: crate::v2::state::OpKey::OutlinerDrop.path(), label: row.label.clone() }),
            None => row_view,
        }
    }

    /// The exclude checkbox: on means included, as Blender draws it.
    fn exclude_box(s: &BState, id: &str) -> BView {
        let key = self::tree::k_restrict(id, "exclude");
        let excluded = s.get(&key) == Some(V_TRUE);
        let next = if excluded { V_FALSE } else { V_TRUE };
        // Ticked AND lit when included, as every other checkable row in the
        // app is — the glyph says which one when you are reading the column,
        // the fill says it when you are not (`on_state_budget`).
        BView::Button(
            BControl::new()
                .with_icon(if excluded { "square" } else { "check_square" })
                .with_active(!excluded)
                .with_tooltip("Exclude from View Layer")
                .with_action(BAction::Set(key, next.to_string())),
        )
    }
}

thread_local! {
    /// The list as last built, with what it was built from: the writes to
    /// every key family but the viewport's (`BState::scene_revision`) and
    /// the loaded model. An orbit changes neither, and building five hundred
    /// rows was most of a debug frame's tree.
    static BUILT: std::cell::RefCell<Option<(u64, usize, std::rc::Rc<BView>)>> = const { std::cell::RefCell::new(None) };
}

impl BOutlinerItem {
    /// What the list is built from, as one pair.
    fn built_from(s: &BState) -> (u64, usize) {
        let model = crate::v2::scene::source::loaded_shared().map_or(0, |m| std::sync::Arc::as_ptr(&m) as usize);
        (s.scene_revision(), model)
    }

    fn build_rows(s: &BState) -> BView {
        let selection = crate::v2::scene::select::BSelection::now(s);
        let restrictions = self::tree::display_mode(s).shows_restrictions();
        let rows = self::tree::flatten(s);
        let views = rows
            .iter()
            .enumerate()
            .map(|(i, r)| Self::row_view(s, r, i, &selection, restrictions))
            .collect();
        BView::column(views)
    }
}

impl BViewItem for BOutlinerItem {
    fn build(&self, s: &BState) -> BView {
        let (rev, model) = Self::built_from(s);
        if let Some(view) = BUILT.with(|b| b.borrow().as_ref().filter(|(r, m, _)| (*r, *m) == (rev, model)).map(|(_, _, v)| std::rc::Rc::clone(v))) {
            return BView::Shared(view);
        }
        let view = std::rc::Rc::new(Self::build_rows(s));
        BUILT.with(|b| *b.borrow_mut() = Some((rev, model, std::rc::Rc::clone(&view))));
        BView::Shared(view)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::OBJECTS;

    fn rows(state: &BState) -> Vec<BView> {
        let BView::Column { list } = BOutlinerItem::new().build(state).shown().clone() else {
            panic!()
        };
        list
    }

    /// Built once for every frame of an orbit: the viewport's keys are not
    /// the list's; a pick is.
    #[test]
    fn the_rows_are_built_again_only_when_what_they_read_changed() {
        let shared = |v: BView| match v {
            BView::Shared(rc) => rc,
            _ => panic!("the list is shared"),
        };
        let mut state = BState::new();
        let first = shared(BOutlinerItem::new().build(&state));
        state.set(&crate::v2::scene::yaw_key(), "33");
        state.set(&crate::v2::scene::zoom_key(), "1.5");
        assert!(std::rc::Rc::ptr_eq(&first, &shared(BOutlinerItem::new().build(&state))), "an orbit: the same rows");
        state.set(CtxKey::Frame, "17");
        state.set(CtxKey::Playing, "true");
        state.set(CtxKey::KeySelected, "17");
        assert!(std::rc::Rc::ptr_eq(&first, &shared(BOutlinerItem::new().build(&state))), "a frame played, a key picked: the same rows");
        state.set(CtxKey::Object, "Cube");
        let picked = shared(BOutlinerItem::new().build(&state));
        assert!(!std::rc::Rc::ptr_eq(&first, &picked), "a pick: built again");
        assert!(std::rc::Rc::ptr_eq(&picked, &shared(BOutlinerItem::new().build(&state))));
    }

    #[test]
    fn default_scene_emits_one_row_per_object_under_two_group_rows() {
        let state = BState::new();
        let list = rows(&state);
        // Scene Collection + Collection + one row per Default_Scene object.
        assert_eq!(list.len(), 2 + OBJECTS.len());
        for row in &list[2..] {
            assert!(matches!(row, BView::TreeRow { .. }));
        }
    }

    #[test]
    fn a_closed_row_shows_what_it_holds_as_badges_with_counts() {
        let mut state = BState::new();
        state.set(&OutlinerKey::Expanded("collection").path(), V_FALSE);
        let list = rows(&state);
        let BView::TreeRow { list: cells, .. } = &list[1] else { panic!("the Collection row") };
        let badges: Vec<(String, usize)> = cells
            .iter()
            .filter_map(|v| match v {
                BView::Badge { icon, count } => Some((icon.clone(), *count)),
                _ => None,
            })
            .collect();
        assert_eq!(badges.iter().map(|(_, n)| n).sum::<usize>(), OBJECTS.len(), "every object under it, counted by kind: {badges:?}");
        // An open row carries none; a closed object row shows its mesh.
        let open = rows(&BState::new());
        let BView::TreeRow { list: cells, .. } = &open[1] else { panic!() };
        assert!(!cells.iter().any(|v| matches!(v, BView::Badge { .. })));
        let BView::TreeRow { list: cells, .. } = &open[2] else { panic!() };
        assert!(matches!(cells.last(), Some(BView::Badge { icon, count: 1 }) if icon == "mesh"), "a closed object holds its mesh: {cells:?}");
        assert_eq!(super::tree::summarise(["mesh", "light", "mesh", "camera", "mesh"]), [("mesh", 3), ("light", 1), ("camera", 1)]);
    }

    #[test]
    fn group_rows_are_open_by_default_and_collapse_on_first_click() {
        let state = BState::new();
        let list = rows(&state);
        let BView::TreeRow {
            depth,
            expanded,
            toggle,
            ..
        } = &list[0]
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
        let list = rows(&state);
        let depths: Vec<u16> = list
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
        state.set(&OutlinerKey::Expanded("scene_collection").path(), V_FALSE);
        assert_eq!(rows(&state).len(), 1);
    }

    #[test]
    fn object_row_uses_shape_icon_and_sets_the_active_object() {
        let state = BState::new();
        let list = rows(&state);
        // The data-type icon is a grid column on the row itself, not a cell,
        // so the first main cell is the clickable name.
        let BView::TreeRow {
            list: cell_list,
            icon: name,
            ..
        } = &list[2]
        else {
            panic!()
        };
        let BView::Button(ctl) = &cell_list[0] else {
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
        assert_eq!(*path, CtxKey::Object.path());
        assert!(!name.is_empty());
    }

    #[test]
    fn search_narrows_to_matching_objects_and_force_opens_ancestors() {
        let mut state = BState::new();
        state.set(&OutlinerKey::Expanded("scene_collection").path(), V_FALSE);
        state.set(&OutlinerKey::Search.path(), OBJECTS[0].name);
        let list = rows(&state);
        // Ancestors are force-opened despite the collapsed flag, and only the
        // matching object survives.
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn the_active_object_row_is_the_selected_row() {
        let mut state = BState::new();
        state.set(CtxKey::Object, OBJECTS[0].name);
        let list = rows(&state);
        // The active object is the selection until one is made, and its
        // row takes the active tone, a step over the selected one.
        let lit: Vec<_> = list
            .iter()
            .filter_map(|r| match r {
                BView::TreeRow { shade: shade @ (BRowShade::Selected | BRowShade::Active), .. } => Some(*shade),
                _ => None,
            })
            .collect();
        assert_eq!(lit, [BRowShade::Active]);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BOutlinerList;

impl BOutlinerList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BOutlinerList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BOutlinerItem::new().build(s)]
    }
}
