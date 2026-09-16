//! Profile modal state - tracks user info and selected action.
//!
//! Store counterpart: `store/modals/profile.rs`

pub mod ctx;
pub mod def;
pub mod r#use;

use crate::bui::BView;
use gpui::App;

use crate::store::BProfileType;

/// Runtime state for the Profile modal.
#[derive(Clone, Debug)]
pub struct ProfileState {
    pub user: Option<String>,
    pub selected_action: Option<BProfileType>,
}

/// Render the Profile modal body as BView data.
pub fn profile_modal_view(_state: &ProfileState, _cx: &App) -> Vec<BView> {
    let mut items: Vec<BView> = vec![BView::Row {
        gap: Some(8.0),
        children: vec![BView::label("Guest User"), BView::label("Not signed in")],
    }];
    for &action in BProfileType::all() {
        items.push(BView::label_icon(action.label(), action.icon()));
    }
    items
}
