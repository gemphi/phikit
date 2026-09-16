//! Unified view tree — one `BViewBuilder` trait behind every view.
//!
//! Mirrors the store hierarchy 1-to-1. A view is a folder when it has parts,
//! a single file otherwise. Each builder holds only its parameters (host,
//! region, area, …); runtime `BState` is passed into `build`.
//!
//!   views/space/   — SpaceView   (store/spaces/)
//!   views/zone/    — ZoneView    (store/zones/)
//!   views/area/    — AreaView    (store/areas/)
//!   views/host/    — HostView    (store/hosts/)
//!   views/region/  — RegionView  (store/regions/)
//!   views/pane/    — PaneView    (store/panes/)
//!   views/part/    — PartView    (store/parts/) — dispatches to concrete part views
//!
//! Leaf rendering stays in `bui/core/`; the `BView` tree node stays in
//! `bui/bview.rs`; tree -> GPUI stays in `bui/render.rs`.

pub mod area;
pub mod host;
pub mod pane;
pub mod part;
pub mod region;
pub mod space;
pub mod zone;

pub use area::AreaView;
pub use host::HostView;
pub use pane::{PaneView, PartContainer};
pub use part::PartView;
pub use region::RegionView;
pub use space::SpaceView;
pub use zone::ZoneView;

use super::bview::BView;
use super::state::BState;

/// Every view in the tree builds itself into a `BView` node from runtime state.
///
/// Object-safe and lifetime-free: parameters live on the struct, `state` is an
/// argument. Implement this at every level (space → zone → area → host →
/// region → pane → part).
pub trait BViewBuilder {
    /// Build the declarative node for this view from runtime state.
    fn build(&self, state: &BState) -> BView;
}
