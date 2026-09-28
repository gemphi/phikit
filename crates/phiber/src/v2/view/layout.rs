//! Layout vocabulary and the renderer seam (was `view/render/mod.rs`).
//! `BRenderer` turns store-level `BView` nodes into output: `view::text`
//! (`BAsciiRenderer`) for headless walks and tests, `view::draw`
//! (`BGpuiRenderer`) for pixels. Each level implements its `B…Layout`
//! trait in its own file (`zone.rs`: `impl BZoneLayout for BZoneType`).
//!
//! One per-level module owns that level's layout decisions — the ONLY place
//! that decodes `BZoneType::Bottom → column`, `BRegionType::UI → right`, etc.
//! Every number is read from `theme::METRICS`; there are no inline literals.



use super::BView;
use crate::v2::state::BState;


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

/// Structural traits: which way a level stacks its children and, for a
/// region, which band of its host it sits in. They answer composition, not
/// styling — how a container is sized, padded and painted is `BFlex`, answered
/// by each level's `BFlexed` impl in the GPUI renderer. (The size answers these
/// traits used to carry read the static `METRICS` defaults, had no callers, and
/// disagreed with the resolved sizes actually drawn; they are gone.)
pub trait BSpaceLayout {
    fn axis(&self) -> BAxis;
}
pub trait BZoneLayout {
    fn axis(&self) -> BAxis;
}
pub trait BAreaLayout {
    fn size(&self) -> BAreaSize;
    fn is_scrollable(&self) -> bool;
}
pub trait BHostLayout {
    fn axis(&self) -> BAxis;
}
/// The pane trait lives with the pane (`pane.rs`); named here with the rest.
pub use crate::v2::pane::BPaneLayout;

pub trait BRegionLayout {
    fn axis(&self) -> BAxis;
    fn placement(&self) -> BPlacement;
}

/// A rendering backend. `render` walks the tree top-down; each backend owns
/// its own output type (GPUI element, ASCII string, DOM node, etc.).
pub trait BRenderer {
    type Output;
    fn render(&self, view: &BView, state: &BState) -> Self::Output;
}
