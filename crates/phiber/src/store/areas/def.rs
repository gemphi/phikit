//! Default area type constants and helpers.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BAreaType {
    Top,
    Left,
    Right,
    Center,
    Bottom,
}

pub const TOP_LABEL: &str = "Top";
pub const LEFT_LABEL: &str = "Left";
pub const RIGHT_LABEL: &str = "Right";
pub const CENTER_LABEL: &str = "Center";
pub const BOTTOM_LABEL: &str = "Bottom";

impl BAreaType {
    /// CSS-like role key for `styled_panel` in the bui layer.
    pub fn role(self) -> &'static str {
        match self {
            BAreaType::Top => "top",
            BAreaType::Left => "left",
            BAreaType::Right => "right",
            BAreaType::Center => "center",
            BAreaType::Bottom => "bottom",
        }
    }

    /// Typed panel role for the BUI layer.
    pub fn panel_role(self) -> crate::store::BPanelRole {
        crate::store::BPanelRole::from_area(self)
    }
}
