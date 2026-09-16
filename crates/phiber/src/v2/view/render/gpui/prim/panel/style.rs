//! Panel-family semantic surface and spacing helpers.
//! Used only within the `prim/panel` family.

use super::super::super::{color::resolved_gradient, ctx::BRenderCtx};
use crate::v2::host::BHostType;
use crate::v2::theme::BSurfaceTone;
use crate::v2::view::prim::BPanelRole;
use gpui::Background;

/// Resolve the role-specific panel surface from the render-pass style.
/// Properties deliberately collapses Body, Sidebar, and Card to one pane
/// surface while other hosts retain the semantic surface ladder.
pub(super) fn panel_bg(role: BPanelRole, c: &BRenderCtx) -> Background {
    let properties = c.host == BHostType::Buts;
    let tone = match role {
        BPanelRole::Header | BPanelRole::Toolbar | BPanelRole::CardHeader => BSurfaceTone::Header,
        BPanelRole::Footer => BSurfaceTone::Footer,
        BPanelRole::Sidebar if properties => BSurfaceTone::Body,
        BPanelRole::Sidebar => BSurfaceTone::Sidebar,
        BPanelRole::Body => BSurfaceTone::Body,
        BPanelRole::Viewport => BSurfaceTone::Viewport,
        BPanelRole::Modal => BSurfaceTone::Modal,
        BPanelRole::Card if properties => BSurfaceTone::Body,
        BPanelRole::Card => BSurfaceTone::Card,
        BPanelRole::Section => BSurfaceTone::Section,
    };
    resolved_gradient(c.style.surface(tone))
}

/// Panel edge to contained controls.
pub(super) fn surface_gutter(c: &BRenderCtx) -> f32 {
    c.style.spacing.pad_md
}

/// Panel edge to text inside a padded control.
pub(super) fn text_gutter(c: &BRenderCtx) -> f32 {
    c.style.spacing.pad_md + c.style.spacing.pad_sm
}
