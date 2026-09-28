//! Asset Shelf — the brushes a paint or sculpt mode offers.
//! Blender ref: `RGN_TYPE_ASSET_SHELF` and its header.
//!
//! Two parts, because there are two regions: the shelf itself is the grid of
//! brushes, and the header is the thin strip along its top edge that names
//! the active brush and folds the shelf away. Both were rendering the same
//! tile row, which is why the Image Editor showed six icons twice over and an
//! empty band underneath.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BPath;
use crate::v2::state::BState;
use crate::v2::state::val::V_TRUE;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use self::set::tiles_for;
pub use set::BAssetTileSpec;
pub use set::has_shelf;

// `BAssetShelfList` — the brush tiles for the active host and mode.
// 
// A tile is an operation: it makes its brush the active one, and reads as
// active while it is. They used to be bare icons that did nothing.
#[derive(Clone, Copy, Debug)]
pub struct BAssetShelfList {
    pub host: BHostType,
}

impl BAssetShelfList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BAssetShelfList {
    fn items(&self, s: &BState) -> Vec<BView> {
        let mode = crate::v2::part::modebar::active_mode(s, self.host);
        let active = active_brush(s, self.host, mode);
        let key = k_brush(self.host);
        tiles_for(self.host, mode)
            .iter()
            .map(|t| {
                BView::Button(
                    BControl::new()
                        .with_icon(t.thumb)
                        .with_tooltip(t.label)
                        .with_active(t.label == active)
                        .with_action(BAction::Set(key.clone(), t.label.to_string())),
                )
            })
            .collect()
    }
}

/// `<host>.shelf.brush` — the active brush in this host.
pub fn k_brush(host: BHostType) -> BPath {
    BPath::owned(vec![host.slug(), "shelf", "brush"])
}

/// `<host>.shelf.expanded` — whether the shelf is unfolded.
pub fn k_expanded(host: BHostType) -> BPath {
    BPath::owned(vec![host.slug(), "shelf", "expanded"])
}

/// The active brush, or the mode's first one.
pub fn active_brush(s: &BState, host: BHostType, mode: &str) -> &'static str {
    let tiles = set::tiles_for(host, mode);
    s.get(&k_brush(host))
        .and_then(|name| tiles.iter().find(|t| t.label == name))
        .or_else(|| tiles.first())
        .map(|t| t.label)
        .unwrap_or("")
}

fn is_expanded(s: &BState, host: BHostType) -> bool {
    s.get(&k_expanded(host)) == Some(V_TRUE)
}

/// The shelf: a grid of brushes, and nothing at all outside a painting mode.
#[derive(Clone, Copy, Debug)]
pub struct BAssetShelf {
    pub host: BHostType,
}

impl BAssetShelf {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BAssetShelf {
    fn build(&self, s: &BState) -> BView {
        let mode = crate::v2::part::modebar::active_mode(s, self.host);
        // No shelf in this mode, or folded away: the region has no content,
        // so the host drops it rather than reserving a strip of empty space.
        if !has_shelf(self.host, mode) || !is_expanded(s, self.host) {
            return BView::empty();
        }
        BView::row(BAssetShelfList::new(self.host).items(s))
    }
}

/// The shelf's header: the fold control and the active brush's name.
#[derive(Clone, Copy, Debug)]
pub struct BAssetShelfHeader {
    pub host: BHostType,
}

impl BAssetShelfHeader {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BAssetShelfHeader {
    fn build(&self, s: &BState) -> BView {
        let mode = crate::v2::part::modebar::active_mode(s, self.host);
        if !has_shelf(self.host, mode) {
            return BView::empty();
        }
        let open = is_expanded(s, self.host);
        BView::row(vec![
            BView::Button(
                BControl::new()
                    .with_icon(if open { "chevron_down" } else { "chevron_up" })
                    .with_label(active_brush(s, self.host, mode))
                    .with_tooltip(if open { "Fold the shelf" } else { "Show the shelf" })
                    .with_active(open)
                    .with_action(BAction::Toggle(k_expanded(self.host))),
            ),
        ])
    }
}
