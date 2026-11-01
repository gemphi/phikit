//! Layout enums and autoresizing masks for UI layout.

bitflags::bitflags! {
    /// Autoresizing masks for responsive layout
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct NSAutoresizingMask: u32 {
        const NONE = 0;
        const MIN_X_MARGIN = 1 << 0;
        const WIDTH_SIZABLE = 1 << 1;
        const MAX_X_MARGIN = 1 << 2;
        const MIN_Y_MARGIN = 1 << 3;
        const HEIGHT_SIZABLE = 1 << 4;
        const MAX_Y_MARGIN = 1 << 5;
    }
}

/// User interface layout orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSUserInterfaceLayoutOrientation {
    Horizontal,
    Vertical,
}

/// Layout attributes for constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSLayoutAttribute {
    Leading,
    Trailing,
    Top,
    Bottom,
    Width,
    Height,
    CenterX,
    CenterY,
    Baseline,
    Fill,
    NotAnAttribute,
}

/// Layout relations for constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSLayoutRelation {
    LessThanOrEqual,
    Equal,
    GreaterThanOrEqual,
}
