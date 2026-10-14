//! Whether an editor's sidebar region is open beside its tab strip —
//! Blender's `SpaceView3D.show_region_ui`, toggled by N (View ▸ Sidebar),
//! restored from a file's saved screen (`app/load_ui.rs`) and dragged shut
//! by its inner edge. Picking a tab opens it on that tab; picking the tab
//! already showing puts it away.

use super::set::{BSidebarBody, tabs_for};
use crate::v2::host::BHostType;
use crate::v2::state::rule::BRule;
use crate::v2::state::val::V_TRUE;
use crate::v2::state::{BPath, BState};

/// One host's sidebar region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BSidebarShown(pub BHostType);

impl BSidebarShown {
    /// `<host>.show.sidebar` — `true` while the region is open.
    pub fn key(self) -> BPath {
        BPath::owned(vec![self.0.slug(), "show", "sidebar"])
    }

    /// The 3D Viewport's key, for View ▸ Sidebar (N).
    pub fn view3d_key() -> BPath {
        Self(BHostType::View3D).key()
    }

    /// `host`'s key, for a View ▸ Sidebar (N) in a menu table several editors
    /// share (`tog_host`): the Image and UV editors, the node editors, the
    /// animation editors each flip their own region.
    pub fn key_for(host: BHostType) -> BPath {
        Self(host).key()
    }

    /// Is the region open? Closed until opened, as Blender's startup file
    /// has it.
    pub fn on(self, s: &BState) -> bool {
        self.holds_panels() && s.get(&self.key()) == Some(V_TRUE)
    }

    /// Does any of the host's tabs draw something in the region?
    pub fn holds_panels(self) -> bool {
        tabs_for(self.0).iter().any(|tab| tab.body != BSidebarBody::None)
    }

    /// The region's width beside the strip while it is open — the width
    /// every sidebar shares (`view::size`) — and the gap between them;
    /// nothing while it is shut.
    pub fn beside_width(self, s: &BState, style: &crate::v2::theme::style::BResolvedUiStyle) -> f32 {
        if !self.on(s) {
            return 0.0;
        }
        crate::v2::view::size::sidebar_column_width(s, style.sidebar_min_width(), style.dimensions.sidebar_min_width) + style.spacing.gap_xs
    }

    /// `<host>.sidebar.tab` — the key a tab writes when picked.
    pub fn tab_key(self) -> BPath {
        BPath::owned(vec![self.0.slug(), "sidebar", "tab"])
    }
}

/// A tab picked on the strip opens the region on it (`BSidebarList` puts
/// the region away itself when the tab picked is the one showing).
pub struct BSidebarOpenRule;

impl BSidebarOpenRule {
    fn host_of(path: &BPath) -> Option<BHostType> {
        match path.segs.as_ref() {
            [host, "sidebar", "tab"] => BHostType::from_slug(host).filter(|h| BSidebarShown(*h).holds_panels()),
            _ => None,
        }
    }
}

impl BRule for BSidebarOpenRule {
    fn wants(&self, path: &BPath) -> bool {
        Self::host_of(path).is_some()
    }
    fn after(&self, path: &BPath, _: &str, s: &mut BState) {
        if let Some(host) = Self::host_of(path) {
            s.set(&BSidebarShown(host).key(), V_TRUE);
        }
    }
}
