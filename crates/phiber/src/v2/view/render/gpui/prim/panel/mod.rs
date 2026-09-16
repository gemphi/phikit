//! Panel-role dispatcher — delegates to focused role modules only.
//! No GPUI builder bodies appear here.

mod body;
mod card;
mod footer;
mod header;
mod heading;
mod modal;
mod scroll;
mod section;
mod sidebar;
pub(super) mod style;
mod toolbar;
mod viewport;

use super::super::ctx::BRenderCtx;
use crate::v2::view::BView;
use crate::v2::view::prim::BPanelRole;
use gpui::AnyElement;

/// Dispatch by `BPanelRole` to its focused named renderer.
pub fn view(role: BPanelRole, children: &[BView], c: &BRenderCtx) -> AnyElement {
    match role {
        BPanelRole::Section => section::BSection(children).render(c),
        BPanelRole::CardHeader => heading::BCardHeader(children).render(c),
        BPanelRole::Card => card::BCard(children).render(c),
        BPanelRole::Header => header::BHeader(children).render(c),
        BPanelRole::Toolbar => toolbar::BToolbar(children).render(c),
        BPanelRole::Sidebar => sidebar::BSidebar(children).render(c),
        BPanelRole::Footer => footer::BFooter(children).render(c),
        BPanelRole::Body => body::BBody(children).render(c),
        BPanelRole::Viewport => viewport::BViewport(children).render(c),
        BPanelRole::Modal => modal::BModal(children).render(c),
    }
}
