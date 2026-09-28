//! What the file browser was opened for. Blender stores this as
//! `FileSelectParams::title`, and the execute button shows it verbatim.

use crate::v2::state::BKey as _;
use crate::v2::state::FileKey;
use crate::v2::state::BState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BFilePurpose {
    Open,
    SaveAs,
    /// `wm.save_as_mainfile(copy=True)`: written, but the document stays
    /// where it was.
    SaveCopy,
    Import,
    Export,
    Link,
    Append,
    /// `IMAGE_OT_save_as` — write the Render Result to a file. The only
    /// purpose that does not name a `.blend`, so it never touches the
    /// recent-files list.
    SaveImage,
}

impl BFilePurpose {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::SaveAs => "save_as",
            Self::SaveCopy => "save_copy",
            Self::Import => "import",
            Self::Export => "export",
            Self::Link => "link",
            Self::Append => "append",
            Self::SaveImage => "save_image",
        }
    }
    /// Modal title.
    pub fn title(self) -> &'static str {
        match self {
            Self::Open => "Open Blender File",
            Self::SaveAs => "Save Blender File",
            Self::SaveCopy => "Save Copy",
            Self::Import => "Import",
            Self::Export => "Export",
            Self::Link => "Link",
            Self::Append => "Append",
            Self::SaveImage => "Save Image",
        }
    }
    /// Execute-button label.
    pub fn exec_label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::SaveAs => "Save As",
            Self::SaveCopy => "Save Copy",
            Self::Import => "Import",
            Self::Export => "Export",
            Self::Link => "Link",
            Self::Append => "Append",
            Self::SaveImage => "Save As Image",
        }
    }
    pub fn from_slug(s: &str) -> Option<Self> {
        [
            Self::Open,
            Self::SaveAs,
            Self::SaveCopy,
            Self::Import,
            Self::Export,
            Self::Link,
            Self::Append,
            Self::SaveImage,
        ]
        .into_iter()
        .find(|p| p.slug() == s)
    }
}

pub fn purpose(s: &BState) -> BFilePurpose {
    s.get(&FileKey::Purpose.path())
        .and_then(BFilePurpose::from_slug)
        .unwrap_or(BFilePurpose::Open)
}
