//! What the file browser was opened for. Blender stores this as
//! `FileSelectParams::title`, and the execute button shows it verbatim.

use super::keys;
use crate::v2::state::BState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BFilePurpose {
    Open,
    SaveAs,
    Import,
    Export,
    Link,
    Append,
}

impl BFilePurpose {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::SaveAs => "save_as",
            Self::Import => "import",
            Self::Export => "export",
            Self::Link => "link",
            Self::Append => "append",
        }
    }
    /// Modal title.
    pub fn title(self) -> &'static str {
        match self {
            Self::Open => "Open Blender File",
            Self::SaveAs => "Save Blender File",
            Self::Import => "Import",
            Self::Export => "Export",
            Self::Link => "Link",
            Self::Append => "Append",
        }
    }
    /// Execute-button label.
    pub fn exec_label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::SaveAs => "Save As",
            Self::Import => "Import",
            Self::Export => "Export",
            Self::Link => "Link",
            Self::Append => "Append",
        }
    }
    pub fn from_slug(s: &str) -> Option<Self> {
        [
            Self::Open,
            Self::SaveAs,
            Self::Import,
            Self::Export,
            Self::Link,
            Self::Append,
        ]
        .into_iter()
        .find(|p| p.slug() == s)
    }
}

pub fn purpose(s: &BState) -> BFilePurpose {
    s.get(&keys::k_purpose())
        .and_then(BFilePurpose::from_slug)
        .unwrap_or(BFilePurpose::Open)
}
