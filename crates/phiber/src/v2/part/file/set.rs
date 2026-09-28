//! File Browser set — sidebar categories, columns, display modes, filters,
//! nav operators. Static data only; anything runtime-resolved (volumes,
//! system folders, the listing itself) lives in the sibling modules.
//!
//! Blender ref: `bl_ui/space_filebrowser.py`, `ED_fileselect.hh`,
//! `DNA_space_enums.h`.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

// ── Sidebar category (FSMenuCategory) ──────────────────────────────────────
/// Blender's `FSMenuCategory` (`ED_fileselect.hh:244`), in declaration order.
/// `System` is the volume list; `SystemBookmarks` is the user's shell folders
/// — the two used to be conflated into one "System" section.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BFileCategory {
    System,
    SystemBookmarks,
    Bookmarks,
    Recent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BFileCategorySpec {
    pub kind: BFileCategory,
    pub label: &'static str,
}

impl BSpec for BFileCategorySpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

pub static CATEGORIES: &[BFileCategorySpec] = &[
    BFileCategorySpec {
        kind: BFileCategory::System,
        label: "Volumes",
    },
    BFileCategorySpec {
        kind: BFileCategory::SystemBookmarks,
        label: "System",
    },
    BFileCategorySpec {
        kind: BFileCategory::Bookmarks,
        label: "Bookmarks",
    },
    BFileCategorySpec {
        kind: BFileCategory::Recent,
        label: "Recent",
    },
];

// ── One static entry inside a sidebar category ─────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BFileEntry {
    pub icon: &'static str,
    pub label: &'static str,
    /// Path token — expanded at runtime by `fsmenu`.
    pub path: &'static str,
    /// `FSMenuEntry::save` — whether the entry persists to the bookmarks file.
    pub save: bool,
}

impl BSpec for BFileEntry {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

const fn e(icon: &'static str, label: &'static str, path: &'static str) -> BFileEntry {
    BFileEntry {
        icon,
        label,
        path,
        save: false,
    }
}

/// The shape of the system-bookmark list. `fsmenu::system_bookmarks()`
/// resolves these against the real environment and drops what does not exist.
pub static SYSTEM: &[BFileEntry] = &[
    e("home",     "Home",       "$HOME"),
    e("monitor",  "Desktop",    "$HOME/Desktop"),
    e("doc",      "Documents",  "$HOME/Documents"),
    e("download", "Downloads",  "$HOME/Downloads"),
    e("speaker",  "Music",      "$HOME/Music"),
    e("image",    "Pictures",   "$HOME/Pictures"),
    e("film",     "Videos",     "$HOME/Videos"),
    e("cloud",    "OneDrive",   "$ONEDRIVE"),
];

// ── Columns (FileAttributeColumnType) ──────────────────────────────────────
/// Blender has exactly three attribute columns (`ED_fileselect.hh:45`);
/// there is no "Type" column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BFileColumn {
    Name,
    DateModified,
    Size,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BFileColumnSpec {
    pub kind: BFileColumn,
    pub label: &'static str,
    /// `FileAttributeColumn::text_align` — Size is right-aligned.
    pub right_align: bool,
}

impl BSpec for BFileColumnSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

pub static COLUMNS: &[BFileColumnSpec] = &[
    BFileColumnSpec {
        kind: BFileColumn::Name,
        label: "Name",
        right_align: false,
    },
    BFileColumnSpec {
        kind: BFileColumn::DateModified,
        label: "Modified Date",
        right_align: false,
    },
    BFileColumnSpec {
        kind: BFileColumn::Size,
        label: "Size",
        right_align: true,
    },
];

// ── Display type (eFileDisplayType) ────────────────────────────────────────
/// `DNA_space_enums.h:535`. The previous table mislabelled these: `List` was
/// captioned "Vertical List" but `Details` was captioned "Horizontal List",
/// which are two different Blender modes, not a list/detail distinction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BFileDisplayType {
    Vertical,
    Horizontal,
    Thumbnail,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BFileDisplaySpec {
    pub kind: BFileDisplayType,
    pub icon: &'static str,
    pub label: &'static str,
}

impl BSpec for BFileDisplaySpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

pub static DISPLAY_MODES: &[BFileDisplaySpec] = &[
    BFileDisplaySpec {
        kind: BFileDisplayType::Vertical,
        icon: "list",
        label: "Vertical List",
    },
    BFileDisplaySpec {
        kind: BFileDisplayType::Horizontal,
        icon: "columns",
        label: "Horizontal List",
    },
    BFileDisplaySpec {
        kind: BFileDisplayType::Thumbnail,
        icon: "grid_2x2",
        label: "Thumbnails",
    },
];

// ── Type filters (FILEBROWSER_PT_filter) ───────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BFileFilterSpec {
    pub icon: &'static str,
    pub label: &'static str,
    /// The `ctx.file.<key>` flag, matching `BFileKind::filter_flag`.
    pub key: &'static str,
}

impl BSpec for BFileFilterSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

/// The ten rows Blender's filter popover draws, in its order.
pub static FILTERS: &[BFileFilterSpec] = &[
    BFileFilterSpec {
        icon: "folder",
        label: "Folders",
        key: "filter_folder",
    },
    BFileFilterSpec {
        icon: "blend",
        label: ".blend Files",
        key: "filter_blender",
    },
    BFileFilterSpec {
        icon: "file_blend",
        label: "Backup .blend Files",
        key: "filter_backup",
    },
    BFileFilterSpec {
        icon: "image",
        label: "Image Files",
        key: "filter_image",
    },
    BFileFilterSpec {
        icon: "film",
        label: "Movie Files",
        key: "filter_movie",
    },
    BFileFilterSpec {
        icon: "file_code",
        label: "Script Files",
        key: "filter_script",
    },
    BFileFilterSpec {
        icon: "type",
        label: "Font Files",
        key: "filter_font",
    },
    BFileFilterSpec {
        icon: "audio",
        label: "Sound Files",
        key: "filter_sound",
    },
    BFileFilterSpec {
        icon: "file_text",
        label: "Text Files",
        key: "filter_text",
    },
    BFileFilterSpec {
        icon: "cloud",
        label: "Volume Files",
        key: "filter_volume",
    },
];

// ── Navigation operators ───────────────────────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BFileNavSpec {
    pub icon: &'static str,
    pub tooltip: &'static str,
    /// Written to `ctx.file.nav`; reduced by `state::file_nav`.
    pub action: &'static str,
}

impl BSpec for BFileNavSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.tooltip
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

pub static NAV: &[BFileNavSpec] = &[
    BFileNavSpec {
        icon: "arrow_left",
        tooltip: "Previous Folder",
        action: "file.previous",
    },
    BFileNavSpec {
        icon: "arrow_right",
        tooltip: "Next Folder",
        action: "file.next",
    },
    BFileNavSpec {
        icon: "arrow_up",
        tooltip: "Parent Directory",
        action: "file.parent",
    },
    BFileNavSpec {
        icon: "refresh",
        tooltip: "Refresh File List",
        action: "file.refresh",
    },
];
