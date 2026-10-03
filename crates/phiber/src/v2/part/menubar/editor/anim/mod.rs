//! The animation editors' menus — Blender 5.0's `DOPESHEET_MT_editor_menus`,
//! `GRAPH_MT_editor_menus` (the Graph editor and, without Marker, the
//! Drivers editor) and `NLA_MT_editor_menus`, entry for entry, labels and
//! shortcuts as the operators and the default keymap give them (read with
//! bpy). Structure: which menus; their entries are `set.rs` (the Dope
//! Sheet's and the ones the editors share), `graph.rs` and `nla.rs`.
//!
//! An entry phiber has no operator for yet says so in the status bar when
//! picked (`menubar/op.rs`); the Marker menu, the Dope Sheet's Sidebar and
//! Clear Preview Range act.

pub mod graph;
pub mod nla;
pub mod set;

use super::super::item::{BMenuSpec, set::{m, polled}};
use super::set::TIME_MARKER;

/// `DOPESHEET_MT_editor_menus` in Dope Sheet and Action Editor modes.
pub static ACTION: &[BMenuSpec] = &[
    m("View", set::VIEW),
    m("Select", set::SELECT),
    m("Marker", TIME_MARKER),
    polled("Channel", set::CHANNEL, set::BAnimMenu::channel_menu),
    m("Key", set::KEY),
    polled("Action", set::ACTION, set::BAnimMenu::action_menu),
];
/// `GRAPH_MT_editor_menus` in F-Curves mode.
pub static IPO: &[BMenuSpec] = &[
    m("View", graph::VIEW),
    m("Select", graph::SELECT),
    m("Marker", TIME_MARKER),
    m("Channel", graph::CHANNEL),
    m("Key", graph::KEY),
];
/// `GRAPH_MT_editor_menus` in Drivers mode: no Marker, no playback.
pub static DRIVERS: &[BMenuSpec] = &[
    m("View", graph::DRIVERS_VIEW),
    m("Select", graph::SELECT),
    m("Channel", graph::DRIVERS_CHANNEL),
    m("Key", graph::KEY),
];
/// `NLA_MT_editor_menus`.
pub static NLA: &[BMenuSpec] = &[
    m("View", nla::VIEW),
    m("Select", nla::SELECT),
    m("Marker", TIME_MARKER),
    m("Add", nla::ADD),
    m("Tracks", nla::TRACKS),
    m("Strips", nla::STRIPS),
];
