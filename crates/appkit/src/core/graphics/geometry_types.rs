//! Geometry primitives - points, sizes, rects, and edge insets.

/// Coordinate point
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NSPoint {
    pub x: f32,
    pub y: f32,
}

impl NSPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }
}

/// Coordinate size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NSSize {
    pub width: f32,
    pub height: f32,
}

impl NSSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Coordinate rect
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NSRect {
    pub origin: NSPoint,
    pub size: NSSize,
}

impl NSRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: NSPoint::new(x, y),
            size: NSSize::new(width, height),
        }
    }

    /// Creates a rect at the origin (0, 0) with the given size.
    /// Use when position will be set later by a layout system or parent view.
    pub fn sized(width: f32, height: f32) -> Self {
        Self {
            origin: NSPoint::zero(),
            size: NSSize::new(width, height),
        }
    }

    /// Creates a square rect at the origin (0, 0).
    pub fn square(side: f32) -> Self {
        Self::sized(side, side)
    }

    /// Creates a rect from an NSSize at the origin (0, 0).
    pub fn from_size(size: NSSize) -> Self {
        Self {
            origin: NSPoint::zero(),
            size,
        }
    }

    pub fn zero() -> Self {
        Self {
            origin: NSPoint::zero(),
            size: NSSize::zero(),
        }
    }
    pub fn width(&self) -> f32 {
        self.size.width
    }
    pub fn height(&self) -> f32 {
        self.size.height
    }
    pub fn x(&self) -> f32 {
        self.origin.x
    }
    pub fn y(&self) -> f32 {
        self.origin.y
    }
    pub fn min_x(&self) -> f32 {
        self.origin.x
    }
    pub fn min_y(&self) -> f32 {
        self.origin.y
    }
    pub fn max_x(&self) -> f32 {
        self.origin.x + self.size.width
    }
    pub fn max_y(&self) -> f32 {
        self.origin.y + self.size.height
    }
    pub fn mid_x(&self) -> f32 {
        self.origin.x + self.size.width / 2.0
    }
    pub fn mid_y(&self) -> f32 {
        self.origin.y + self.size.height / 2.0
    }
    pub fn contains_point(&self, p: &NSPoint) -> bool {
        p.x >= self.origin.x && p.x <= self.max_x() && p.y >= self.origin.y && p.y <= self.max_y()
    }

    pub fn offset(&self, dx: f32, dy: f32) -> Self {
        Self::new(
            self.origin.x + dx,
            self.origin.y + dy,
            self.size.width,
            self.size.height,
        )
    }

    pub fn intersection(&self, other: &NSRect) -> Self {
        let x = self.origin.x.max(other.origin.x);
        let y = self.origin.y.max(other.origin.y);
        let max_x = self.max_x().min(other.max_x());
        let max_y = self.max_y().min(other.max_y());

        if max_x >= x && max_y >= y {
            Self::new(x, y, max_x - x, max_y - y)
        } else {
            Self::zero()
        }
    }

    // Layout helpers - eliminates repetitive calculations
    /// Creates rect with uniform padding from parent edges
    pub fn inset_by(&self, padding: f32) -> Self {
        Self::new(
            self.origin.x + padding,
            self.origin.y + padding,
            self.size.width - padding * 2.0,
            self.size.height - padding * 2.0,
        )
    }

    /// Creates full-width row at specific Y with padding
    pub fn row_at(&self, y: f32, height: f32, padding: f32) -> Self {
        Self::new(
            self.origin.x + padding,
            y,
            self.size.width - padding * 2.0,
            height,
        )
    }

    /// Creates full-height column at specific X with padding
    pub fn column_at(&self, x: f32, width: f32, padding: f32) -> Self {
        Self::new(
            x,
            self.origin.y + padding,
            width,
            self.size.height - padding * 2.0,
        )
    }
}

/// Edge insets
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NSEdgeInsets {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}

impl NSEdgeInsets {
    pub fn new(top: f32, left: f32, bottom: f32, right: f32) -> Self {
        Self {
            top,
            left,
            bottom,
            right,
        }
    }

    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            left: value,
            bottom: value,
            right: value,
        }
    }
}
