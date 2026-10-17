//! Area — a slot inside a zone that holds one active host.

pub mod set;

pub mod current;
pub mod maximize;
pub mod size;
pub mod switch;

use crate::v2::host::BHostItem;
use crate::v2::host::BHostType;
use crate::v2::spec::BSpec;
use crate::v2::spec::BSpecKind;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::zone::BZoneType;
pub use current::BScope;
pub use current::area_key;
pub use current::current_area;
pub use current::current_scope;
pub use current::current_space;
pub use current::switcher_key;
pub use current::with_area;
pub use current::with_scope;
pub use current::with_space;
pub use size::ResizeError;
pub use size::apply_delta;
pub use size::apply_drag;
pub use size::area_fraction;
pub use size::area_size_key;
pub use size::area_size_prefix;
pub use size::boundary_pair;
pub use size::clamp_to_min;
pub use size::normalize;
pub use size::reset_equal;
pub use size::snap_to_equal;

// `BAreaType` — position tag inside a zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BAreaType {
    Top,
    Left,
    Center,
    Right,
    Bottom,
}

// `BAreaSpec` — one area within a space. Static; owned by its space.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAreaSpec {
    pub kind: BAreaType,
    pub zone: BZoneType,
    pub default_host: BHostType,
    /// Hosts this area can switch between.
    pub hosts: &'static [BHostType],
}

impl BSpec for BAreaSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Area(self.kind)
    }
    fn label(&self) -> &'static str {
        self.kind.label()
    }
}

// `BAreaItem` — one area. Resolves the active host and delegates to `BHostItem`.

#[derive(Clone, Copy, Debug)]
pub struct BAreaItem {
    pub area: BAreaSpec,
}

impl BAreaItem {
    pub fn new(area: BAreaSpec) -> Self {
        Self { area }
    }

    /// The editor type this area is showing: whatever the switcher wrote into
    /// this area's slot, else the area's declared default.
    fn active_host(&self, s: &BState) -> BHostType {
        let Some(scope) = current::current_scope() else {
            return self.area.default_host;
        };
        s.get(&current::area_key(scope, &self.area))
            .and_then(BHostType::from_slug)
            .unwrap_or(self.area.default_host)
    }
}

impl BViewItem for BAreaItem {
    fn build(&self, state: &BState) -> BView {
        let host = self.active_host(state);
        // Record this area for the whole subtree so the switcher in the
        // host header can address the slot it lives in.
        // … and as the area its view keys belong to (`scene::view_slot`), so
        // a control built here — the N panel's focal length — edits this
        // area's view, the same slot the renderer draws it under.
        let slot = crate::v2::scene::view_slot::BViewSlot::new(crate::v2::space::BSpaceList::active(state).slug(), self.area.kind.label());
        current::with_area(self.area, || {
            slot.with(|| BView::Area { kind: self.area.kind, child: Box::new(BHostItem::new(host).build(state)) })
        })
    }
}

// `BAreaList` — iterate the areas of a zone.

#[derive(Clone, Copy, Debug)]
pub struct BAreaList<'a> {
    pub areas: &'a [BAreaSpec],
}

impl<'a> BAreaList<'a> {
    pub fn new(areas: &'a [BAreaSpec]) -> Self {
        Self { areas }
    }
}

impl<'a> BViewList for BAreaList<'a> {
    fn items(&self, state: &BState) -> Vec<BView> {
        self.areas
            .iter()
            .map(|a| BAreaItem::new(*a).build(state))
            .collect()
    }
}

// ── Layout (was `view/render/area.rs`) ──
// Area-level layout: each area wraps its single active host. Sizing is
// driven by the area's `BAreaType` and read from `theme::METRICS`.
// 
// Blender ref: `ScrArea` allocates a rectangle in the screen `AreaMap`.
// Top/Bottom get fixed heights, Left/Right get fixed widths, Center flexes.
// Left/Right areas scroll when their host content overflows.
use crate::v2::theme::METRICS;
use crate::v2::view::layout::{BAreaLayout, BAreaSize};

impl BAreaLayout for BAreaType {
    fn size(&self) -> BAreaSize {
        match self {
            BAreaType::Top => BAreaSize {
                width: None,
                height: Some(METRICS.h_header),
            },
            BAreaType::Bottom => BAreaSize {
                width: None,
                height: Some(METRICS.h_bottom_strip),
            },
            // Left/Right are content- AND zone-sized: the host's side regions
            // (UI panel = METRICS.w_panel, strips = w_toolbar / w_navbar …)
            // dictate the width, and the zone seam may override it at drag
            // time. Pinning `w_area_left/right` here pinned the zone NARROWER
            // than the panel metric (196 < 240) and clipped the Buts panel
            // after side regions started stretching (`min_w + w_full`).
            BAreaType::Left | BAreaType::Right => BAreaSize {
                width: None,
                height: None,
            },
            BAreaType::Center => BAreaSize {
                width: None,
                height: None,
            },
        }
    }
    fn is_scrollable(&self) -> bool {
        matches!(self, BAreaType::Left | BAreaType::Right)
    }
}
