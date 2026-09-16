//! Outliner set — display modes, restriction columns, data-block icons.
//! Blender ref: `bl_ui/space_outliner.py`,
//! `editors/space_outliner/outliner_tree.cc` (SpaceOutliner display modes),
//! `editors/space_outliner/outliner_intern.hh` (TE_* icons per ID type).

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

// ── Display mode (SpaceOutliner.display_mode) ──────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BOutlineMode {
    Scenes,
    ViewLayer,
    Sequence,
    LibraryOverrides,
    /// `SO_LIBRARIES` — the "Blender File" display mode.
    Libraries,
    DataApi,
    OrphanData,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOutlineModeSpec {
    pub kind: BOutlineMode,
    pub label: &'static str,
}

impl BSpec for BOutlineModeSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

pub static MODES: &[BOutlineModeSpec] = &[
    BOutlineModeSpec {
        kind: BOutlineMode::ViewLayer,
        label: "View Layer",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::Scenes,
        label: "Scenes",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::Libraries,
        label: "Blender File",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::Sequence,
        label: "Video Sequencer",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::LibraryOverrides,
        label: "Library Overrides",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::DataApi,
        label: "Data API",
    },
    BOutlineModeSpec {
        kind: BOutlineMode::OrphanData,
        label: "Orphan Data",
    },
];

// ── Restriction columns (visibility toggles at the row's right side) ───────
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BOutlineToggle {
    Viewport,     // eye  — 👁 disable in viewports
    Render,       // cam  — disable in renders
    SelectLock,   // arrow — disable selection
    Hide,         // eye-slash — hide in viewport
    Holdout,      // holdout in view layer
    IndirectOnly, // indirect-only in view layer
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOutlineToggleSpec {
    pub kind: BOutlineToggle,
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub state_key: &'static str,
    /// Whether Blender shows this column without being asked.
    ///
    /// Only two are on by default — `show_restrict_column_hide` (the eye) and
    /// `show_restrict_column_render` (the camera). Selectability, disable-in-
    /// viewports, holdout and indirect-only are all opt-in from the Filter
    /// popover. Drawing all six made the row a wall of icons that looks
    /// nothing like Blender's.
    pub default_on: bool,
}

impl BSpec for BOutlineToggleSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.tooltip
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

/// Restriction columns in Blender's SCREEN order (left → right).
///
/// `outliner_draw_restrictbuts` lays these out right-to-left from the region
/// edge — Indirect Only, Holdout, Render, Viewport, Hide, Select — which puts
/// Select nearest the name and Indirect Only at the far right. Holdout and
/// Indirect Only exist only in View Layer mode (`view_layer_only`).
///
/// Icons are chosen from the shipped `assets/icons` set: there is no
/// `arrow`/`holdout`/`indirect` svg, so selectability uses `mouse_pointer`,
/// holdout uses `circle_dashed` and indirect-only uses `lightbulb`.
pub static TOGGLES: &[BOutlineToggleSpec] = &[
    BOutlineToggleSpec {
        kind: BOutlineToggle::SelectLock,
        icon: "mouse_pointer",
        tooltip: "Disable Selection",
        state_key: "hide_select",
        default_on: false,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Hide,
        icon: "eye",
        tooltip: "Hide in Viewport",
        state_key: "hide_get",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Viewport,
        icon: "monitor",
        tooltip: "Disable in Viewports",
        state_key: "hide_viewport",
        default_on: false,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Render,
        icon: "camera",
        tooltip: "Disable in Renders",
        state_key: "hide_render",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Holdout,
        icon: "circle_dashed",
        tooltip: "Holdout",
        state_key: "holdout",
        default_on: false,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::IndirectOnly,
        icon: "lightbulb",
        tooltip: "Indirect Only",
        state_key: "indirect_only",
        default_on: false,
    },
];

impl BOutlineToggle {
    /// Holdout and Indirect Only are View-Layer-only columns.
    pub fn view_layer_only(self) -> bool {
        matches!(self, BOutlineToggle::Holdout | BOutlineToggle::IndirectOnly)
    }
}

impl BOutlineMode {
    pub fn slug(self) -> &'static str {
        match self {
            BOutlineMode::ViewLayer => "ViewLayer",
            BOutlineMode::Scenes => "Scenes",
            BOutlineMode::Sequence => "Sequence",
            BOutlineMode::LibraryOverrides => "LibraryOverrides",
            BOutlineMode::Libraries => "Libraries",
            BOutlineMode::DataApi => "DataApi",
            BOutlineMode::OrphanData => "OrphanData",
        }
    }
    /// Accepts either the compact slug (`"ViewLayer"`) or the UI label
    /// (`"View Layer"`) — the modebar dropdown writes labels.
    pub fn from_slug(s: &str) -> Option<Self> {
        MODES
            .iter()
            .find(|m| m.kind.slug() == s || m.label == s)
            .map(|m| m.kind)
    }
    /// Blender shows the restriction columns only in View Layer and Scenes;
    /// Data API, the Sequencer and Blender File draw none.
    pub fn shows_restrictions(self) -> bool {
        matches!(self, BOutlineMode::ViewLayer | BOutlineMode::Scenes)
    }
}

