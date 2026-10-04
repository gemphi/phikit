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


use crate::v2::scene::{BObject3D, scene_objects};
use crate::v2::state::BState;

/// Generic icon for a loaded (imported) object, which carries no primitive
/// `BShape`.
pub const LOADED_OBJECT_ICON: &str = "mesh";

/// Intern a runtime object name to a `&'static str` so it can be a `BPath`
/// segment.
///
/// The pool itself moved to `state::intern` once the animation channels and the
/// node editors' cards needed the same thing — a second pool would leak the
/// same name twice. This stays as the Outliner's own spelling of it.
pub fn intern_name(name: &str) -> &'static str {
    crate::v2::state::intern(name)
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
    /// What a collapsed row holds, one entry per kind with its count — the
    /// icon row after a closed item's name (`outliner_draw_iconrow`).
    pub summary: Vec<(&'static str, usize)>,
}

/// `outliner_draw_iconrow`'s merge: the kinds in first-seen order, counted.
pub fn summarise<'a>(kinds: impl IntoIterator<Item = &'a str>) -> Vec<(&'static str, usize)> {
    let mut out: Vec<(&'static str, usize)> = Vec::new();
    for kind in kinds {
        match out.iter_mut().find(|(k, _)| *k == kind) {
            Some((_, n)) => *n += 1,
            None => out.push((intern_name(kind), 1)),
        }
    }
    out
}

pub use super::key::*;

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
    let filter = super::filter::BOutlineFilter::now(s);
    let filtering = filter.searching();

    // A loaded `.blend` brings its own collection tree (`layer.rs`); the
    // flat list below is only the Default_Scene / `.phx` fallback.
    if let Some(rows) = crate::v2::scene::source::with_loaded(|m| {
        m.and_then(|m| super::layer::rows(s, m, &filter))
    }) {
        return rows;
    }

    let mut objects: Vec<(String, &'static str)> = scene_objects(s)
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
            let hit = filter.hits(&name);
            if (filtering && !hit) || !filter.shows(s, &name, "MESH") {
                None
            } else {
                Some((name, icon, filtering && hit))
            }
        })
        .collect();

    // A search force-opens the ancestors of every match.
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
        summary: if scene_open { Vec::new() } else { summarise(["collection"]) },
    });
    if !scene_open {
        return rows;
    }

    // Collections off: the objects hang straight off the Scene Collection.
    let depth = if filter.collections { 2 } else { 1 };
    rows.extend(filter.collections.then(|| BOutlineRow {
        id: "collection",
        kind: BRowKind::Collection,
        depth: 1,
        icon: "collection",
        label: "Collection".to_string(),
        expanded: Some(coll_open),
        object: None,
        matched: false,
        summary: if coll_open { Vec::new() } else { summarise(visible.iter().map(|(_, icon, _)| *icon)) },
    }));
    if filter.collections && !coll_open {
        return rows;
    }

    for (name, icon, matched) in visible {
        let id = intern_name(&name);
        // Objects start collapsed, like Blender's outliner.
        let open = filter.contents && is_expanded_or(s, id, false) && !filtering;
        rows.push(BOutlineRow {
            id,
            kind: BRowKind::Object,
            depth,
            icon,
            label: name.clone(),
            expanded: filter.contents.then_some(open),
            object: Some(name.clone()),
            matched,
            summary: if open || !filter.contents { Vec::new() } else { summarise(["mesh"]) },
        });
        if open {
            // The object row opens onto its data block, which opens onto
            // what it holds — vertex groups, UV maps, materials.
            let data_id = intern_name(&format!("{name}/data"));
            let data_open = is_expanded_or(s, data_id, false) && !filtering;
            rows.push(BOutlineRow {
                id: data_id,
                kind: BRowKind::Data,
                depth: depth + 1,
                icon: "mesh",
                label: name.clone(),
                expanded: Some(data_open),
                object: None,
                matched: false,
                summary: if data_open { Vec::new() } else { summarise(DATA_BLOCKS.iter().map(|(_, icon, _)| *icon)) },
            });
            if data_open {
                for (suffix, icon, label) in DATA_BLOCKS {
                    rows.push(BOutlineRow {
                        id: intern_name(&format!("{name}/data/{suffix}")),
                        kind: BRowKind::Data,
                        depth: depth + 2,
                        icon,
                        label: (*label).to_string(),
                        expanded: None,
                        object: None,
                        matched: false,
                        summary: Vec::new(),
                    });
                }
            }
        }
    }
    rows
}

/// What a mesh data block holds, in Blender's outliner order
/// (`space_outliner/tree/tree_element_id_mesh.cc`).
const DATA_BLOCKS: &[(&str, &str, &str)] = &[
    ("vertex_groups", "vertex_groups", "Vertex Groups"),
    ("uv_layers", "uv_layers", "UV Maps"),
    ("material", "material", "Material"),
];
