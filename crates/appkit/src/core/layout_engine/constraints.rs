//! # Layout Constraints
//!
//! Sizing primitives for the declarative layout engine.
//! Defines how views express their size preferences and how
//! the flexbox resolver interprets them.

use crate::graphics::NSRect;

/// Edge insets for padding/margin around a view node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edges {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Edges {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }

    pub fn zero() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }

    /// Total horizontal inset (left + right).
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Total vertical inset (top + bottom).
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

impl Default for Edges {
    fn default() -> Self {
        Self::zero()
    }
}

/// How a single dimension (width or height) is sized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeDimension {
    /// Exact pixel value.
    Fixed(f32),
    /// Fills remaining space equally with other Flexible siblings.
    Flexible,
    /// Takes a fraction (0.0–1.0) of the parent's available space.
    Fraction(f32),
    /// Has a minimum size but can grow if space is available.
    Min(f32),
}

impl Default for SizeDimension {
    fn default() -> Self {
        SizeDimension::Flexible
    }
}

/// Size specification for a view node (width × height).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: SizeDimension,
    pub height: SizeDimension,
}

impl Size {
    pub fn new(width: SizeDimension, height: SizeDimension) -> Self {
        Self { width, height }
    }

    pub fn fixed(width: f32, height: f32) -> Self {
        Self {
            width: SizeDimension::Fixed(width),
            height: SizeDimension::Fixed(height),
        }
    }

    pub fn flexible() -> Self {
        Self {
            width: SizeDimension::Flexible,
            height: SizeDimension::Flexible,
        }
    }

    pub fn fill_width(height: SizeDimension) -> Self {
        Self {
            width: SizeDimension::Flexible,
            height,
        }
    }

    pub fn fill_height(width: SizeDimension) -> Self {
        Self {
            width,
            height: SizeDimension::Flexible,
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::flexible()
    }
}

/// A unique identifier for a resolved view frame.
pub type ViewId = u64;

/// The output of the layout engine: a flat list of absolute frames
/// keyed by ViewId, ready for rendering.
#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub frames: Vec<(ViewId, NSRect)>,
}

impl LayoutResult {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    pub fn push(&mut self, id: ViewId, frame: NSRect) {
        self.frames.push((id, frame));
    }

    /// Look up the frame for a given ViewId.
    pub fn frame_for(&self, id: ViewId) -> Option<NSRect> {
        self.frames
            .iter()
            .find(|(vid, _)| *vid == id)
            .map(|(_, f)| *f)
    }

    /// Total number of resolved frames.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

impl Default for LayoutResult {
    fn default() -> Self {
        Self::new()
    }
}
