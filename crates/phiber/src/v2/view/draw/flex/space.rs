//! The padding and margin half of `BFlex`'s builder — every inset a
//! container declares, each one a `BSpace` the theme resolves.
//!
//! Split out of `element.rs` for the file cap; `element.rs` keeps the record
//! itself and how it takes space, `surface.rs` what it paints.

use super::element::BFlex;
use super::token::BSpace;

impl BFlex {
    pub const fn pad(self, space: BSpace) -> Self {
        self.pad_x(space).pad_y(space)
    }

    pub const fn pad_x(mut self, space: BSpace) -> Self {
        self.pad_left = Some(space);
        self.pad_right = Some(space);
        self
    }

    pub const fn pad_y(mut self, space: BSpace) -> Self {
        self.pad_top = Some(space);
        self.pad_bottom = Some(space);
        self
    }

    pub const fn pad_top(mut self, space: BSpace) -> Self {
        self.pad_top = Some(space);
        self
    }

    pub const fn pad_right(mut self, space: BSpace) -> Self {
        self.pad_right = Some(space);
        self
    }

    pub const fn pad_left(mut self, space: BSpace) -> Self {
        self.pad_left = Some(space);
        self
    }

    pub const fn pad_bottom(mut self, space: BSpace) -> Self {
        self.pad_bottom = Some(space);
        self
    }

    pub const fn margin_x(mut self, space: BSpace) -> Self {
        self.margin_left = Some(space);
        self.margin_right = Some(space);
        self
    }

    pub const fn margin_y(mut self, space: BSpace) -> Self {
        self.margin_top = Some(space);
        self.margin_bottom = Some(space);
        self
    }

    pub const fn margin_left(mut self, space: BSpace) -> Self {
        self.margin_left = Some(space);
        self
    }

    pub const fn raised_by(mut self, space: BSpace) -> Self {
        self.raised_by = Some(space);
        self
    }
}