// ── Row type icons per Blender ID-type (partial; the common ones) ──────────
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BOutlineRowType {
    SceneCollection,
    Collection,
    Object,
    Mesh,
    Curve,
    Surface,
    Metaball,
    Text,
    Lattice,
    GreasePencil,
    Camera,
    Light,
    LightProbe,
    Speaker,
    Armature,
    Bone,
    Material,
    Texture,
    Image,
    Shader,
    Modifier,
    Constraint,
    ParticleSystem,
    Action,
    Nla,
    Driver,
    ShapeKey,
    World,
    WorkSpace,
    Library,
    Node,
    Brush,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOutlineRowSpec {
    pub kind: BOutlineRowType,
    pub icon: &'static str,
    pub label: &'static str,
}

impl BSpec for BOutlineRowSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

const fn r(kind: BOutlineRowType, icon: &'static str, label: &'static str) -> BOutlineRowSpec {
    BOutlineRowSpec { kind, icon, label }
}

pub static ROW_TYPES: &[BOutlineRowSpec] = &[
    r(
        BOutlineRowType::SceneCollection,
        "scene-collection",
        "Scene Collection",
    ),
    r(BOutlineRowType::Collection, "collection", "Collection"),
    r(BOutlineRowType::Object, "object", "Object"),
    r(BOutlineRowType::Mesh, "mesh", "Mesh"),
    r(BOutlineRowType::Curve, "curve", "Curve"),
    r(BOutlineRowType::Surface, "surface", "Surface"),
    r(BOutlineRowType::Metaball, "metaball", "Metaball"),
    r(BOutlineRowType::Text, "text", "Text"),
    r(BOutlineRowType::Lattice, "lattice", "Lattice"),
    r(
        BOutlineRowType::GreasePencil,
        "grease-pencil",
        "Grease Pencil",
    ),
    r(BOutlineRowType::Camera, "camera", "Camera"),
    r(BOutlineRowType::Light, "light", "Light"),
    r(BOutlineRowType::LightProbe, "light-probe", "Light Probe"),
    r(BOutlineRowType::Speaker, "speaker", "Speaker"),
    r(BOutlineRowType::Armature, "armature", "Armature"),
    r(BOutlineRowType::Bone, "bone", "Bone"),
    r(BOutlineRowType::Material, "material", "Material"),
    r(BOutlineRowType::Texture, "texture", "Texture"),
    r(BOutlineRowType::Image, "image", "Image"),
    r(BOutlineRowType::Shader, "shader", "Shader"),
    r(BOutlineRowType::Modifier, "modifier", "Modifier"),
    r(BOutlineRowType::Constraint, "constraint", "Constraint"),
    r(
        BOutlineRowType::ParticleSystem,
        "particles",
        "Particle System",
    ),
    r(BOutlineRowType::Action, "action", "Action"),
    r(BOutlineRowType::Nla, "nla", "NLA Strip"),
    r(BOutlineRowType::Driver, "driver", "Driver"),
    r(BOutlineRowType::ShapeKey, "shape-key", "Shape Key"),
    r(BOutlineRowType::World, "world", "World"),
    r(BOutlineRowType::WorkSpace, "workspace", "Workspace"),
    r(BOutlineRowType::Library, "library", "Library"),
    r(BOutlineRowType::Node, "node", "Node"),
    r(BOutlineRowType::Brush, "brush", "Brush"),
];

// ── Header controls (bl_ui OUTLINER_HT_header) ─────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOutlineFilterSpec {
    pub icon: &'static str,
    pub label: &'static str,
    pub key: &'static str,
}

pub static FILTERS: &[BOutlineFilterSpec] = &[
    BOutlineFilterSpec {
        icon: "filter",
        label: "Search",
        key: "filter_text",
    },
    BOutlineFilterSpec {
        icon: "case_sensitive",
        label: "Case Sensitive",
        key: "use_filter_case_sensitive",
    },
    BOutlineFilterSpec {
        icon: "whole_word",
        label: "Exact Match",
        key: "use_filter_complete",
    },
    BOutlineFilterSpec {
        icon: "collection",
        label: "Collections",
        key: "use_filter_collection",
    },
    BOutlineFilterSpec {
        icon: "object",
        label: "Objects",
        key: "use_filter_object",
    },
    BOutlineFilterSpec {
        icon: "git_branch",
        label: "Object Children",
        key: "use_filter_children",
    },
];
