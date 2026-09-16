//! `BView` — declarative view tree. Seven store-level variants plus primitives.
//! Primitives (`Row`/`Column`/`Panel`/`Icon`/`Label`/`Button`) legal ONLY
//! inside the `body` of a `Part` node.

pub mod prim;
pub mod render;

use crate::v2::area::BAreaType;
use crate::v2::host::BHostType;
use crate::v2::pane::BPaneType;
use crate::v2::part::BPartType;
use crate::v2::region::BRegionType;
use crate::v2::space::BSpaceType;
use crate::v2::state::BState;
use crate::v2::zone::BZoneType;
pub use prim::{
    BAction, BColAlign, BColWidth, BControl, BPanelRole, BPopoverSide, BRailEntry, BRailGroup,
    BRailKind, BRailSide, BRailView, BRowShade, BSortDirection, BTableCell, BZoneToggle,
};

#[derive(Clone, Debug)]
pub enum BView {
    // ── Store-level containers ──
    Space {
        kind: BSpaceType,
        children: Vec<BView>,
    },
    Zone {
        kind: BZoneType,
        children: Vec<BView>,
    },
    Area {
        kind: BAreaType,
        child: Box<BView>,
    },
    Host {
        kind: BHostType,
        children: Vec<BView>,
    },
    Region {
        kind: BRegionType,
        children: Vec<BView>,
    },
    Pane {
        kind: BPaneType,
        children: Vec<BView>,
    },
    Part {
        kind: BPartType,
        body: Box<BView>,
    },
    Modal {
        kind: crate::v2::modal::BModalType,
        body: Box<BView>,
    },

