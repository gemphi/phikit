//! Drivers host definitions matching Blender 5.2 `SpaceGraph` (Drivers mode) in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct DriversConfig {
    pub show_only_selected: bool,
    pub show_hidden: bool,
}

impl Default for DriversConfig {
    fn default() -> Self {
        Self {
            show_only_selected: true,
            show_hidden: false,
        }
    }
}
