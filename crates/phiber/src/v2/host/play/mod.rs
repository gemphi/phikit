//! Play host — the game picker: a match chosen set by set (`phiplay`'s
//! sport, teams, kit, shoes, theme, venue, surface and seats) and played
//! into the scene. Structure only: what the host *is*, and where its
//! content lives. The regions are in `set.rs`, the picker in `part::play`.

pub mod set;

use crate::v2::host::{BHostSpec, BHostType};

pub use set::REGIONS;

/// Key used for this host inside a `BPath`.
pub const SLUG: &str = "play";

pub static SPEC: BHostSpec = BHostSpec {
    kind: BHostType::Play,
    label: "Play",
    icon: "game",
    regions: set::REGIONS,
};
