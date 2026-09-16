//! `BZoneType` — grid position tag.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BZoneType {
    Top,
    Left,
    Center,
    Right,
    Bottom,
    Full,
}

impl BZoneType {
    pub const ALL: &'static [BZoneType] = &[
        BZoneType::Top,
        BZoneType::Left,
        BZoneType::Center,
        BZoneType::Right,
        BZoneType::Bottom,
        BZoneType::Full,
    ];

    pub fn spec(self) -> &'static super::BZoneSpec {
        &super::SET[self as usize]
    }
    pub fn label(self) -> &'static str {
        self.spec().label
    }
    pub fn icon(self) -> &'static str {
        self.spec().icon
    }
    pub fn tooltip(self) -> &'static str {
        self.spec().tooltip
    }
    pub fn is_toggleable(self) -> bool {
        self.spec().toggleable
    }

    /// Stable lowercase slug used in state-key paths (Requirement 54.5).
    pub fn slug(self) -> &'static str {
        match self {
            BZoneType::Top => "top",
            BZoneType::Left => "left",
            BZoneType::Center => "center",
            BZoneType::Right => "right",
            BZoneType::Bottom => "bottom",
            BZoneType::Full => "full",
        }
    }

    /// Exact inverse of [`BZoneType::slug`]. Returns `None` for unknown input.
    pub fn from_slug(s: &str) -> Option<BZoneType> {
        match s {
            "top" => Some(BZoneType::Top),
            "left" => Some(BZoneType::Left),
            "center" => Some(BZoneType::Center),
            "right" => Some(BZoneType::Right),
            "bottom" => Some(BZoneType::Bottom),
            "full" => Some(BZoneType::Full),
            _ => None,
        }
    }
}
