//! Host — one editor kind.
//!
//! Every host owns a folder: `host/<name>/mod.rs` is the structure (what the
//! host is — kind, label, icon, slug) and `host/<name>/set.rs` is the content
//! (its regions, panes and parts). `set.rs` here is only the index that puts
//! them in `BHostType` order. Widget content lives in `part::`.

pub mod header;
pub mod set;
pub mod action;
pub mod activity;
pub mod asset;
pub mod buts;
pub mod clip;
pub mod compositor;
pub mod console;
pub mod drivers;
pub mod file;
pub mod geonode;
pub mod image;
pub mod info;
pub mod ipo;
pub mod logic;
pub mod nla;
pub mod node;
pub mod outliner;
pub mod pref;
pub mod seq;
pub mod shader;
pub mod spreadsheet;
pub mod statusbar;
pub mod text;
pub mod time;
pub mod topbar;
pub mod uv;
pub mod view3d;

use crate::v2::region::BRegionList;
use crate::v2::region::BRegionSpec;
use crate::v2::spec::BSpec;
use crate::v2::spec::BSpecKind;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

// `BHostType` — editor kind. Mirrors Blender 5.2 space types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BHostType {
    // Layout hosts (Blender's default workspace)
    Topbar,
    View3D,
    Outliner,
    Buts,
    Time,
    Statusbar,
    // Other Blender editors
    File,
    Image,
    Info,
    Seq,
    Text,
    Action,
    Nla,
    Node,
    Logic,
    Console,
    Uv,
    Compositor,
    Shader,
    GeoNode,
    Clip,
    Drivers,
    Asset,
    Spreadsheet,
    Ipo,
    Prefs,
    // Phiber-native global navigation rail (lives in a Left zone).
    Activity,
    Empty,
}

impl BHostType {
    /// Every host in declaration order.
    pub const ALL: &'static [BHostType] = &[
        Self::Topbar,
        Self::View3D,
        Self::Outliner,
        Self::Buts,
        Self::Time,
        Self::Statusbar,
        Self::File,
        Self::Image,
        Self::Info,
        Self::Seq,
        Self::Text,
        Self::Action,
        Self::Nla,
        Self::Node,
        Self::Logic,
        Self::Console,
        Self::Uv,
        Self::Compositor,
        Self::Shader,
        Self::GeoNode,
        Self::Clip,
        Self::Drivers,
        Self::Asset,
        Self::Spreadsheet,
        Self::Ipo,
        Self::Prefs,
        Self::Activity,
        Self::Empty,
    ];

    /// Hosts a Center-zone area can switch between (every editor except the
    /// two global chrome hosts Topbar and Statusbar, and the null Empty).
    pub const SWITCHABLE: &'static [BHostType] = &[
        Self::View3D,
        Self::Outliner,
        Self::Buts,
        Self::File,
        Self::Image,
        Self::Info,
        Self::Seq,
        Self::Text,
        Self::Action,
        Self::Nla,
        Self::Time,
        Self::Node,
        Self::Logic,
        Self::Console,
        Self::Uv,
        Self::Compositor,
        Self::Shader,
        Self::GeoNode,
        Self::Clip,
        Self::Drivers,
        Self::Asset,
        Self::Spreadsheet,
        Self::Ipo,
        Self::Prefs,
    ];

    pub fn label(self) -> &'static str {
        self.spec().label
    }
    pub fn icon(self) -> &'static str {
        self.spec().icon
    }

    /// Hosts whose Center region owns a wall-to-wall graphical canvas that
    /// Blender paints edge-to-edge (`RGN_TYPE_WINDOW`-dominant editors): the
    /// 3D viewport, every node graph, image / UV editors, the sequencer /
    /// clip previews, and the animation timelines (Action / Nla / Ipo /
    /// Drivers). For these the content region fills the full host height with
    /// **no** surround padding (the canvas / grid / graph / track sheet
    /// reaches every pane edge) and the area draws no card chrome.
    ///
    /// This is the single source of truth for the full-bleed treatment — the
    /// area and region walkers consult it rather than re-declaring the set.
    /// Whether the side panels *float* is a separate axis; see
    /// [`Self::floats_side_panels`].
    /// A host that is a fixed-width strip: it has one width, never gives it
    /// away, and the zone holding it has no boundary to drag.
    pub const fn is_fixed_strip(self) -> bool {
        matches!(self, Self::Activity)
    }

    /// A host that is one header strip tall — the status bar and the top
    /// bar. It takes its own height from the zone and the editors beside it
    /// share the rest, so a taller bottom zone grows the Timeline, not the
    /// status bar.
    pub const fn is_fixed_bar(self) -> bool {
        matches!(self, Self::Statusbar | Self::Topbar)
    }

    pub const fn fills_content(self) -> bool {
        matches!(
            self,
            Self::View3D
                | Self::Node
                | Self::Shader
                | Self::Compositor
                | Self::GeoNode
                | Self::Image
                | Self::Uv
                | Self::Seq
                | Self::Clip
                | Self::Action
                | Self::Nla
                | Self::Ipo
                | Self::Drivers
                | Self::Time
        )
    }

    /// Content-fill hosts whose Left / Right panes are *floating* overlays
    /// (Blender's translucent N-panel / T-panel over the viewport / graph),
    /// as opposed to docked side regions. This is the subset of
    /// [`Self::fills_content`] that uses the Toolbar / Sidebar floating
    /// treatment: the 3D viewport, node graphs, and image / UV / sequencer /
    /// clip previews.
    ///
    /// The timelines (Action / Nla / Ipo / Drivers) are deliberately excluded
    /// even though they fill content: their Left pane is a **docked** Channels
    /// list (track names) that must keep its flex slot, not float over the
    /// track sheet.
    /// Hosts whose Tools region is a column of panels (the file browser's
    /// source list, the asset browser's libraries) rather than an icon rail.
    pub const fn tools_hold_panels(self) -> bool {
        matches!(self, Self::File | Self::Asset)
    }

    /// Hosts whose search field sits in their tool header (the file
    /// browser's path strip), not in the header's middle.
    pub const fn searches_in_tool_header(self) -> bool {
        matches!(self, Self::File | Self::Asset)
    }

    /// Hosts that keep their own header when raised in a dialog: the file
    /// browser window shows its View / Select menus under the title, as
    /// Blender's does, and Preferences its View / Preferences menus with
    /// Save Preferences at the right (`USERPREF_HT_header`). Every other
    /// elevated host is named by the title bar and its header would be a
    /// second one.
    pub const fn keeps_header_in_dialog(self) -> bool {
        matches!(self, Self::File | Self::Asset | Self::Prefs)
    }

    pub const fn floats_side_panels(self) -> bool {
        matches!(
            self,
            Self::View3D
                | Self::Node
                | Self::Shader
                | Self::Compositor
                | Self::GeoNode
                | Self::Image
                | Self::Uv
                | Self::Seq
                | Self::Clip
        )
    }

    /// Hosts whose header divides its width into three equal cells rather
    /// than letting the Left and Right ends hug their content.
    ///
    /// The top bar spans the window and its workspace tabs are the one thing
    /// the eye tracks across a space change. With hugging ends, the centre
    /// cell starts wherever the Left cell happens to finish, so switching to a
    /// space whose menus are a different width slid the tabs sideways. Equal
    /// thirds pin the centre to the window's centre, whatever the menus do.
    ///
    /// Editor headers are deliberately excluded: an area can be narrower than
    /// its own menu row, and thirds would crush it.
    pub const fn pins_header_cells(self) -> bool {
        matches!(self, Self::Topbar | Self::Statusbar)
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|&h| h.slug() == s)
    }
}


