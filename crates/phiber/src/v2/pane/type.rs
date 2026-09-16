//! `BPaneType` — pane slot inside a region.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPaneType {
    Left,
    Center,
    Right,
}

impl BPaneType {
    pub fn label(self) -> &'static str {
        match self {
            BPaneType::Left => "Left",
            BPaneType::Center => "Center",
            BPaneType::Right => "Right",
        }
    }
}
