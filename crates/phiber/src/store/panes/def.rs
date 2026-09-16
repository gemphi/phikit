//! Pane constants - labels, `ALL`, and `LABELS`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPaneType {
    Left,
    Center,
    Right,
}

pub const LEFT_LABEL: &str = "Left";
pub const CENTER_LABEL: &str = "Center";
pub const RIGHT_LABEL: &str = "Right";
