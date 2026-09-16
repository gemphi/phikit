//! Non-Linear Animation host definitions matching Blender 5.2 `SpaceNla` in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct NlaConfig {
    pub show_strip_curves: bool,
    pub show_track_curves: bool,
}

impl Default for NlaConfig {
    fn default() -> Self {
        Self {
            show_strip_curves: false,
            show_track_curves: true,
        }
    }
}
