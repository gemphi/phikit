//! Outliner tree model — builds Blender's View Layer hierarchy from the
//! render-pass Scene_Source and flattens it into drawable rows.
//!
//! Blender ref: `outliner_tree.cc` (`outliner_build_tree`) and
//! `tree/tree_display_view_layer.cc`. The hierarchy is
//!
//! ```text
//! Scene Collection
//!   └ Collection
//!       └ Object
//!           └ Object Data (mesh)
//! ```
//!
//! Search is *recursive flagging*, not filtering: a matching row keeps its
//! ancestors and force-opens them (`TSE_CHILDSEARCH` / `TSELEM_OPEN`), so the
//! tree never loses its shape while you type. Sorting is natural,
//! case-insensitive and ON by default — Blender's flag is the inverted
//! `SO_SKIP_SORT_ALPHA`.

use std::collections::HashMap;
use std::sync::Mutex;

use super::set::BOutlineMode;
use crate::v2::scene::{BObject3D, scene_source};
use crate::v2::state::{
    BPath, BState,
    val::{V_FALSE, V_TRUE},
};

/// Generic icon for a loaded (imported) object, which carries no primitive
/// `BShape`.
pub const LOADED_OBJECT_ICON: &str = "mesh";

/// Intern a runtime object name to a `&'static str` so it can be a `BPath`
/// segment. A plain `Box::leak` per frame would leak every frame; interning
/// leaks each distinct name at most once, so the leaked set is bounded by the
/// number of distinct object names ever shown.
pub fn intern_name(name: &str) -> &'static str {
    static POOL: Mutex<Option<HashMap<String, &'static str>>> = Mutex::new(None);
    let mut guard = POOL.lock().unwrap();
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(&s) = map.get(name) {
        return s;
    }
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    map.insert(name.to_string(), leaked);
    leaked
}

/// What a row represents — decides which columns and actions it gets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BRowKind {
    /// The scene's master collection.
    SceneCollection,
    /// A nested collection.
    Collection,
    /// An object (the only kind carrying restriction toggles + selection).
    Object,
    /// An object's data-block (mesh).
    Data,
}

/// One flattened, drawable row.
#[derive(Clone, Debug)]
pub struct BOutlineRow {
    /// Stable id used for the expansion key.
    pub id: &'static str,
    pub kind: BRowKind,
    pub depth: u16,
    pub icon: &'static str,
    pub label: String,
    /// `None` when the row is a leaf — the disclosure slot stays reserved but
    /// blank so names remain column-aligned (Blender reserves it too).
    pub expanded: Option<bool>,
    /// Object name for selection and restriction keys.
    pub object: Option<String>,
    /// The row itself matched the search text.
    pub matched: bool,
}

// ── state keys ─────────────────────────────────────────────────────────────

pub fn k_expanded(id: &'static str) -> BPath {
    BPath::owned(vec!["outliner", "expanded", id])
}
pub fn k_search() -> BPath {
    BPath::owned(vec!["outliner", "search"])
}
pub fn k_display_mode() -> BPath {
    BPath::owned(vec!["outliner", "display_mode"])
}
pub fn k_sort_alpha() -> BPath {
    BPath::owned(vec!["outliner", "sort_alpha"])
}
/// Per-object restriction flag, e.g. `ctx.outliner.Cube.hide_render`.
pub fn k_restrict(object: &str, field: &'static str) -> BPath {
    BPath::owned(vec!["outliner", intern_name(object), field])
}

/// Whether a restriction COLUMN is shown, e.g. `ctx.outliner.col.hide_get`.
/// Blender's `show_restrict_column_*` flags, set from the Filter popover.
pub fn k_column(field: &'static str) -> BPath {
    BPath::owned(vec!["outliner", "col", field])
}

/// Open/closed state of the Outliner filter popover (OUTLINER_PT_filter).
pub fn k_filter_open() -> BPath {
    BPath::owned(vec!["outliner", "filter_open"])
}

/// Is `object` hidden in the viewport? Read by the 3D viewport, which is what
/// turns the eye icon from decoration into a working control.
pub fn is_hidden(s: &BState, object: &str) -> bool {
    s.get(&k_restrict(object, "hide_get")) == Some(V_TRUE)
        || s.get(&k_restrict(object, "hide_viewport")) == Some(V_TRUE)
}

