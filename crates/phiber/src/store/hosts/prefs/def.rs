//! Preferences host definitions matching Blender 5.2 `SpaceUserPref` in `DNA_space_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrefsSection {
    Interface,
    Theme,
    Viewport,
    Lights,
    Editing,
    Animation,
    Addons,
    Input,
    Keymap,
    System,
    SaveLoad,
    FilePaths,
    Experimental,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrefsConfig {
    pub active_section: PrefsSection,
}

impl Default for PrefsConfig {
    fn default() -> Self {
        Self {
            active_section: PrefsSection::Interface,
        }
    }
}
