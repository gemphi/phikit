//! Part set — one `BPartSpec` per `BPartType`, in the enum's own order.
//!
//! It used to carry a label per entry and held 28 of the 37 parts. Two things
//! made that possible: the label was a second copy of `BPartType::label`, with
//! nothing comparing them, and nothing required the table to be complete —
//! unlike `host/set.rs`, whose exhaustive match stops the build when a host is
//! missing. Nine parts were simply absent, and since nothing read this table
//! they stayed absent.
//!
//! Now an entry is its kind and nothing else (`BPartSpec::label` asks the enum),
//! and `every_part_is_a_part` checks this list against `BPartType::ALL`, so a
//! part added without a line here fails the build's tests rather than going
//! quietly missing.

use super::BPartSpec;
use super::BPartType;

pub static SET: &[BPartSpec] = &[
    BPartSpec { kind: BPartType::Menubar },
    BPartSpec { kind: BPartType::Modebar },
    BPartSpec { kind: BPartType::Switcher },
    BPartSpec { kind: BPartType::Search },
    BPartSpec { kind: BPartType::Filter },
    BPartSpec { kind: BPartType::Spacebar },
    BPartSpec { kind: BPartType::Toolbar },
    BPartSpec { kind: BPartType::Sidebar },
    BPartSpec { kind: BPartType::Panel },
    BPartSpec { kind: BPartType::Channels },
    BPartSpec { kind: BPartType::Statusbar },
    BPartSpec { kind: BPartType::StatusLog },
    BPartSpec { kind: BPartType::Zonebar },
    BPartSpec { kind: BPartType::NavBar },
    BPartSpec { kind: BPartType::HUD },
    BPartSpec { kind: BPartType::Execute },
    BPartSpec { kind: BPartType::FileNav },
    BPartSpec { kind: BPartType::FileRefresh },
    BPartSpec { kind: BPartType::PathBar },
    BPartSpec { kind: BPartType::TypeFilter },
    BPartSpec { kind: BPartType::SourceList },
    BPartSpec { kind: BPartType::Dataset },
    BPartSpec { kind: BPartType::AssetShelf },
    BPartSpec { kind: BPartType::AssetShelfHeader },
    BPartSpec { kind: BPartType::ToolProps },
    BPartSpec { kind: BPartType::AdjustLastOp },
    BPartSpec { kind: BPartType::Playback },
    BPartSpec { kind: BPartType::FrameRange },
    BPartSpec { kind: BPartType::ImageBlock },
    BPartSpec { kind: BPartType::ActionTemplate },
    BPartSpec { kind: BPartType::UvSelect },
    BPartSpec { kind: BPartType::UvTransform },
    BPartSpec { kind: BPartType::ImageDisplay },
    BPartSpec { kind: BPartType::Scrubbing },
    BPartSpec { kind: BPartType::Activity },
    BPartSpec { kind: BPartType::CaptionLead },
    BPartSpec { kind: BPartType::CaptionTrail },
    BPartSpec { kind: BPartType::Custom },
];
