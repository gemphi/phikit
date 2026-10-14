//! Toolbar — vertical column of tool icons.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::part::strip;
use crate::v2::state::BPath;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use crate::v2::view::prim::BRailEntry;
use crate::v2::view::prim::BRailGroup;
use crate::v2::view::prim::BRailKind;
use crate::v2::view::prim::BRailSide;
use crate::v2::view::prim::BRailView;
use self::set::tools_for;
pub use set::BToolSpec;

/// Whether an editor's tool region is shown — Blender's
/// `SpaceView3D.show_region_toolbar`, toggled by T (View ▸ Toolbar). Shown by
/// default, as Blender's startup file has the tool column open; the same
/// show/hide family as the sidebar (`sidebar::show`, N), so an editor hides
/// its own tool rail. Resumed with the session like the sidebar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BToolbarShown(pub BHostType);

impl BToolbarShown {
    /// `<host>.show.toolbar`.
    pub fn key(self) -> BPath {
        BPath::owned(vec![self.0.slug(), "show", "toolbar"])
    }

    /// The 3D Viewport's key, for a plain View ▸ Toolbar entry.
    pub fn view3d_key() -> BPath {
        Self(BHostType::View3D).key()
    }

    /// `host`'s key, for a shared View ▸ Toolbar (T) table (`tog_host`).
    pub fn key_for(host: BHostType) -> BPath {
        Self(host).key()
    }

    /// Shown unless turned off — on by default (`BFlipRule`'s `!` sense).
    pub fn on(self, s: &BState) -> bool {
        s.get(&self.key()) != Some(crate::v2::state::val::V_FALSE)
    }
}

// `BToolbarList` — reads `tools_for(host)` + `host.<h>.tool` and emits ONE
// `BView::Rail` (the shared vertical strip element) with a fixed bottom
// expand/paging cluster.
#[derive(Clone, Copy, Debug)]
pub struct BToolbarList {
    pub host: BHostType,
}

impl BToolbarList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    /// The whole strip as one view.
    pub fn rail(&self, s: &BState) -> BView {
        let tools = tools_for(self.host);
        let default_tool = tools.first().map(|t| t.label).unwrap_or("");
        let active = s.host(self.host).get("tool").unwrap_or(default_tool);
        let expanded = strip::is_expanded(s, self.host, "tool");

        let select_key = BPath::owned(vec![self.host.slug(), "tool"]);
        let mut entries: Vec<BRailEntry> = tools
            .iter()
            .map(|t| BRailEntry {
                control: BControl::new()
                    .with_icon(t.icon)
                    .with_label(t.label)
                    .with_active(t.label == active)
                    .with_action(BAction::Set(select_key.clone(), t.label.to_string())),
                group: BRailGroup::Top,
            })
            .collect();
        let labels: Vec<&'static str> = tools.iter().map(|t| t.label).collect();
        entries.extend(strip::cluster_entries(
            self.host,
            "tool",
            select_key,
            &labels,
            active,
            expanded,
        ));
        BView::rail(BRailView {
            kind: BRailKind::Tools,
            side: BRailSide::Left,
            expanded,
            entries,
            panels: None,
        })
    }
}

impl BViewList for BToolbarList {
    fn items(&self, _s: &BState) -> Vec<BView> {
        vec![]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BToolbar {
    pub host: BHostType,
}

impl BToolbar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BToolbar {
    fn build(&self, s: &BState) -> BView {
        // T hides the whole tool rail (`BToolbarShown`), as Blender's
        // `show_region_toolbar` hides the region; the floating pane draws
        // nothing when its parts are empty (`view::draw::pane`).
        if !BToolbarShown(self.host).on(s) {
            return BView::empty();
        }
        BToolbarList::new(self.host).rail(s)
    }
}
