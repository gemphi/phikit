//! Modal — a top-level overlay peer of Space. Opens on top of the active
//! space when `ctx.modal` is set. Each modal is a self-contained mini-app.

pub mod set;
pub mod list;
pub mod about;
pub mod collab;
pub mod command;
pub mod file_browser;
pub mod file_close;
pub mod revert;
pub mod help;
pub mod profile;
pub mod reader;
pub mod new_image;
pub mod rename;
pub mod render;
pub mod setting;
pub mod workspace;
pub mod splash;

use crate::v2::spec::BSpec;
use crate::v2::spec::BSpecKind;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList as _;
pub use list::BModalList;
pub use set::SET;

// `BModalType` — every overlay dialog that can be opened.
// Opens by writing its slug to `ctx.modal`; closes by clearing the key.
// 
// `FileBrowser` is the same underlying widget as the `File` host — Blender
// reuses `space_file` for both the workspace editor and the modal dialog
// (Open / Save As / Import / Export). The overlay flavour reads
// `ctx.file.purpose` for its title (Open Blender File / Save As / …).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BModalType {
    Settings,
    Help,
    Profile,
    Command,
    Splash,
    About,
    FileBrowser,
    /// Blender's render window — an Image Editor in its own window
    /// (`WM_OT_render` with `render_display_type == 'WINDOW'`).
    Render,
    /// Workspace picker (Activity ▸ Spaces).
    Spaces,
    /// Collaboration: chat / stream sessions (Activity ▸ Chat, Stream).
    Collab,
    /// Reading a document from the store: the manual, the release notes.
    Reader,
    /// F2: the active object's name (`TOPBAR_PT_name`).
    Rename,
    /// "Save changes before closing?" (`wm_close_file_dialog`).
    FileClose,
    /// "Revert to the Saved File" (`wm.revert_mainfile`'s confirm).
    Revert,
    /// `image.new`'s dialog (`image_new_draw`).
    NewImage,
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
        Self::Render,
        Self::Spaces,
        Self::Collab,
        Self::Reader,
        Self::Rename,
        Self::FileClose,
        Self::Revert,
        Self::NewImage,
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
            Self::Render => "render",
            Self::Spaces => "spaces",
            Self::Collab => "collab",
            Self::Reader => "reader",
            Self::Rename => "rename",
            Self::FileClose => "file_close",
            Self::Revert => "revert",
            Self::NewImage => "new_image",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|m| m.slug() == s)
    }

    pub fn spec(self) -> &'static self::BModalSpec {
        &self::SET[self as usize]
    }
    pub fn label(self) -> &'static str {
        self.spec().label
    }
    pub fn icon(self) -> &'static str {
        self.spec().icon
    }

    /// What the title bar shows: the spec's icon and label, except for the
    /// reader, whose title is the document's.
    pub fn heading(self, s: &crate::v2::state::BState) -> (&'static str, &'static str) {
        match self {
            Self::Reader => {
                let doc = self::reader::current(s);
                (doc.icon, doc.title)
            }
            Self::Rename if s.get(crate::v2::state::AppKey::RenameMarker).is_some() => ("marker", self::rename::MARKER_TITLE),
            _ => (self.icon(), self.label()),
        }
    }
}

// `BModalSpec` — static descriptor for one modal.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BModalSpec {
    pub kind: BModalType,
    pub label: &'static str,
    pub icon: &'static str,
    pub width: f32,
    pub height: f32,
    pub shape: BModalShape,
    /// The editors the dialog raises, declared exactly as a space declares
    /// its areas and built by the same zone builder: the file dialog is a
    /// Center area whose default host is File; Preferences is the Prefs
    /// editor; the render window is an Image editor. Empty for the dialogs
    /// that are Blender menus rather than editors (Splash, About, Help…),
    /// whose body is their own `BViewItem`.
    pub areas: &'static [crate::v2::area::BAreaSpec],
}

/// How a modal takes the window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BModalShape {
    /// A card of the declared size, centred.
    Card,
    /// The whole window (Welcome, About).
    Fullscreen,
    /// A reading sheet: `share` of the window's width, its full height,
    /// centred — a page over the workspace.
    Sheet { share: f32 },
    /// A search palette: half the window wide, hung under the top bar, on a
    /// see-through ground with a border — the field is its heading, so it
    /// has no title bar.
    Palette,
}

impl BSpec for BModalSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(crate::v2::part::BPartType::Custom) // modals rendered as overlay parts
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

// `BModalItem` — one modal. Emits `BView::Modal` populated by the
// per-modal content module.

#[derive(Clone, Copy, Debug)]
pub struct BModalItem {
    pub modal: BModalType,
}

impl BModalItem {
    pub fn new(modal: BModalType) -> Self {
        Self { modal }
    }
}

impl BViewItem for BModalItem {
    /// A dialog that raises editors (`spec.areas`) is built exactly as a
    /// space is: its areas go through the zone builder, scoped to the modal
    /// so its slots stay its own. A dialog that is a menu (Splash, About…)
    /// builds its own body.
    fn build(&self, s: &BState) -> BView {
        let spec = self.modal.spec();
        if !spec.areas.is_empty() {
            let zones = crate::v2::area::with_scope(self.modal, || {
                crate::v2::zone::BZoneList::of(spec.areas).items(s)
            });
            let body = match zones.len() {
                1 => zones.into_iter().next().unwrap_or_else(BView::empty),
                _ => BView::column(zones),
            };
            return BView::Modal { kind: self.modal, body: Box::new(body) };
        }
        let body = match self.modal {
            BModalType::Settings => setting::BSettingsModal.build(s),
            BModalType::Help => help::BHelpModal.build(s),
            BModalType::Profile => profile::BProfileModal.build(s),
            BModalType::Command => command::BCommandModal.build(s),
            BModalType::FileBrowser => file_browser::BFileBrowserModal.build(s),
            BModalType::Splash => splash::BSplashModal.build(s),
            BModalType::About => about::BAboutModal.build(s),
            BModalType::Render => render::BRenderModal.build(s),
            BModalType::Spaces => workspace::BSpacesModal.build(s),
            BModalType::Reader => reader::BReaderModal.build(s),
            BModalType::Rename => rename::BRenameModal.build(s),
            BModalType::FileClose => file_close::BFileCloseModal.build(s),
            BModalType::Revert => revert::BRevertModal.build(s),
            BModalType::Collab => collab::BCollabModal.build(s),
            BModalType::NewImage => new_image::BNewImageModal.build(s),
        };
        BView::Modal {
            kind: self.modal,
            body: Box::new(body),
        }
    }
}

// Per-modal content — one folder per BModalType variant.
