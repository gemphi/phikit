//! Topbar host definitions matching Blender 5.2 `SpaceTopBar` in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TopbarConfig {
    pub show_menus: bool,
    pub show_workspaces: bool,
    pub show_scene_selector: bool,
    pub show_view_layer_selector: bool,
}
