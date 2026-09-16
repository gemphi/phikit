//! Help modal state - tracks selected link.
//!
//! Store counterpart: `store/modals/help.rs`

pub mod ctx;
pub mod def;
pub mod r#use;

use crate::bui::BView;
use gpui::App;

use crate::store::BHelpType;

/// Runtime state for the Help modal.
#[derive(Clone, Debug)]
pub struct HelpState {
    pub selected_link: Option<BHelpType>,
}

/// Render the Help modal body as BView data.
pub fn help_view(state: &HelpState, _cx: &App) -> Vec<BView> {
    let links = BHelpType::all();
    let mut items: Vec<BView> = vec![BView::label("Help")];

    // If a link is selected, render its doc body instead of the list.
    if let Some(selected) = state.selected_link {
        if let Some(&link) = links.iter().find(|&&l| l == selected) {
            items.push(BView::label_icon(link.label(), link.icon()));
            items.push(BView::separator());
            for paragraph in link.doc().split("\n\n") {
                for line in paragraph.split('\n') {
                    if !line.is_empty() {
                        items.push(BView::label(line));
                    }
                }
                items.push(BView::separator());
            }
            return items;
        }
    }

    // Default: show all help links.
    for &link in links {
        items.push(BView::label_icon(link.label(), link.icon()));
    }
    items.push(BView::separator());
    items.push(BView::label("Phiber v0.1.0 - Built with GPUI + Rust"));
    items
}
