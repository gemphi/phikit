//! Which part is which: every `BPart` implementation, in one place.
//!
//! The registration half of a part — the variant it answers to and what it is
//! called — belongs beside the enum it registers against, not scattered across
//! thirty folders; the BEHAVIOUR half (`BViewItem::build`) stays on the type in
//! its own file, where it was.
//!
//! `LABEL` reads `BPartType::label`, which is a `const fn`, so a part's name
//! exists exactly once in the crate. It used to exist three times — the enum's
//! match, a string in `part/set.rs`, and nothing comparing them.
//!
//! The macro is the point: adding a part is one line here, and the compiler
//! then requires the type to exist, to have `::new(host)`, and to build a view.

use super::{BPart, BPartType};
use crate::v2::host::BHostType;

macro_rules! parts {
    ($($variant:ident => $ty:path,)*) => {
        $(
            impl BPart for $ty {
                const KIND: BPartType = BPartType::$variant;
                const LABEL: &'static str = BPartType::$variant.label();
                fn make(host: BHostType) -> Self {
                    Self::new(host)
                }
            }
        )*
    };
}

parts! {
    Menubar => super::BMenubar,
    Modebar => super::BModebar,
    Switcher => super::BSwitcher,
    Search => crate::v2::part::search::BSearch,
    Filter => crate::v2::part::filter::BFilter,
    Spacebar => super::BSpacebar,
    Toolbar => super::BToolbar,
    Sidebar => super::BSidebar,
    Panel => super::BPanel,
    Channels => super::BChannels,
    Statusbar => super::BStatusbar,
    Zonebar => super::BZonebar,
    NavBar => super::BNavBar,
    HUD => super::BHud,
    Execute => super::BExecute,
    FileNav => crate::v2::part::file::nav::BFileNav,
    FileRefresh => crate::v2::part::file::nav::BFileRefresh,
    PathBar => crate::v2::part::file::path_bar::BPathBar,
    SourceList => crate::v2::part::file::sidebar::BSourceList,
    TypeFilter => crate::v2::part::file::path_bar::BTypeFilter,
    AssetShelf => super::BAssetShelf,
    AdjustLastOp => super::BToolProps,
    Activity => super::BActivity,
    StatusLog => crate::v2::part::statuslog::BStatusLog,
    Custom => super::BCustomItem,
    Playback => crate::v2::part::playback::BPlayback,
    FrameRange => crate::v2::part::framerange::BFrameRange,
    ImageBlock => crate::v2::part::image::header::BImageBlock,
    ActionTemplate => crate::v2::part::action::template::BActionTemplate,
    ImageDisplay => crate::v2::part::image::header::BImageDisplay,
}
