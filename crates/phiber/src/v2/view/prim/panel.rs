//! Panel primitives — the theming tag on `BView::Panel`.

use crate::v2::view::BView;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPanelRole {
    Header,
    Body,
    Footer,
    Card,
    CardHeader,
    Section,
    Toolbar,
    Sidebar,
    Viewport,
    Modal,
}

impl BView {
    pub fn panel(role: BPanelRole, children: Vec<BView>) -> Self {
        Self::Panel { role, children }
    }
}
