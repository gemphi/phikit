//! `BAreaType` — position tag inside a zone.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BAreaType {
    Top,
    Left,
    Center,
    Right,
    Bottom,
}

impl BAreaType {
    pub fn label(self) -> &'static str {
        match self {
            BAreaType::Top => "Top",
            BAreaType::Left => "Left",
            BAreaType::Center => "Center",
            BAreaType::Right => "Right",
            BAreaType::Bottom => "Bottom",
        }
    }
}
