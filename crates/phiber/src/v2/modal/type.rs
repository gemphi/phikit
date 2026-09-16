//! `BModalType` — every overlay dialog that can be opened.
//! Opens by writing its slug to `ctx.modal`; closes by clearing the key.
//!
//! `FileBrowser` is the same underlying widget as the `File` host — Blender
//! reuses `space_file` for both the workspace editor and the modal dialog
//! (Open / Save As / Import / Export). The overlay flavour reads
//! `ctx.file.purpose` for its title (Open Blender File / Save As / …).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BModalType {
    Settings,
    Help,
    Profile,
    Command,
    Splash,
    About,
    FileBrowser,
}

impl BModalType {
    pub const ALL: &'static [BModalType] = &[
        Self::Settings,
        Self::Help,
        Self::Profile,
        Self::Command,
        Self::Splash,
        Self::About,
        Self::FileBrowser,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Settings => "settings",
            Self::Help => "help",
            Self::Profile => "profile",
            Self::Command => "command",
            Self::Splash => "splash",
            Self::About => "about",
            Self::FileBrowser => "file_browser",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|m| m.slug() == s)
    }

    pub fn spec(self) -> &'static super::BModalSpec {
        &super::SET[self as usize]
    }
    pub fn label(self) -> &'static str {
        self.spec().label
    }
    pub fn icon(self) -> &'static str {
        self.spec().icon
    }
}