// `BHostSpec` — one editor host with its regions inline.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BHostSpec {
    pub kind: BHostType,
    pub label: &'static str,
    pub icon: &'static str,
    pub regions: &'static [BRegionSpec],
}

impl BSpec for BHostSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Host(self.kind)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

// `BHostItem` — one host. Emits `BView::Host` populated by `BRegionList`.

#[derive(Clone, Copy, Debug)]
pub struct BHostItem {
    pub host: BHostType,
}

impl BHostItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BHostItem {
    fn build(&self, state: &BState) -> BView {
        BView::Host {
            kind: self.host,
            list: BRegionList::new(self.host).items(state),
        }
    }
}

// `BHostList` — iterate a set of hosts (Switcher popover contents).

#[derive(Clone, Copy, Debug)]
pub struct BHostList<'a> {
    pub hosts: &'a [BHostType],
}

impl<'a> BHostList<'a> {
    pub fn new(hosts: &'a [BHostType]) -> Self {
        Self { hosts }
    }
}

impl<'a> BViewList for BHostList<'a> {
    fn items(&self, state: &BState) -> Vec<BView> {
        self.hosts
            .iter()
            .map(|&h| BHostItem::new(h).build(state))
            .collect()
    }
}

// Per-host region declarations, re-exported so `host::<name>::REGIONS` reads
// as before.

// ── Layout (was `view/render/host.rs`) ──
// Host-level layout: hosts stack their regions vertically (Header on top,
// ToolHeader below it, Center flexing, Footer at the bottom).
// Blender ref: `screen_geom.cc` region packing.
use crate::v2::view::layout::{BAxis, BHostLayout};

impl BHostLayout for BHostType {
    /// Hosts always stack their regions vertically. Region kinds decide
    /// their own placement within this column (top-aligned Header /
    /// bottom-aligned Footer / flex Center).
    fn axis(&self) -> BAxis {
        BAxis::Column
    }
}
