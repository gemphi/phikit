//! Modebar — the top-left mode dropdown of a host's header. Blender ref:
//! `VIEW3D_HT_header` mode picker (`layout.template_header()` +
//! `object.mode` enum). Anchor shows the active mode's icon + label;
//! click opens a popover listing every mode from `set::modes_for(host)`.
//!
//! For the Outliner, Blender places the search field (`filter_text`) and a
//! Filter popover button inline in the header immediately after the display-
//! mode dropdown. We replicate this by returning a `BView::Row` wrapping
//! `[mode_popover, search_field, filter_popover]` when `host == Outliner`.

pub mod set;
pub mod workspace;
pub mod shading;

use crate::v2::host::BHostType;
use crate::v2::state::BKey as _;
use crate::v2::state::BState;
use crate::v2::state::MenuKey;
use crate::v2::state::OutlinerKey;
use crate::v2::state::val::V_EMPTY;
use crate::v2::state::val::V_FALSE;
use crate::v2::state::val::V_TRUE;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use crate::v2::view::prim::BPopoverSide;
use self::set::default_mode;
use self::set::mode_key;
use self::set::modes_for;
pub use set::BModeSpec;

// `BModebarList` — reads `modes_for(host)` + active mode from `ctx.mode`.
#[derive(Clone, Copy, Debug)]
pub struct BModebarList {
    pub host: BHostType,
}

impl BModebarList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BModebarList {
    fn items(&self, s: &BState) -> Vec<BView> {
        let key = mode_key(self.host);
        let active = s.get(&key).unwrap_or_else(|| default_mode(self.host));
        modes_for(self.host)
            .iter()
            .map(|m| {
                BView::MenuItem(
                    BControl::new()
                        .with_icon(if m.label == active { "check" } else { m.icon })
                        .with_label(m.label)
                        .with_action(BAction::Set(key.clone(), m.label.to_string())),
                )
            })
            .collect()
    }
}

/// The mode a host is currently in — the one place that resolves the key and
/// its fallback, so a part that varies by mode (the asset shelf) reads the
/// same answer the mode bar shows.
pub fn active_mode(s: &BState, host: BHostType) -> &'static str {
    s.get(&mode_key(host))
        .and_then(|m| modes_for(host).iter().find(|spec| spec.label == m))
        .map(|spec| spec.label)
        .unwrap_or_else(|| default_mode(host))
}

#[derive(Clone, Copy, Debug)]
pub struct BModebar {
    pub host: BHostType,
}

impl BModebar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

fn active_icon(host: BHostType, active: &str) -> &'static str {
    modes_for(host)
        .iter()
        .find(|m| m.label == active)
        .map(|m| m.icon)
        .unwrap_or("object")
}

/// The outliner's filter popover (`OUTLINER_PT_filter`), for `part::filter` to place at the right
/// end of the header.
pub fn outliner_filter(s: &BState) -> BView {
    let fkey = OutlinerKey::FilterOpen.path();
    let open = s.get(&fkey) == Some(V_TRUE);
    let toggle = BAction::Set(
        fkey.clone(),
        if open { V_FALSE.to_string() } else { V_TRUE.to_string() },
    );
    let close = BAction::Set(fkey, V_FALSE.to_string());
    let anchor = BView::Button(BControl::new().with_icon("filter").with_action(toggle));
    BView::popover(
        anchor,
        crate::v2::part::outliner::filter::popover::body(s),
        open,
        close,
        BPopoverSide::Below,
    )
}

impl BViewItem for BModebar {
    fn build(&self, s: &BState) -> BView {
        // Host-scoped like every other menubar popover, so opening the mode
        // dropdown in one editor does not open it in the rest.
        let key = MenuKey::Top(self.host.slug(), "mode").path();
        let open = s.get(&key) == Some(V_TRUE);
        let toggle = BAction::Toggle(key.clone());
        let close = BAction::Set(key, V_EMPTY.to_string());
        let active = s
            .get(&mode_key(self.host))
            .unwrap_or_else(|| default_mode(self.host));
        // Same header-height anchor as the menubar labels, so the mode
        // dropdown lines up with the menus beside it.
        let anchor = BView::MenuAnchor(
            BControl::new()
                .with_icon(active_icon(self.host, active))
                .with_label(active)
                .with_active(open)
                .with_action(toggle),
        );
        let body = BModebarList::new(self.host).items(s);
        let mode_popover = BView::popover(anchor, body, open, close, BPopoverSide::Below);

        // The header's shape is owned by `region::complete_header`:
        // the switcher and menus on the left, the host's search in the
        // middle (`part::search`), its filters on the right
        // (`part::filter`). The mode picker is the only thing the mode bar
        // contributes — building a search here as well is what put two
        // search fields in the outliner's header.
        // The shading radio moved to the header's right end
        // (`part::filter`), so the mode bar contributes only the mode picker
        // — the same shape in every host.
        mode_popover
    }
}