    // ── Primitives: legal ONLY inside Part.body ──
    Row {
        children: Vec<BView>,
    },
    /// List row with a background shade (flat / alternating / selected).
    ListRow {
        children: Vec<BView>,
        shade: prim::BRowShade,
    },
    /// A table body row whose cells share the header's column geometry, so
    /// Name / Date / Size line up into aligned columns (Blender `space_file`
    /// `draw_details_columns`). `action`, when set, makes the whole row
    /// clickable (select a file / navigate into a directory).
    TableRow {
        cells: Vec<prim::BTableCell>,
        shade: prim::BRowShade,
        action: Option<BAction>,
    },
    /// The table header row — uses the SAME cell widths as `TableRow` bodies
    /// so columns align. Header cells are clickable to sort.
    TableHeader {
        cells: Vec<prim::BTableCell>,
    },
    /// A table-column heading with a stable reserved direction-indicator slot.
    SortHeader {
        label: String,
        direction: Option<prim::BSortDirection>,
        align: prim::BColAlign,
        action: BAction,
    },
    /// A Properties accordion heading. The entire row owns one toggle action;
    /// its disclosure chip is visual only, avoiding nested duplicate dispatch.
    AccordionHeader {
        title: String,
        expanded: bool,
        action: BAction,
    },
    /// Tree row — a `ListRow` that also owns Blender's outliner row geometry:
    /// `depth * METRICS.indent_unit` of left indent, a reserved disclosure
    /// slot (present but blank on leaves, so names stay column-aligned), then
    /// the row cells. Blender ref: `outliner_draw_tree_element`.
    TreeRow {
        depth: u16,
        /// `Some(true)` open, `Some(false)` closed, `None` = leaf (slot is
        /// reserved but nothing is drawn).
        expanded: Option<bool>,
        /// Fired when the disclosure triangle is clicked.
        toggle: Option<BAction>,
        /// Data-type icon, drawn in its own grid column immediately after
        /// the disclosure column — which is what puts a child row's arrow
        /// directly beneath its parent's icon.
        icon: String,
        /// Main cells (the name), which flex and clip.
        children: Vec<BView>,
        /// Right-aligned cluster (the restriction toggles), which never
        /// shrinks and is pushed hard against the row's right edge.
        trail: Vec<BView>,
        shade: prim::BRowShade,
    },
    /// Fixed-width blank of `level * METRICS.indent_unit`.
    Indent {
        level: u16,
    },
    Column {
        children: Vec<BView>,
    },
    Panel {
        role: BPanelRole,
        children: Vec<BView>,
    },
    Label {
        text: String,
    },
    /// Rotated-90° effect — column of characters.
    VLabel {
        text: String,
    },
    /// Bordered value read-out for property rows.
    Field {
        text: String,
    },
    /// Interactive search / text-entry field.
    ///
    /// Displays `value` when non-empty, `placeholder` otherwise (dimmed).
    /// `on_change` is fired on click to clear, giving immediate feedback; it
    /// carries the target `BPath` so the tree re-filters on the next render.
    ///
    /// Full IME text entry requires an `EntityInputHandler` entity (future
    /// work). For now the field shows the current state value and lets the
    /// user clear it. Blender ref: `UI_BTYPE_TEXT` / `uiItemStringO`.
    SearchField {
        placeholder: String,
        value: String,
        /// Fired on click-to-clear (with an empty String value), and in
        /// future on every IME keystroke. Must be `BAction::Set(path, _)`.
        on_change: BAction,
    },
    Icon {
        name: String,
    },
    /// Controls: one shared props record (`BControl`) tagged by the kind of
    /// control it renders. The differences between a button, a menu item, a
    /// menubar anchor and a tab are render-side, not data-side — so they are
    /// the same record with different `with_*` props, not four structs each
    /// with their own constructor-per-prop-combination family.
    Button(prim::BControl),
    /// Menu popover row — no bg by default, hover-only highlight.
    MenuItem(prim::BControl),
    /// Thin horizontal separator inside a menu popover.
    MenuSep,
    /// Flexible space — grows to push adjacent regions apart (e.g. the
    /// activity bar's Top/Center/Bottom three-region layout). Renders as a
    /// `flex_1` filler with no visual chrome.
    Spacer,
    /// Compact right-aligned Topbar controls for declared zone visibility.
    ZoneToggleGroup {
        controls: Vec<BZoneToggle>,
    },
    /// Strip tab — active/hover bg blends with the enclosing pane.
    Tab(prim::BControl),
    /// One vertical strip — the Activity bar, a Toolbar, a Properties NavBar
    /// or a Sidebar. The single primitive behind every vertical rail: entries
    /// carry their `group` (Top / Center / Bottom) and the renderer inserts
    /// flexible spacers between consecutive non-empty groups. When
    /// `expanded`, entry labels render inline; collapsed, they render as a
    /// hover tooltip at the strip's edge.
    Rail(prim::BRailView),
    /// Top-level menubar anchor — a header-height button that carries the
    /// open/closed state of the menu it deploys. Distinct from `Button`
    /// because it spans the FULL header height, so its accent underline
    /// lands exactly on the header's bottom border and the popover that
    /// drops from it starts on that same line.
    MenuAnchor(prim::BControl),
    /// Menu popover — anchor in flow; body deferred while open.
    Popover {
        anchor: Box<BView>,
        body: Vec<BView>,
        open: bool,
        close: BAction,
        side: prim::BPopoverSide,
    },
    /// Host-specific canvas (viewport / tree / node graph / text buffer).
    Canvas {
        host: BHostType,
    },
    /// Determinate progress bar with a leading status label. `fraction` is
    /// clamped to `0.0..=1.0` by the renderer. Used by the statusbar to show
    /// blend-import progress (Blender's `WM_progress`).
    ProgressBar {
        fraction: f32,
        label: String,
    },
    Empty,
}

/// Emits exactly one `BView::<level>` variant.
pub trait BViewItem {
    fn build(&self, state: &BState) -> BView;
}
/// Iterates a store collection into children.
pub trait BViewList {
    fn items(&self, state: &BState) -> Vec<BView>;
}

impl BView {
    pub fn empty() -> Self {
        Self::Empty
    }
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}
