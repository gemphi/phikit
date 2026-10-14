//! Zone — a grid slot inside a space.

pub mod set;
pub mod size;
pub mod visibility;

use crate::v2::area::BAreaList;
use crate::v2::area::BAreaSpec;
use crate::v2::space::BSpaceSpec;
use crate::v2::spec::BSpec;
use crate::v2::spec::BSpecKind;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
pub use set::SET;
pub use size::apply_zone_drag;
pub use size::is_resizable as is_zone_resizable;
pub use size::reset_zone_size;
pub use size::zone_size;
pub use size::zone_size_key;

// `BZoneType` — grid position tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BZoneType {
    Top,
    Left,
    Center,
    Right,
    Bottom,
    Full,
    /// The activity rail: a fixed strip down the window's far left, in every
    /// space, never toggled and never dragged.
    Rail,
}

impl BZoneType {
    pub const ALL: &'static [BZoneType] = &[
        BZoneType::Top,
        BZoneType::Left,
        BZoneType::Center,
        BZoneType::Right,
        BZoneType::Bottom,
        BZoneType::Full,
        BZoneType::Rail,
    ];

    /// Zones in the order they are laid out: the top strip, then the middle
    /// band left to right, then the bottom strip.
    pub const IN_ORDER: &'static [BZoneType] = &[
        BZoneType::Top,
        BZoneType::Rail,
        BZoneType::Left,
        BZoneType::Center,
        BZoneType::Right,
        BZoneType::Full,
        BZoneType::Bottom,
    ];

    pub fn spec(self) -> &'static self::BZoneSpec {
        &self::SET[self as usize]
    }
    pub fn label(self) -> &'static str {
        self.spec().label
    }
    pub fn icon(self) -> &'static str {
        self.spec().icon
    }
    pub fn tooltip(self) -> &'static str {
        self.spec().tooltip
    }
    pub fn is_toggleable(self) -> bool {
        self.spec().toggleable
    }

    /// Stable lowercase slug used in state-key paths (Requirement 54.5).
    pub fn slug(self) -> &'static str {
        match self {
            BZoneType::Top => "top",
            BZoneType::Left => "left",
            BZoneType::Center => "center",
            BZoneType::Right => "right",
            BZoneType::Bottom => "bottom",
            BZoneType::Full => "full",
            BZoneType::Rail => "rail",
        }
    }

    /// Exact inverse of [`BZoneType::slug`]. Returns `None` for unknown input.
    pub fn from_slug(s: &str) -> Option<BZoneType> {
        match s {
            "top" => Some(BZoneType::Top),
            "left" => Some(BZoneType::Left),
            "center" => Some(BZoneType::Center),
            "right" => Some(BZoneType::Right),
            "bottom" => Some(BZoneType::Bottom),
            "full" => Some(BZoneType::Full),
            "rail" => Some(BZoneType::Rail),
            _ => None,
        }
    }
}

// `BZoneSpec` — static per-zone data.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BZoneSpec {
    pub kind: BZoneType,
    pub label: &'static str,
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub toggleable: bool,
}

impl BSpec for BZoneSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Zone(self.kind)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

// `BZoneItem` — one zone. Visibility gate + delegate to `BAreaList`.

#[derive(Clone, Debug)]
pub struct BZoneItem<'a> {
    pub zone: BZoneType,
    pub areas: &'a [BAreaSpec],
}

impl<'a> BZoneItem<'a> {
    pub fn new(zone: BZoneType, areas: &'a [BAreaSpec]) -> Self {
        Self { zone, areas }
    }

    fn is_visible(&self, state: &BState) -> bool {
        self::visibility::is_visible(state, self.zone)
    }
}

impl<'a> BViewItem for BZoneItem<'a> {
    fn build(&self, state: &BState) -> BView {
        if self.areas.is_empty() || !self.is_visible(state) {
            return BView::empty();
        }
        BView::Zone {
            kind: self.zone,
            list: BAreaList::new(self.areas).items(state),
        }
    }
}

// `BZoneList` — iterate the zones of a space.

#[derive(Clone, Copy, Debug)]
pub struct BZoneList<'a> {
    pub areas: &'a [BAreaSpec],
}

impl<'a> BZoneList<'a> {
    pub fn new(space: &'a BSpaceSpec) -> Self {
        Self { areas: space.areas }
    }

    /// The zones of any declared area set — a space's, or a dialog's.
    pub fn of(areas: &'a [BAreaSpec]) -> Self {
        Self { areas }
    }
}

impl<'a> BViewList for BZoneList<'a> {
    fn items(&self, state: &BState) -> Vec<BView> {
        // Ctrl Space: one area fills the workspace (`area::maximize`) — it is
        // drawn alone, in the zone that fills, whichever zone declared it.
        if let Some(label) = crate::v2::area::current::current_space()
            .and_then(|space| crate::v2::area::maximize::maximized_in(state, space))
            && let Some(area) = self.areas.iter().find(|a| a.kind.label() == label)
        {
            return vec![BZoneItem::new(BZoneType::Center, std::slice::from_ref(area)).build(state)];
        }
        // Zones in screen order, left to right across the middle band: the
        // rail leads. (`ALL` is discriminant order, which the spec table is
        // indexed by, so the rail sits at its end there.)
        BZoneType::IN_ORDER
            .iter()
            .map(|&z| BZoneItem::new(z, areas_in(self.areas, z)).build(state))
            .filter(|v| !v.is_empty())
            .collect()
    }
}

/// The contiguous run of `areas` declared in `zone` (area tables are grouped
/// by zone).
pub fn areas_in(areas: &[BAreaSpec], zone: BZoneType) -> &[BAreaSpec] {
    let mut start = None;
    let mut end = 0usize;
    for (i, a) in areas.iter().enumerate() {
        if a.zone == zone {
            if start.is_none() {
                start = Some(i);
            }
            end = i + 1;
        }
    }
    match start {
        Some(s) => &areas[s..end],
        None => &[],
    }
}

// ── Layout (was `view/render/zone.rs`) ──
// Zone-level layout: `BZoneType` decodes into orientation.
// 
// - `Top` / `Bottom` — horizontal strips whose areas stack vertically.
// - `Left` / `Center` / `Right` — full-height columns of stacked editors.
// - `Full` — horizontal row of side-by-side areas.
// 
// Heights and widths are `BFlex` decisions (`gpui/zone.rs`), read from the
// resolved theme.
use crate::v2::view::layout::{BAxis, BZoneLayout};

impl BZoneLayout for BZoneType {
    fn axis(&self) -> BAxis {
        match self {
            BZoneType::Top | BZoneType::Bottom => BAxis::Column,
            BZoneType::Left | BZoneType::Center | BZoneType::Right | BZoneType::Rail => {
                BAxis::Column
            }
            BZoneType::Full => BAxis::Row,
        }
    }
}
