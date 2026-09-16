//! `BHostType` — editor kind. Mirrors Blender 5.2 space types.

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

    pub fn spec(self) -> &'static super::BHostSpec {
        super::set::spec(self)
    }
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

    pub fn from_slug(s: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|&h| super::set::slug(h) == s)
    }
}

/// Convenience re-export so callers can write `host_slug(h)`.
pub fn host_slug(h: BHostType) -> &'static str {
    super::set::slug(h)
}
