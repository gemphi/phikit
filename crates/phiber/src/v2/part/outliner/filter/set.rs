//! The Filter popover's boxes — `OUTLINER_PT_filter`.
//!
//! Blender ref: `bl_ui/space_outliner.py` (`OUTLINER_PT_filter.draw`) and
//! the `SpaceOutliner` properties in `makesrna/intern/rna_space.cc`.
//!
//! Each box is one RNA name under `ctx.outliner.filter`, its label as
//! Blender writes it, and the value Blender ships with — `default_on`, so a
//! box nobody has touched answers the way a fresh Blender does rather than
//! `false`. That distinction matters here: the four Filter boxes and every
//! type box are on in Blender, and defaulting them off empties the tree.
//!
//! The groups are the popover's own order: Search, then Filter, then the
//! object-state dropdown, then Types.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOutlineFilterSpec {
    pub icon: &'static str,
    pub label: &'static str,
    /// The RNA name, which is also the key under `ctx.outliner.filter`.
    pub key: &'static str,
    /// What Blender ships with, used until the box is ticked either way.
    pub default_on: bool,
}

/// `filter_text`'s two modifiers — the popover's "Search" group.
pub static SEARCH: [BOutlineFilterSpec; 2] = [CASE, EXACT];

/// `use_filter_case_sensitive`.
pub const CASE: BOutlineFilterSpec = BOutlineFilterSpec {
    icon: "case_sensitive",
    label: "Case Sensitive",
    key: "use_filter_case_sensitive",
    default_on: false,
};

/// `use_filter_complete` — match the whole name, not a substring.
pub const EXACT: BOutlineFilterSpec = BOutlineFilterSpec {
    icon: "whole_word",
    label: "Exact Match",
    key: "use_filter_complete",
    default_on: false,
};

/// The popover's "Filter" group, in Blender's order. All four ship on: the
/// Outliner lists everything until it is asked not to.
pub static FILTERS: [BOutlineFilterSpec; 4] = [COLLECTIONS, OBJECTS, CONTENTS, CHILDREN];

/// `use_filter_collection`.
pub const COLLECTIONS: BOutlineFilterSpec = BOutlineFilterSpec {
    icon: "collection",
    label: "Collections",
    key: "use_filter_collection",
    default_on: true,
};

/// `use_filter_object`.
pub const OBJECTS: BOutlineFilterSpec = BOutlineFilterSpec {
    icon: "object",
    label: "Objects",
    key: "use_filter_object",
    default_on: true,
};

/// `use_filter_object_content` — an object's data, modifiers and material
/// slots, listed under it.
pub const CONTENTS: BOutlineFilterSpec = BOutlineFilterSpec {
    icon: "mesh_data",
    label: "Object Contents",
    key: "use_filter_object_content",
    default_on: true,
};

/// `use_filter_children` — Blender names this one without the `object`
/// infix that the type boxes carry.
pub const CHILDREN: BOutlineFilterSpec = BOutlineFilterSpec {
    icon: "git_branch",
    label: "Object Children",
    key: "use_filter_children",
    default_on: true,
};

/// `filter_state`: which objects the tree lists at all.
pub const STATE_KEY: &str = "filter_state";

/// `filter_state`'s items. The first is the default — everything — and the
/// two the tree reads specially (`VISIBLE`, `SELECTABLE`) narrow it by what
/// the view layer allows rather than by what is selected.
pub static STATES: [(&str, &str); 5] = [
    ("ALL", "All"),
    ("VISIBLE", "Visible"),
    ("SELECTED", "Selected"),
    ("ACTIVE", "Active"),
    ("SELECTABLE", "Selectable"),
];

/// `filter_invert` — the state above, inverted. Blender only offers it
/// once a state other than All is picked, and so does the popover.
pub const INVERT: BOutlineFilterSpec = BOutlineFilterSpec {
    icon: "arrow_swap",
    label: "Invert",
    key: "filter_invert",
    default_on: false,
};

/// The object-type boxes, each with the `object_type` names it admits.
///
/// Others is last and names nothing: it answers for every type no other box
/// claims, which is also how `file_has` decides whether to list it at all.
pub static TYPES: [(BOutlineFilterSpec, &[&str]); 7] = [
    (
        BOutlineFilterSpec { icon: "mesh_data", label: "Meshes", key: "use_filter_object_mesh", default_on: true },
        &["MESH", "SURFACE", "CURVE", "CURVES", "FONT", "META", "POINTCLOUD", "VOLUME"],
    ),
    (
        BOutlineFilterSpec { icon: "armature", label: "Armatures", key: "use_filter_object_armature", default_on: true },
        &["ARMATURE"],
    ),
    (
        BOutlineFilterSpec { icon: "empty", label: "Empties", key: "use_filter_object_empty", default_on: true },
        &["EMPTY"],
    ),
    (
        BOutlineFilterSpec { icon: "camera", label: "Cameras", key: "use_filter_object_camera", default_on: true },
        &["CAMERA"],
    ),
    (
        BOutlineFilterSpec { icon: "light", label: "Lights", key: "use_filter_object_light", default_on: true },
        &["LIGHT", "LIGHT_PROBE"],
    ),
    (
        BOutlineFilterSpec { icon: "grease_pencil", label: "Grease Pencil", key: "use_filter_object_grease_pencil", default_on: true },
        &["GPENCIL", "GREASEPENCIL"],
    ),
    (
        BOutlineFilterSpec { icon: "object", label: "Others", key: "use_filter_object_others", default_on: true },
        &[],
    ),
];
