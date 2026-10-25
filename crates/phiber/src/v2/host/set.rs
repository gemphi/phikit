//! How a `BHostType` finds its own spec and key.
//!
//! This file is the *index*, not the content. Each host owns a folder:
//! `host/<name>/mod.rs` says what the host is (kind, label, icon, slug) and
//! `host/<name>/set.rs` holds its regions, panes and parts.
//!
//! The lookup is an exhaustive `match`, not a table indexed by the enum's
//! discriminant. That is the whole point: **adding a `BHostType` variant
//! stops the build until its host exists**. A positional table would have
//! compiled happily and silently handed the new host its neighbour's
//! regions.

use super::{BHostSpec, BHostType};
use super::{
    action, activity, asset, buts, clip, compositor, console, drivers, file, geonode, image,
    info, ipo, logic, nla, node, outliner, play, pref, project, seq, shader, spreadsheet, statusbar, text,
    time, topbar, uv, view3d,
};
use crate::v2::region::BRegionSpec;

static EMPTY_REGIONS: &[BRegionSpec] = &[];

/// The placeholder host — an area with nothing assigned to it yet. No
/// regions, so it draws as an empty frame rather than a broken editor.
static EMPTY: BHostSpec = BHostSpec {
    kind: BHostType::Empty,
    label: "",
    icon: "",
    regions: EMPTY_REGIONS,
};

impl BHostType {
    /// This host's spec — its label, icon and regions.
    pub fn spec(self) -> &'static BHostSpec {
        match self {
            BHostType::Topbar => &topbar::SPEC,
            BHostType::View3D => &view3d::SPEC,
            BHostType::Outliner => &outliner::SPEC,
            BHostType::Buts => &buts::SPEC,
            BHostType::Time => &time::SPEC,
            BHostType::Statusbar => &statusbar::SPEC,
            BHostType::File => &file::SPEC,
            BHostType::Image => &image::SPEC,
            BHostType::Info => &info::SPEC,
            BHostType::Seq => &seq::SPEC,
            BHostType::Text => &text::SPEC,
            BHostType::Action => &action::SPEC,
            BHostType::Nla => &nla::SPEC,
            BHostType::Node => &node::SPEC,
            BHostType::Logic => &logic::SPEC,
            BHostType::Console => &console::SPEC,
            BHostType::Uv => &uv::SPEC,
            BHostType::Compositor => &compositor::SPEC,
            BHostType::Shader => &shader::SPEC,
            BHostType::GeoNode => &geonode::SPEC,
            BHostType::Clip => &clip::SPEC,
            BHostType::Drivers => &drivers::SPEC,
            BHostType::Asset => &asset::SPEC,
            BHostType::Spreadsheet => &spreadsheet::SPEC,
            BHostType::Ipo => &ipo::SPEC,
            BHostType::Prefs => &pref::SPEC,
            BHostType::Play => &play::SPEC,
            BHostType::Project => &project::SPEC,
            BHostType::Activity => &activity::SPEC,
            BHostType::Empty => &EMPTY,
        }
    }

    /// Key used for this host inside a `BPath` (state key). Each host names
    /// its own in `host/<name>/mod.rs`, so the key lives beside the host
    /// rather than in a table that has to be kept in step with one.
    pub fn slug(self) -> &'static str {
        match self {
            BHostType::Topbar => topbar::SLUG,
            BHostType::View3D => view3d::SLUG,
            BHostType::Outliner => outliner::SLUG,
            BHostType::Buts => buts::SLUG,
            BHostType::Time => time::SLUG,
            BHostType::Statusbar => statusbar::SLUG,
            BHostType::File => file::SLUG,
            BHostType::Image => image::SLUG,
            BHostType::Info => info::SLUG,
            BHostType::Seq => seq::SLUG,
            BHostType::Text => text::SLUG,
            BHostType::Action => action::SLUG,
            BHostType::Nla => nla::SLUG,
            BHostType::Node => node::SLUG,
            BHostType::Logic => logic::SLUG,
            BHostType::Console => console::SLUG,
            BHostType::Uv => uv::SLUG,
            BHostType::Compositor => compositor::SLUG,
            BHostType::Shader => shader::SLUG,
            BHostType::GeoNode => geonode::SLUG,
            BHostType::Clip => clip::SLUG,
            BHostType::Drivers => drivers::SLUG,
            BHostType::Asset => asset::SLUG,
            BHostType::Spreadsheet => spreadsheet::SLUG,
            BHostType::Ipo => ipo::SLUG,
            BHostType::Prefs => pref::SLUG,
            BHostType::Play => play::SLUG,
            BHostType::Project => project::SLUG,
            BHostType::Activity => activity::SLUG,
            BHostType::Empty => "empty",
        }
    }
}