/// Expansion state with a per-row default.
///
/// Blender opens the Scene Collection and its collections but leaves objects
/// closed, so their data-blocks (mesh, materials, modifiers) stay out of the
/// way until asked for. A single "default open" rule would show a Mesh row
/// under every object on first paint.
pub fn is_expanded_or(s: &BState, id: &'static str, default_open: bool) -> bool {
    match s.get(&k_expanded(id)) {
        Some(V_FALSE) => false,
        Some(V_TRUE) => true,
        _ => default_open,
    }
}

/// Rows default to expanded; only an explicit `"false"` collapses one.
pub fn is_expanded(s: &BState, id: &'static str) -> bool {
    is_expanded_or(s, id, true)
}

pub fn search_text(s: &BState) -> String {
    s.get(&k_search()).unwrap_or("").trim().to_lowercase()
}

pub fn display_mode(s: &BState) -> BOutlineMode {
    s.get(&k_display_mode())
        .and_then(BOutlineMode::from_slug)
        .unwrap_or(BOutlineMode::ViewLayer)
}

/// Blender sorts alphabetically by default; the UI flag is the inverted
/// `SO_SKIP_SORT_ALPHA`, so only an explicit `"false"` turns sorting off.
pub fn sort_alpha(s: &BState) -> bool {
    s.get(&k_sort_alpha()) != Some(V_FALSE)
}

// ── build ──────────────────────────────────────────────────────────────────

fn object_icon_and_name(object: &BObject3D) -> (&'static str, String) {
    match object {
        BObject3D::Primitive { name, shape, .. } => (shape.icon(), name.clone()),
        BObject3D::Loaded { name, .. } => (LOADED_OBJECT_ICON, name.clone()),
    }
}

/// Case-insensitive natural-ish ordering: compare lowercased, which is what
/// `BLI_strcasecmp_natural` gives for the names we produce.
fn sort_key(name: &str) -> String {
    name.to_lowercase()
}

/// Flatten the View Layer hierarchy into drawable rows.
pub fn flatten(s: &BState) -> Vec<BOutlineRow> {
    let query = search_text(s);
    let filtering = !query.is_empty();

    let mut objects: Vec<(String, &'static str)> = scene_source(s)
        .iter()
        .map(|o| {
            let (icon, name) = object_icon_and_name(o);
            (name, icon)
        })
        .collect();
    if sort_alpha(s) {
        objects.sort_by(|a, b| sort_key(&a.0).cmp(&sort_key(&b.0)));
    }

    let visible: Vec<(String, &'static str, bool)> = objects
        .into_iter()
        .filter_map(|(name, icon)| {
            let hit = name.to_lowercase().contains(&query);
            if filtering && !hit {
                None
            } else {
                Some((name, icon, filtering && hit))
            }
        })
        .collect();

    // A search force-opens the ancestors of every match so the hits are
    // reachable without the user expanding anything.
    let scene_open = filtering || is_expanded(s, "scene_collection");
    let coll_open = filtering || is_expanded(s, "collection");

    let mut rows = Vec::with_capacity(visible.len() * 2 + 2);
    rows.push(BOutlineRow {
        id: "scene_collection",
        kind: BRowKind::SceneCollection,
        depth: 0,
        icon: "scene",
        label: "Scene Collection".to_string(),
        expanded: Some(scene_open),
        object: None,
        matched: false,
    });
    if !scene_open {
        return rows;
    }

    rows.push(BOutlineRow {
        id: "collection",
        kind: BRowKind::Collection,
        depth: 1,
        icon: "collection",
        label: "Collection".to_string(),
        expanded: Some(coll_open),
        object: None,
        matched: false,
    });
    if !coll_open {
        return rows;
    }

    for (name, icon, matched) in visible {
        let id = intern_name(&name);
        // Objects start collapsed, like Blender's outliner.
        let open = is_expanded_or(s, id, false) && !filtering;
        rows.push(BOutlineRow {
            id,
            kind: BRowKind::Object,
            depth: 2,
            icon,
            label: name.clone(),
            expanded: Some(open),
            object: Some(name.clone()),
            matched,
        });
        if open {
            let data_id = intern_name(&format!("{name}/data"));
            rows.push(BOutlineRow {
                id: data_id,
                kind: BRowKind::Data,
                depth: 3,
                icon: "mesh",
                label: name,
                expanded: None,
                object: None,
                matched: false,
            });
        }
    }
    rows
}
