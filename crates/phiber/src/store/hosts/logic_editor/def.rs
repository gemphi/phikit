//! Logic Editor host definitions matching Blender 5.2 `SpaceLogic` in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct LogicEditorConfig {
    pub show_sensors: bool,
    pub show_controllers: bool,
    pub show_actuators: bool,
}

impl Default for LogicEditorConfig {
    fn default() -> Self {
        Self {
            show_sensors: true,
            show_controllers: true,
            show_actuators: true,
        }
    }
}
