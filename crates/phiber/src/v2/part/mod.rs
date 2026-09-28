//! Part — the leaf. Every variant lives in its own subfolder with item + list.

pub mod set;
pub mod activity;
pub mod assetshelf;
pub mod caption;
pub mod channels;
pub mod custom;
pub mod action;
pub mod buts;
pub mod clip;
pub mod file;
pub mod image;
pub mod node;
pub mod outliner;
pub mod pref;
pub mod seq;
pub mod spreadsheet;
pub mod text;
pub mod time;
pub mod view3d;
pub mod execute;
pub mod hud;
pub mod menubar;
pub mod modebar;
pub mod navbar;
pub mod props;
pub mod sidebar;
pub mod spacebar;
pub mod statusbar;
pub mod statuslog;
pub mod strip;
pub mod filter;
pub mod framerange;
pub mod playback;
pub mod search;
pub mod switcher;
pub mod toolbar;
pub mod toolprops;
pub mod zonebar;

use crate::v2::host::BHostType;
use crate::v2::spec::BSpec;
use crate::v2::spec::BSpecKind;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use self::activity::BActivity;
use self::assetshelf::BAssetShelf;
use self::caption::BCaption;
use self::caption::BCaptionSide;
use self::channels::BChannels;
use self::custom::BCustom;
use self::execute::BExecute;
use self::hud::BHud;
use self::menubar::BMenubar;
use self::modebar::BModebar;
use self::navbar::BNavBar;
use self::props::BPanel;
use self::sidebar::BSidebar;
use self::spacebar::BSpacebar;
use self::statusbar::BStatusbar;
use self::switcher::BSwitcher;
use self::toolbar::BToolbar;
use self::toolprops::BToolProps;
use self::zonebar::BZonebar;
pub use set::SET;

// `BPartType` — leaf variants.
// 
// `Activity` is Phiber-native (not in Blender): a full-height leftmost bar
// with global navigation (home / chat / spaces / stream / settings / profile).
// Everything else mirrors a Blender part.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPartType {
    Menubar,
    Modebar,
    Switcher,
    Search,
    Filter,
    Spacebar,
    Toolbar,
    Sidebar,
    Panel,
    Channels,
    Statusbar,
    /// The action log at the foot of the window.
    StatusLog,
    Zonebar,
    NavBar,
    HUD,
    Execute,
    /// The file browser's folder moves — previous, next, parent — the start
    /// of its path strip's left column (`FILEBROWSER_PT_directory_path`).
    FileNav,
    /// The refresh at that column's end, over the source list's edge.
    FileRefresh,
    /// The file browser's directory strip — Blender's `FILEBROWSER_PT_directory_path`
    /// in its `ui` region: the directory and the search, starting where the
    /// file list does.
    PathBar,
    /// The file browser's type filter — the hidden-files eye, the filter
    /// toggle and its settings popover — at the right end of its path strip.
    TypeFilter,
    /// The file browser's source list — Blender's `tools` region: volumes,
    /// system folders, bookmarks, recent (the asset browser's libraries and
    /// catalogs).
    SourceList,
    AssetShelf,
    AssetShelfHeader,
    ToolProps,
    AdjustLastOp,
    /// The Timeline header's transport group (`playback_controls`).
    Playback,
    /// The Timeline header's current frame and Start / End fields.
    FrameRange,
    /// The Image editor header's image data-block (`template_ID`).
    ImageBlock,
    /// The Action Editor header's action and slot (`template_action`).
    ActionTemplate,
    /// The UV editor header's selection settings, left of the menus.
    UvSelect,
    /// The UV editor header's pivot, snapping and proportional editing.
    UvTransform,
    /// The Image editor header's right end: gizmo / overlay toggles, render
    /// slot, view layer, pass and display channels.
    ImageDisplay,
    Scrubbing,
    Activity,
    CaptionLead,
    CaptionTrail,
    Custom,
}

impl BPartType {
    pub fn label(self) -> &'static str {
        match self {
            BPartType::Menubar => "Menu Bar",
            BPartType::Modebar => "Mode Bar",
            BPartType::Switcher => "Switcher",
            BPartType::Search => "Search",
            BPartType::Filter => "Filter",
            BPartType::Spacebar => "Space Bar",
            BPartType::Toolbar => "Toolbar",
            BPartType::Sidebar => "Sidebar",
            BPartType::Panel => "Panel",
            BPartType::Channels => "Channels",
            BPartType::Statusbar => "Status Bar",
            BPartType::Zonebar => "Zone Visibility",
            BPartType::NavBar => "Nav Bar",
            BPartType::HUD => "HUD",
            BPartType::Execute => "Execute",
            BPartType::FileNav => "Folder Navigation",
            BPartType::FileRefresh => "Refresh",
            BPartType::PathBar => "File Path",
            BPartType::SourceList => "Source List",
            BPartType::TypeFilter => "Type Filter",
            BPartType::AssetShelf => "Asset Shelf",
            BPartType::AssetShelfHeader => "Asset Shelf Header",
            BPartType::ToolProps => "Tool Properties",
            BPartType::AdjustLastOp => "Adjust Last Op",
            BPartType::Scrubbing => "Scrubbing",
            BPartType::Playback => "Playback",
            BPartType::FrameRange => "Frame Range",
            BPartType::ImageBlock => "Image",
            BPartType::ActionTemplate => "Action",
            BPartType::UvSelect => "UV Selection",
            BPartType::UvTransform => "UV Transform",
            BPartType::ImageDisplay => "Image Display",
            BPartType::Activity => "Activity",
            BPartType::CaptionLead | BPartType::CaptionTrail => "Window Controls",
            BPartType::StatusLog => "Action Log",
            BPartType::Custom => "Custom",
        }
    }
}

