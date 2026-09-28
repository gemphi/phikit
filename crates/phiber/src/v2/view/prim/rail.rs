//! Rail primitive — THE vertical strip: the Activity bar, the Toolbar, the
//! Properties NavBar and the Sidebar are the same element with different
//! data, not four widgets.

use crate::v2::view::BView;
use crate::v2::view::prim::BControl;

/// WHICH vertical strip a `Rail` is. This one enum drives every behavioural
/// difference between the strips — width, expansion, cluster layout — so the
/// Activity bar, the Toolbar, the Properties NavBar and the Sidebar are the
/// same element with different data, not four widgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRailKind {
    /// Phiber-native global navigation: top / center / bottom groups, no
    /// expand cluster.
    Activity,
    /// Blender's left tool strip: tools + fixed bottom expand/paging cluster.
    Tools,
    /// Blender's Properties context-tab rail (left).
    NavBar,
    /// Blender's editor side rail (right).
    Sidebar,
}

impl BRailKind {
    /// Strip kinds share the fixed bottom expand/paging cluster; the Activity
    /// bar's bottom is its own Bottom group (user / settings) instead.
    pub fn has_strip_cluster(self) -> bool {
        !matches!(self, BRailKind::Activity)
    }
}

/// Which edge of the editor the strip is pinned to. Left rails round their
/// active cap on the left and fly labels out to the right; Right rails mirror.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRailSide {
    Left,
    Right,
}

/// Vertical slot a rail entry belongs to. `Top` entries stack from the top,
/// `Bottom` entries pin to the bottom, and `Center` entries float between
/// them — flexible spacers are inserted between consecutive non-empty
/// groups by the renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRailGroup {
    Top,
    Center,
    Bottom,
}

/// One rail entry: a `BControl` — the same record buttons, tabs and menu
/// items carry (icon / label / active / action) — plus the vertical group it
/// belongs to. State-derived — never carries its own state.
#[derive(Clone, Debug)]
pub struct BRailEntry {
    pub control: BControl,
    pub group: BRailGroup,
}

/// The whole strip as one data record — `BView::Rail` simply wraps it.
#[derive(Clone, Debug)]
pub struct BRailView {
    pub kind: BRailKind,
    pub side: BRailSide,
    pub expanded: bool,
    pub entries: Vec<BRailEntry>,
    /// The region beside the strip while it is open — the sidebar's panels
    /// (`part/sidebar/panel.rs`); `None` for a strip alone.
    pub panels: Option<Vec<crate::v2::view::BView>>,
}

impl BRailEntry {
    pub fn new(control: BControl, group: BRailGroup) -> Self {
        Self { control, group }
    }
}

impl BView {
    pub fn rail(v: BRailView) -> Self {
        Self::Rail(v)
    }
}
