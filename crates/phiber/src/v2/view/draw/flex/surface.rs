//! The half of `BFlex`'s builder that says what a container paints and where
//! it sits; `element.rs` has how it takes space and arranges its children.

use super::element::BFlex;
use super::token::{BEdge, BFill, BInk, BOverflow, BPlace, BRadius, BType};
use crate::v2::theme::BElevation;

impl BFlex {
    // ── What it paints ──────────────────────────────────────────────────

    pub const fn fill(mut self, fill: BFill) -> Self {
        self.fill = Some(fill);
        self
    }

    pub const fn border(mut self, edge: BEdge) -> Self {
        self.border = Some(edge);
        self
    }

    /// What it paints under the pointer.
    pub const fn hovered(mut self, fill: BFill) -> Self {
        self.hover_fill = Some(fill);
        self
    }

    pub const fn radius(mut self, radius: BRadius) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Sits at `elevation`, and casts what that level casts in this theme.
    pub const fn elevated(mut self, elevation: BElevation) -> Self {
        self.elevation = elevation;
        self
    }

    /// Lifted off the surface beneath but still docked — a card, a drawer.
    pub const fn lifted(self) -> Self {
        self.elevated(BElevation::Raised)
    }

    /// Sets the text colour inside to the theme's dim text — a shortcut, a
    /// caption, anything a step quieter than what it annotates.
    pub const fn dim_ink(mut self) -> Self {
        self.ink = Some(BInk::Dim);
        self
    }

    /// Sets the text colour inside to the theme's text.
    pub const fn ink(mut self) -> Self {
        self.ink = Some(BInk::Text);
        self
    }

    /// Sets the text colour inside to one of the theme's text tones.
    pub const fn ink_tone(mut self, ink: BInk) -> Self {
        self.ink = Some(ink);
        self
    }

    pub const fn type_size(mut self, size: BType) -> Self {
        self.type_size = Some(size);
        self
    }

    // ── Where it sits ───────────────────────────────────────────────────

    pub const fn place(mut self, place: BPlace) -> Self {
        self.place = place;
        self
    }

    pub const fn overflow(mut self, overflow: BOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub const fn occludes(mut self) -> Self {
        self.occludes = true;
        self
    }

    pub const fn dimmed(mut self) -> Self {
        self.dimmed = true;
        self
    }

    /// Make this container the hover group `name`.
    pub const fn hover_group(mut self, name: &'static str) -> Self {
        self.group = Some(name);
        self
    }

    /// Show this container only while the pointer is over the hover group
    /// `name` that encloses it.
    pub const fn revealed_by(mut self, name: &'static str) -> Self {
        self.revealed_by = Some(name);
        self
    }
}
