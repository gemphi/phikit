//! Header right-hand controls — how the content is filtered or displayed.
//!
//! The twin of `part::search`: every host presents itself the same way —
//! what it is on the left, what you are looking for in the middle, how it is
//! filtered or displayed on the right. A host with nothing to put there
//! leaves the right end empty.
//!
//! Blender ref: `OUTLINER_PT_filter` and the file browser's
//! `FILEBROWSER_PT_filter`, both opened from an icon in their header.

use crate::v2::state::AppKey;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::state::val::V_FALSE;
use crate::v2::state::val::V_TRUE;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use crate::v2::view::prim::BPopoverSide;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::state::BPath;

/// Below this the header has no room for a row of controls on its right, so
/// they fold into one icon that opens them. Blender does the same with its
/// header popovers when an editor gets narrow.
const NARROW: f32 = 1100.0;

/// True when the window is too narrow for inline right-hand controls.
pub fn is_narrow(s: &BState) -> bool {
    s.get(AppKey::WindowWidth)
        .and_then(|w| w.parse::<f32>().ok())
        .is_some_and(|w| w < NARROW)
}

/// Below this the top bar's three cells no longer fit one line: the left
/// cell (the brand and the File… menus) keeps the first line and the
/// workspace tabs and the right-hand controls go under it.
const TOPBAR_WRAP: f32 = 960.0;

/// True when the top bar wraps to two lines.
pub fn topbar_wraps(s: &BState) -> bool {
    s.get(AppKey::WindowWidth)
        .and_then(|w| w.parse::<f32>().ok())
        .is_some_and(|w| w < TOPBAR_WRAP)
}

/// `ctx.header.<host>.filters` — whether the folded control set is open.
fn k_open(host: BHostType) -> BPath {
    BPath::owned(vec!["header", host.slug(), "filters"])
}

/// Fold `body` behind one icon, for a header with no room to show it inline.
fn folded(s: &BState, host: BHostType, body: Vec<BView>) -> BView {
    let key = k_open(host);
    let open = s.get(&key) == Some(V_TRUE);
    let toggle = BAction::Set(
        key.clone(),
        if open { V_FALSE } else { V_TRUE }.to_string(),
    );
    let anchor = BView::Button(BControl::new().with_icon("filter").with_action(toggle));
    BView::popover(
        anchor,
        body,
        open,
        BAction::Set(key, V_FALSE.to_string()),
        BPopoverSide::BelowEnd,
    )
}

#[derive(Clone, Copy, Debug)]
pub struct BFilter {
    pub host: BHostType,
}

impl BFilter {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

/// True when a host has right-hand header controls to offer.
pub fn has_filter(host: BHostType) -> bool {
    matches!(host, BHostType::Outliner | BHostType::View3D | BHostType::File | BHostType::Asset)
}

impl BViewItem for BFilter {
    fn build(&self, s: &BState) -> BView {
        match self.host {
            BHostType::Outliner => crate::v2::part::modebar::outliner_filter(s),
            // The viewport's shading radio is a display control, so it belongs
            // at the right end of the header like every other one — not beside
            // the mode picker on the left. On a narrow window the four modes
            // fold behind one icon rather than crowding the strip.
            BHostType::View3D => {
                let shading = crate::v2::part::modebar::shading::view(s);
                if is_narrow(s) {
                    folded(s, self.host, vec![shading])
                } else {
                    shading
                }
            }
            // The file browsers: display mode, hidden files, the filter and
            // its popover — Blender's header, right of the spacer.
            BHostType::File | BHostType::Asset => {
                crate::v2::part::file::path_bar::header_icons(s)
            }
            _ => BView::empty(),
        }
    }
}
