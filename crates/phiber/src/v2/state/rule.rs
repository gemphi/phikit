//! Rules: what else changes when a key is set.
//!
//! A `Set` writes one key. Some keys mean more than their value — a named
//! viewpoint is two angles, a frame step is a new frame, a path typed into
//! the file browser is a trip through its history. Each of those meanings
//! is a [`BRule`] that lives with the level it belongs to (the viewpoint
//! with the scene, the step with the timeline, the history with the file
//! browser), and this file is only the table that runs them.
//!
//! Rules run once per `Set`, in table order, after the value is stored.
//! What a rule writes does not trigger other rules: a rule that needs a
//! follow-on writes it itself, so the chain is readable in one place.

use super::{BPath, BState};

/// One follow-up of a `Set`.
pub trait BRule: Sync {
    /// Whether this rule runs when `path` is set.
    fn wants(&self, path: &BPath) -> bool;
    /// Runs after `path` was set to `value`.
    fn after(&self, path: &BPath, value: &str, s: &mut BState);
}

/// Every rule in the app, in the order they run.
fn table() -> &'static [&'static dyn BRule] {
    use crate::v2::modal::{reader::back, rename, workspace::pick};
    use crate::v2::part::activity::choose;
    use crate::v2::part::menubar::op as menu;
    use crate::v2::part::outliner::op as outliner;
    use crate::v2::part::{file::op, pref::rule as pref, props::active, props::fold, time::range, time::step};
    use crate::v2::scene::{edit, marker, select, view_op, viewpoint};
    use crate::v2::space::step as space;
    &[
        &viewpoint::BViewpointRule,
        &pref::BThemePickRule,
        &pref::BFactoryRule,
        &choose::BActivityTabRule,
        &choose::BActivityPickRule,
        &step::BFrameStepRule,
        &step::BFrameLockRule,
        &step::BPlayFromRule,
        &range::BFrameRangeRule,
        &active::BActiveObjectRule,
        &select::BPickSelectsRule,
        &select::BSelectOpRule,
        &edit::BSceneEditRule,
        &view_op::BView3dOpRule,
        &marker::BMarkerOpRule,
        &rename::BRenameOpenRule,
        &rename::BRenameRule,
        &outliner::BOutlinerOpRule,
        &outliner::BHideDeselectsRule,
        &crate::v2::part::outliner::drop::BOutlinerDropRule,
        &crate::v2::scene::action::rule::BActionOpRule,
        &crate::v2::part::modebar::workspace::BWorkspaceModeRule,
        &crate::v2::scene::transform::BTransformRule,
        &crate::v2::part::sidebar::show::BSidebarOpenRule,
        &crate::v2::scene::xform::BTransformOpRule,
        &fold::BPropsFoldRule,
        &space::BSpaceStepRule,
        &menu::BReportRule,
        &menu::BFlipRule,
        &crate::v2::modal::command::palette::BPaletteDismissRule,
        &crate::v2::view::draw::canvas::image::zoom::BImageZoomStepRule,
        &back::BReaderOpenRule,
        &back::BReaderCloseRule,
        &pick::BSpacePickedRule,
        &op::BSortRule,
        &op::BNavRule,
        &op::BPurposeRule,
        &op::BBookmarkAddRule,
        &op::BSelectRule,
    ]
}

/// Run every rule that wants `path`.
pub fn after_set(state: &mut BState, path: &BPath, value: &str) {
    for rule in table() {
        if rule.wants(path) {
            rule.after(path, value, state);
        }
    }
}
