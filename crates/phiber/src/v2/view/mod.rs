//! `BView` — declarative view tree. Seven store-level variants plus primitives.
//! Primitives (`Row`/`Column`/`Panel`/`Icon`/`Label`/`Button`) legal ONLY
//! inside the `body` of a `Part` node.

pub mod prim;
pub mod scroll;
pub mod size;
pub mod draw;
pub mod layout;
pub mod text;

use crate::v2::area::BAreaType;
use crate::v2::host::BHostType;
use crate::v2::pane::BPaneType;
use crate::v2::part::BPartType;
use crate::v2::region::BRegionType;
use crate::v2::space::BSpaceType;
use crate::v2::state::BState;
use crate::v2::zone::BZoneType;
pub use prim::{
    BAction, BCard, BCardKind, BColAlign, BColWidth, BControl, BPopoverSide, BRailEntry, BRailGroup,
    BRailKind, BRailSide, BRailView, BRowShade, BSortDirection, BTableCell, BThumb, BZoneToggle,
};

#[derive(Clone, Debug)]
pub enum BView {
    // ── Store-level containers ──
    Space {
        kind: BSpaceType,
        list: Vec<BView>,
    },
    Zone {
        kind: BZoneType,
        list: Vec<BView>,
    },
    Area {
        kind: BAreaType,
        child: Box<BView>,
    },
    Host {
        kind: BHostType,
        list: Vec<BView>,
    },
    Region {
        kind: BRegionType,
        list: Vec<BView>,
    },
    Pane {
        kind: BPaneType,
        list: Vec<BView>,
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
        list: Vec<BView>,
    },
    /// List row with a background shade (flat / alternating / selected).
    ListRow {
        list: Vec<BView>,
        shade: prim::BRowShade,
    },
    /// A table body row whose cells share the header's column geometry, so
    /// Name / Date / Size line up into aligned columns (Blender `space_file`
    /// `draw_details_columns`). `action`, when set, makes the whole row
    /// clickable (select a file / navigate into a directory).
    TableRow {
        cell_list: Vec<prim::BTableCell>,
        shade: prim::BRowShade,
        action: Option<BAction>,
    },
    /// The table header row — uses the SAME cell widths as `TableRow` bodies
    /// so columns align. Header cells are clickable to sort.
    TableHeader {
        cell_list: Vec<prim::BTableCell>,
    },
    /// A table-column heading with a stable reserved direction-indicator slot.
    SortHeader {
        label: String,
        direction: Option<prim::BSortDirection>,
        align: prim::BColAlign,
        action: BAction,
    },
    /// A panel — Blender's `Panel`: heading strip, body, optional foot,
    /// and its traits (fold, kind). See `card`.
    Card(Box<prim::BCard>),
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
        /// Between the disclosure and the icon: a collection row's exclude
        /// checkbox (`outliner_draw_tree_element`'s `TSE_LAYER_COLLECTION`
        /// checkbox). Empty on every other row.
        lead: Vec<BView>,
        /// Data-type icon, drawn in its own grid column immediately after
        /// the disclosure column — which is what puts a child row's arrow
        /// directly beneath its parent's icon.
        icon: String,
        /// Main cells (the name), which flex and clip.
        list: Vec<BView>,
        /// Right-aligned cluster (the restriction toggles), which never
        /// shrinks and is pushed hard against the row's right edge.
        trail: Vec<BView>,
        shade: prim::BRowShade,
        /// What the row carries when dragged and what a drop on it fires.
        drag: Option<prim::BTreeDrag>,
    },
    /// Fixed-width blank of `level * METRICS.indent_unit`.
    Indent {
        level: u16,
    },
    Column {
        list: Vec<BView>,
    },
    /// A view built once and shown again until what it was built from
    /// changes — the Outliner's rows across an orbit (`part::outliner`).
    /// Rendered as the view it holds; cloned by reference.
    Shared(std::rc::Rc<BView>),
    /// A property row split into a label column and a value column —
    /// Blender's `uiLayout.use_property_split` row (`ui_item_prop_split`).
    Split {
        list: Vec<BView>,
    },
    Label {
        text: String,
    },
    /// A read-out as its two halves: the caption and the figure. Kept apart
    /// all the way to the renderer so the two can be drawn in different
    /// shades (`view::draw::pair`) — a joined string cannot be.
    Pair {
        label: String,
        value: String,
    },
    /// Wrapping text for reading, by role.
    Prose {
        role: prim::BProse,
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
    /// Text field the user can type into.
    ///
    /// Shows `value`, or a dimmed `placeholder` when it is empty. Clicking it
    /// puts the caret there (`state::edit`); the window's key handler then
    /// writes every keystroke into the field's own key, so the tree re-filters
    /// or re-navigates on the next render. Blender ref: `UI_BTYPE_TEXT`.
    SearchField {
        placeholder: String,
        value: String,
        /// True while this field holds the caret: it draws the caret and the
        /// focused border.
        editing: bool,
        /// Names the state key being edited. Must be `BAction::Set(path, _)`.
        on_change: BAction,
        /// True for a filter, which leads with the magnifier; a field that
        /// names something (a file, a path) does not.
        searches: bool,
    },
    /// A path read as the folders it runs through, Explorer's address bar:
    /// each crumb its folder's link, the box around them a click into
    /// typing the path (`edit`, run in order).
    Crumbs {
        trail: Vec<(String, BAction)>,
        edit: Vec<BAction>,
    },
    Icon {
        name: String,
    },
    /// Controls: one shared props record (`BControl`) tagged by the kind of
    /// control it renders. The differences between a button, a menu item, a
    /// menubar anchor and a tab are render-side, not data-side — so they are
    /// the same record with different `with_*` props, not four structs each
    /// with their own constructor-per-prop-combination family.
    /// A widget that edits one state value (`prim::ctrl`).
    Ctrl(Box<prim::BCtrl>),
    /// A bar you sweep: the pointer's place along it is the value
    /// (`view/draw/track.rs`). `share` is how full it is drawn, 0..1.
    ///
    /// The joint between a declarative tree and a gesture. `track::draggable`
    /// wants a `Stateful<Div>`, which only a renderer has, so anything that
    /// describes its contents as `BView` — a popover's body, a panel's rows —
    /// could not put a draggable bar in them. Now it says `Track` and the walker
    /// does the wiring, the way it already does for a `Ctrl` or a `Popover`.
    Track {
        track: Box<crate::v2::view::draw::BTrack>,
        share: f32,
        /// What the bar reads out, beside itself.
        text: String,
    },
    /// Blender's colour wheel and its value bar (`view/draw/track/wheel`).
    Wheel {
        wheel: Box<crate::v2::view::draw::BWheel>,
    },
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
    /// Preview tile (File Browser thumbnails display).
    Thumbnail(Box<prim::BThumb>),
    /// Platform window controls in the self-drawn title bar
    /// (`leading`: macOS traffic-light space; else min / max / close).
    WindowControls { leading: bool },
    /// Wrapping left-to-right layout of tiles.
    Grid {
        list: Vec<BView>,
    },
    /// Host-specific canvas (viewport / tree / node graph / text buffer).
    /// `camera` is the 3D viewport's camera view: whether it looks through
    /// the scene camera now (`RegionView3D.persp == RV3D_CAMOB`, with its
    /// frame and passepartout) and the key that says so for this area.
    Canvas {
        host: BHostType,
        camera: crate::v2::scene::camera_view::BCameraSlot,
    },
    /// Determinate progress bar with a leading status label. `fraction` is
    /// clamped to `0.0..=1.0` by the renderer. `wide` is the full-width shape
    /// (bar over its label, the loading panel's bar across the bottom edge);
    /// the narrow shape boxes the bar to a fixed width so its label can change
    /// every tick without sliding it (the render job's inline status-bar bar).
    ProgressBar {
        fraction: f32,
        label: String,
        wide: bool,
    },
    /// The action log (`state::log`): its newest line when folded, every
    /// line in a scroller when `open`, with the control that folds it.
    Log {
        open: bool,
        toggle: prim::BControl,
    },
    /// A person's picture, round, `size` px across: the image at `path`
    /// when there is one, else the user glyph in a ring (`draw/avatar.rs`).
    Avatar {
        path: String,
        size: f32,
    },
    /// An icon with a count beside it — the Outliner's icon row for a
    /// collapsed item (`outliner_draw_iconrow`): what is folded under it,
    /// one glyph per kind, the number when there is more than one
    /// (`draw/badge.rs`).
    Badge {
        icon: String,
        count: usize,
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
    /// The view as built: itself, or the one a `Shared` holds (a kept view
    /// is shown as what it keeps).
    pub fn shown(&self) -> &BView {
        match self {
            Self::Shared(view) => view.shown(),
            other => other,
        }
    }

    pub fn empty() -> Self {
        Self::Empty
    }
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}
