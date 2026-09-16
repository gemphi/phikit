//! Zone constants - labels and zone labels only.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BZoneType {
    Full,
    Top,
    Left,
    Right,
    Center,
    Bottom,
}

pub const FULL_LABEL: &str = "Full";
pub const TOP_LABEL: &str = "Top";
pub const LEFT_LABEL: &str = "Left";
pub const RIGHT_LABEL: &str = "Right";
pub const CENTER_LABEL: &str = "Center";
pub const BOTTOM_LABEL: &str = "Bottom";
