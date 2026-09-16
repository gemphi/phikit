//! Dope Sheet host definitions matching Blender 5.2 `SpaceAction` in `DNA_space_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DopeSheetMode {
    DopeSheet = 0,
    ActionEditor = 1,
    ShapeKeyEditor = 2,
    GreasePencil = 3,
    Mask = 4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DopeSheetConfig {
    pub mode: DopeSheetMode,
    pub action_name: Option<String>,
    pub show_only_selected: bool,
}

impl Default for DopeSheetConfig {
    fn default() -> Self {
        Self {
            mode: DopeSheetMode::DopeSheet,
            action_name: None,
            show_only_selected: true,
        }
    }
}
