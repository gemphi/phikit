//! Part — the leaf. Every variant lives in its own subfolder with item + list.

mod kind;
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
use self::custom::BCustomItem;
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
    /// The Spreadsheet's dataset region: the object's geometry as a tree,
    /// one row per domain, the chosen one active.
    Dataset,
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
    /// Every part, in the order the enum declares them.
    ///
    /// Hand-kept beside the enum, like `BHostType::ALL` — Rust cannot walk a
    /// plain enum's variants. `every_part_is_a_part` checks the two agree, so
    /// a variant added without a line here fails rather than going missing
    /// from whatever walks parts.
    pub const ALL: &'static [Self] = &[
        Self::Menubar,
        Self::Modebar,
        Self::Switcher,
        Self::Search,
        Self::Filter,
        Self::Spacebar,
        Self::Toolbar,
        Self::Sidebar,
        Self::Panel,
        Self::Channels,
        Self::Statusbar,
        Self::StatusLog,
        Self::Zonebar,
        Self::NavBar,
        Self::HUD,
        Self::Execute,
        Self::FileNav,
        Self::FileRefresh,
        Self::PathBar,
        Self::TypeFilter,
        Self::SourceList,
        Self::Dataset,
        Self::AssetShelf,
        Self::AssetShelfHeader,
        Self::ToolProps,
        Self::AdjustLastOp,
        Self::Playback,
        Self::FrameRange,
        Self::ImageBlock,
        Self::ActionTemplate,
        Self::UvSelect,
        Self::UvTransform,
        Self::ImageDisplay,
        Self::Scrubbing,
        Self::Activity,
        Self::CaptionLead,
        Self::CaptionTrail,
        Self::Custom,
    ];
}

impl BPartType {
    pub const fn label(self) -> &'static str {
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
            BPartType::Dataset => "Dataset",
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

/// What every part is: a kind, a label, a way to make one for a host, and a
/// view it builds.
///
/// `structure.md` has declared a `BPart` trait since the tree was laid out and
/// `part/part.rs` was never written, so the shape lived as a convention — every
/// part folder defines a `B<Name>` with `::new(host)` and `impl BViewItem`. A
/// convention with nothing checking it holds until someone is in a hurry: the
/// file browser's five parts reached free functions for months and nothing
/// said so.
///
/// This is that convention with a name. `BViewItem` is the supertrait because
/// building a view is what a part is FOR; the two consts are what the
/// registration lists need, kept beside the builder rather than in three
/// separate tables that drift.
///
/// It does not remove the dispatcher's match — Rust cannot turn a runtime
/// `BPartType` into a type's associated consts, so something has to map one to
/// the other. What it removes is the chance of the map disagreeing with the
/// part: `every_part_is_a_part` checks each implementor's `KIND` and `LABEL`
/// against the enum's own answers.
pub trait BPart: BViewItem {
    /// The variant this part answers to.
    const KIND: BPartType;
    /// What it is called. `BPartType::label` must agree.
    const LABEL: &'static str;
    /// One of these, for a host.
    fn make(host: BHostType) -> Self;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPartSpec {
    pub kind: BPartType,
}

impl BPartSpec {
    /// The part's label — `BPartType`'s own, not a second copy.
    ///
    /// This field used to hold its own string, so the label existed twice: once
    /// here in a table of 28 entries and once in `BPartType::label`'s match of
    /// 37. Nothing forced the table to be complete and nothing compared the two.
    pub const fn label(&self) -> &'static str {
        self.kind.label()
    }
}

impl BSpec for BPartSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(self.kind)
    }
    fn label(&self) -> &'static str {
        self.label()
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
            // The file browser's five used to reach free functions here while
            // every other part went through `BViewItem`. Five exceptions in one
            // corner is how an interface stops being one: nothing enforced the
            // shape, so nothing told anyone it had been dropped.
            BPartType::FileNav => crate::v2::part::file::nav::BFileNav::new(h).build(s),
            BPartType::FileRefresh => crate::v2::part::file::nav::BFileRefresh::new(h).build(s),
            BPartType::PathBar => crate::v2::part::file::path_bar::BPathBar::new(h).build(s),
            BPartType::SourceList => crate::v2::part::file::sidebar::BSourceList::new(h).build(s),
            BPartType::Dataset => crate::v2::part::spreadsheet::dataset::BDataset::new().build(s),
            BPartType::TypeFilter => crate::v2::part::file::path_bar::BTypeFilter::new(h).build(s),
            BPartType::AssetShelf => BAssetShelf::new(h).build(s),
            BPartType::AssetShelfHeader => {
                crate::v2::part::assetshelf::BAssetShelfHeader::new(h).build(s)
            }
            BPartType::ToolProps | BPartType::AdjustLastOp => BToolProps::new(h).build(s),
            BPartType::Activity => BActivity::new(h).build(s),
            BPartType::CaptionLead => BCaption { host: h, side: BCaptionSide::Lead }.build(s),
            BPartType::CaptionTrail => BCaption { host: h, side: BCaptionSide::Trail }.build(s),
            BPartType::StatusLog => crate::v2::part::statuslog::BStatusLog::new(h).build(s),
            BPartType::Custom => BCustomItem::new(h).build(s),
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
