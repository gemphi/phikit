//! `BRenderer` — the single seam that turns store-level `BView` nodes into
//! pixels. Backend-agnostic: `BAsciiRenderer` is included for headless walks
//! and tests; a GPUI backend plugs in later without touching the tree.
//!
//! One per-level module owns that level's layout decisions — the ONLY place
//! that decodes `BZoneType::Bottom → column`, `BRegionType::UI → right`, etc.
//! Every number is read from `theme::METRICS`; there are no inline literals.

pub mod area;
pub mod ascii;
pub mod gpui;
pub mod host;
pub mod pane;
pub mod region;
pub mod space;
pub mod zone;

use super::BView;
use crate::v2::state::BState;

pub use ascii::BAsciiRenderer;
pub use gpui::BGpuiRenderer;

/// Layout axis a container distributes its children along.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BAxis {
    Row,
    Column,
}

/// Vertical / horizontal alignment for a region inside its host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BPlacement {
    Top,
    Bottom,
    Left,
    Right,
    Flex,
}

/// Fixed dimensions applied to an area. `None` means flex on that axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAreaSize {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

/// Layout trait: every store-level enum implements the one relevant to it.
/// Traits are per-level because Space/Host don't have `fixed_size`, Areas
/// have both width+height, Regions have alignment.
pub trait BSpaceLayout {
    fn axis(&self) -> BAxis;
}
pub trait BZoneLayout {
    fn axis(&self) -> BAxis;
    fn fixed_height(&self) -> Option<f32>;
}
pub trait BAreaLayout {
    fn size(&self) -> BAreaSize;
    fn is_scrollable(&self) -> bool;
}
pub trait BHostLayout {
    fn axis(&self) -> BAxis;
}
pub trait BRegionLayout {
    fn axis(&self) -> BAxis;
    fn placement(&self) -> BPlacement;
    fn fixed_size(&self) -> Option<f32>;
}
pub use pane::BPaneLayout;

/// A rendering backend. `render` walks the tree top-down; each backend owns
/// its own output type (GPUI element, ASCII string, DOM node, etc.).
pub trait BRenderer {
    type Output;
    fn render(&self, view: &BView, state: &BState) -> Self::Output;
}
