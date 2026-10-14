//! The areas' keys — which editor a slot shows, its share of its zone, and
//! the drag in flight, under `ctx.area.*`, `ctx.area_size.*`,
//! `ctx.switcher.*` and `ctx.area_resize.*`.

use crate::v2::space::BSpaceType;
use crate::v2::state::BPath;
use crate::v2::zone::BZoneType;
use super::BKey;

/// Area indices are small; a zone holds at most this many.
const SLOTS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AreaKey {
    /// The editor an area slot shows: `ctx.area.<scope>.<slot>.<default host>`.
    /// The scope is the space or dialog that owns the slot (`area::BScope`).
    Host(&'static str, &'static str, &'static str),
    /// Whether that slot's editor-type popover is open.
    Switcher(&'static str, &'static str, &'static str),
    /// Area `index`'s share of its zone: `ctx.area_size.<space>.<zone>.<index>`.
    Size(BSpaceType, BZoneType, usize),
    /// `screen.screen_full_area`: `<space slug>|<area label>` while an area
    /// fills its workspace (`area::maximize`). Not resumed.
    Maximized,
    /// The seam drag in flight: which space, zone and boundary.
    DragSpace,
    DragZone,
    DragIndex,
}

impl AreaKey {
    /// The prefix every `Switcher` key starts with, for closing every open
    /// editor-type popover at once.
    pub fn switcher_prefix() -> String {
        format!("{}.", BPath::owned(vec!["switcher"]).encode())
    }

    /// The prefix every `Size` key of `(space, zone)` starts with, for
    /// sweeping a zone's fractions.
    pub fn size_prefix(space: BSpaceType, zone: BZoneType) -> String {
        format!("{}.", BPath::owned(vec!["area_size", space.slug(), zone.slug()]).encode())
    }
}

impl BKey for AreaKey {
    fn path(&self) -> BPath {
        match self {
            Self::Host(scope, slot, host) => BPath::owned(vec!["area", scope, slot, host]),
            Self::Switcher(scope, slot, host) => BPath::owned(vec!["switcher", scope, slot, host]),
            Self::Size(space, zone, index) => BPath::owned(vec![
                "area_size",
                space.slug(),
                zone.slug(),
                SLOTS[(*index).min(SLOTS.len() - 1)],
            ]),
            Self::Maximized => BPath::owned(vec!["area", "maximized"]),
            Self::DragSpace => BPath::owned(vec!["area_resize", "drag", "space"]),
            Self::DragZone => BPath::owned(vec!["area_resize", "drag", "zone"]),
            Self::DragIndex => BPath::owned(vec!["area_resize", "drag", "index"]),
        }
    }
}