// `BPartSpec` — one part descriptor (identity + label).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPartSpec {
    pub kind: BPartType,
    pub label: &'static str,
}

impl BSpec for BPartSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(self.kind)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

// `BPartItem` — one part; dispatcher by `BPartType`.
// Every per-part composer implements `BViewItem`.

#[derive(Clone, Copy, Debug)]
pub struct BPartItem {
    pub host: BHostType,
    pub part: BPartType,
}

impl BPartItem {
    pub fn new(host: BHostType, part: BPartType) -> Self {
        Self { host, part }
    }

    fn body(&self, s: &BState) -> BView {
        let h = self.host;
        match self.part {
            BPartType::Menubar => BMenubar::new(h).build(s),
            BPartType::Modebar => BModebar::new(h).build(s),
            BPartType::Switcher => BSwitcher::new(h).build(s),
            BPartType::Search => crate::v2::part::search::BSearch::new(h).build(s),
            BPartType::Filter => crate::v2::part::filter::BFilter::new(h).build(s),
            BPartType::Spacebar => BSpacebar::new(h).build(s),
            BPartType::Toolbar => BToolbar::new(h).build(s),
            BPartType::Sidebar => BSidebar::new(h).build(s),
            BPartType::Panel => BPanel::new(h).build(s),
            BPartType::Channels => BChannels::new(h).build(s),
            BPartType::Statusbar => BStatusbar::new(h).build(s),
            BPartType::Zonebar => BZonebar::new(h).build(s),
            BPartType::NavBar => BNavBar::new(h).build(s),
            BPartType::HUD => BHud::new(h).build(s),
            BPartType::Execute => BExecute::new(h).build(s),
            BPartType::FileNav => crate::v2::part::file::nav::view(s),
            BPartType::FileRefresh => crate::v2::part::file::nav::refresh(s),
            BPartType::PathBar => crate::v2::part::file::path_bar::view(h, s),
            BPartType::SourceList => crate::v2::part::file::sidebar::source_list(h, s),
            BPartType::TypeFilter => crate::v2::part::file::path_bar::type_filter(s),
            BPartType::AssetShelf => BAssetShelf::new(h).build(s),
            BPartType::AssetShelfHeader => {
                crate::v2::part::assetshelf::BAssetShelfHeader::new(h).build(s)
            }
            BPartType::ToolProps | BPartType::AdjustLastOp => BToolProps::new(h).build(s),
            BPartType::Activity => BActivity::new(h).build(s),
            BPartType::CaptionLead => BCaption { host: h, side: BCaptionSide::Lead }.build(s),
            BPartType::CaptionTrail => BCaption { host: h, side: BCaptionSide::Trail }.build(s),
            BPartType::StatusLog => crate::v2::part::statuslog::BStatusLog::new(h).build(s),
            BPartType::Custom => BCustom::new(h).build(s),
            BPartType::Playback => crate::v2::part::playback::BPlayback::new(h).build(s),
            BPartType::FrameRange => crate::v2::part::framerange::BFrameRange::new(h).build(s),
            BPartType::ImageBlock => crate::v2::part::image::header::BImageBlock::new(h).build(s),
            BPartType::ActionTemplate => crate::v2::part::action::template::BActionTemplate::new(h).build(s),
            BPartType::UvSelect => crate::v2::part::image::uv::BUvSelect(h).build(s),
            BPartType::UvTransform => crate::v2::part::image::uv::BUvTransform(h).build(s),
            BPartType::ImageDisplay => crate::v2::part::image::header::BImageDisplay::new(h).build(s),
            BPartType::Scrubbing => BView::empty(),
        }
    }
}

impl BViewItem for BPartItem {
    fn build(&self, state: &BState) -> BView {
        BView::Part {
            kind: self.part,
            body: Box::new(self.body(state)),
        }
    }
}

// `BPartList` — iterate the parts of a pane.

#[derive(Clone, Copy, Debug)]
pub struct BPartList<'a> {
    pub host: BHostType,
    pub parts: &'a [BPartType],
}

impl<'a> BPartList<'a> {
    pub fn new(host: BHostType, parts: &'a [BPartType]) -> Self {
        Self { host, parts }
    }
}

impl<'a> BViewList for BPartList<'a> {
    fn items(&self, state: &BState) -> Vec<BView> {
        self.parts
            .iter()
            .map(|&p| BPartItem::new(self.host, p).build(state))
            .collect()
    }
}

// One subfolder per BPartType variant.
