/// Whether a descendant paints pixels over its enclosing surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BSurfaceOpacity {
    #[default]
    Transparent,
    Opaque,
}

impl BSurfaceOpacity {
    pub const fn is_opaque(self) -> bool {
        matches!(self, Self::Opaque)
    }
}

/// Position of one row in a vertical composition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BRowPosition {
    first: bool,
    last: bool,
}

impl BRowPosition {
    pub const fn at(index: usize, count: usize) -> Self {
        Self {
            first: count > 0 && index == 0,
            last: count > 0 && index < count && index + 1 == count,
        }
    }

    const fn from_edges(first: bool, last: bool) -> Self {
        Self { first, last }
    }
}

/// Position of one cell in a row/column composition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BCellPosition {
    row: BRowPosition,
    leftmost: bool,
    rightmost: bool,
}

impl BCellPosition {
    pub const fn at(row: usize, rows: usize, column: usize, columns: usize) -> Self {
        Self {
            row: BRowPosition::at(row, rows),
            leftmost: columns > 0 && column == 0,
            rightmost: columns > 0 && column < columns && column + 1 == columns,
        }
    }

    const fn from_edges(first: bool, last: bool, leftmost: bool, rightmost: bool) -> Self {
        Self {
            row: BRowPosition::from_edges(first, last),
            leftmost,
            rightmost,
        }
    }
}

/// Corners of an enclosing surface that an element may paint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BCorners {
    pub tl: bool,
    pub tr: bool,
    pub bl: bool,
    pub br: bool,
}

impl BCorners {
    pub const NONE: Self = Self::new(false, false, false, false);
    pub const ALL: Self = Self::new(true, true, true, true);

    pub const fn new(tl: bool, tr: bool, bl: bool, br: bool) -> Self {
        Self { tl, tr, bl, br }
    }

    pub const fn any(self) -> bool {
        self.tl || self.tr || self.bl || self.br
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self::new(
            self.tl && other.tl,
            self.tr && other.tr,
            self.bl && other.bl,
            self.br && other.br,
        )
    }

    pub const fn without(self, claimed: Self) -> Self {
        Self::new(
            self.tl && !claimed.tl,
            self.tr && !claimed.tr,
            self.bl && !claimed.bl,
            self.br && !claimed.br,
        )
    }

    pub const fn assign_row(
        self,
        position: BRowPosition,
        opacity: BSurfaceOpacity,
    ) -> BCornerOwnership {
        self.assign(
            Self::new(position.first, position.first, position.last, position.last),
            opacity,
        )
    }

    pub const fn assign_cell(
        self,
        position: BCellPosition,
        opacity: BSurfaceOpacity,
    ) -> BCornerOwnership {
        self.assign(
            Self::new(
                position.row.first && position.leftmost,
                position.row.first && position.rightmost,
                position.row.last && position.leftmost,
                position.row.last && position.rightmost,
            ),
            opacity,
        )
    }

    const fn assign(self, candidate: Self, opacity: BSurfaceOpacity) -> BCornerOwnership {
        let child = if opacity.is_opaque() {
            self.intersection(candidate)
        } else {
            Self::NONE
        };
        BCornerOwnership {
            surface: self.without(child),
            child,
        }
    }

    /// Compatibility constructor for existing host renderers.
    pub const fn row(first: bool, last: bool) -> Self {
        Self::ALL
            .assign_row(
                BRowPosition::from_edges(first, last),
                BSurfaceOpacity::Opaque,
            )
            .child
    }

    /// Compatibility constructor for existing host renderers.
    pub const fn cell(first: bool, last: bool, leftmost: bool, rightmost: bool) -> Self {
        Self::ALL
            .assign_cell(
                BCellPosition::from_edges(first, last, leftmost, rightmost),
                BSurfaceOpacity::Opaque,
            )
            .child
    }
}

/// Exclusive ownership after considering one child at a composition level.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BCornerOwnership {
    pub surface: BCorners,
    pub child: BCorners,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_assignment_intersects_inherited_corners() {
        let inherited = BCorners::new(true, false, true, true);
        let first = inherited.assign_row(BRowPosition::at(0, 3), BSurfaceOpacity::Opaque);
        assert_eq!(first.child, BCorners::new(true, false, false, false));
        assert_eq!(first.surface, BCorners::new(false, false, true, true));
        assert_eq!(
            inherited
                .assign_row(BRowPosition::at(1, 3), BSurfaceOpacity::Opaque)
                .child,
            BCorners::NONE
        );
    }

    #[test]
    fn transparent_cell_leaves_every_corner_with_surface() {
        let ownership =
            BCorners::ALL.assign_cell(BCellPosition::at(0, 1, 0, 1), BSurfaceOpacity::Transparent);
        assert_eq!(ownership.child, BCorners::NONE);
        assert_eq!(ownership.surface, BCorners::ALL);
    }
}
