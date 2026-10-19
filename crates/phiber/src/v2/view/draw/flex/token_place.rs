//! The placement tokens — where a child sits relative to its parent, which
//! edge of a popover anchors it, which element type the token layer builds.
//! Split from `token.rs` so each file sits under the 240-line read cap
//! (`tests/level_structure.rs`).

use super::*;

/// Where a container sits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BPlace {
    /// In the flow, positioning nothing.
    Flow,
    /// In the flow, and the anchor its absolute children position against.
    Anchor,
    /// Over its parent, covering it edge to edge.
    Overlay,
    /// Floating over its parent against one side, inset from that side and
    /// the top by `inset` and from the bottom by `foot` — the viewport's
    /// toolbar and sidebar, whose pinned cluster sits a cap gap off the
    /// bottom edge.
    Float { side: BSide, inset: BSpace, foot: BSpace },
    /// Floating directly above its anchor, flush with the anchor's left and
    /// right edges — the action log's drawer over the status bar.
    Drawer,
    /// At a fraction of its anchor, sized by a fraction of it: a node, a
    /// frame box or a hit target on a node canvas, whose coordinates are the
    /// canvas's own `0..1` space (`canvas::node::view::BNodeView`). A canvas
    /// is the one place a box's position IS data rather than a token, so the
    /// numbers come through the place token instead of being written on the
    /// element as raw GPUI style.
    Fraction { x: f32, y: f32, w: f32, h: f32 },
}

/// A side of the parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BSide {
    Left,
    Right,
}


/// Text defaults for everything inside.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BType {
    /// Body text — a panel title (Blender's `paneltitle` is the widget size).
    Body,
    Header,
    Caption,
    /// Reading text — a modal's prose (`draw::prose`): the body size scaled
    /// up for a paragraph, since 11 px is a label's size, not a page's.
    Prose,
    /// A heading over reading text.
    ProseHeading,
    /// The title of a page of reading text.
    ProseTitle,
}

/// How far reading text is set over the chrome's sizes: a paragraph at a
/// quarter over the body, a heading at a third over the header, a title at
/// a half over it — so the hierarchy reads at a glance and every size still
/// follows the Typography preferences. Before this a paragraph had NO size
/// and took the element default (16 px) while its title took the 12 px
/// header, so the title was the smallest text on the page.
pub const PROSE_SCALE: f32 = 1.25;
pub const PROSE_HEADING_SCALE: f32 = 1.35;
pub const PROSE_TITLE_SCALE: f32 = 1.6;
