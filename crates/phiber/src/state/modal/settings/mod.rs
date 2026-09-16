//! Settings modal state - tracks active category and search query.
//!
//! Store counterpart: `store/modals/settings.rs`

pub mod ctx;
pub mod def;
pub mod r#use;

use crate::bui::BView;
use gpui::App;

use crate::store::BSettingsType;

/// Runtime state for the Settings modal.
#[derive(Clone, Debug)]
pub struct SettingsState {
    pub active_category: BSettingsType,
    pub search: String,
}

/// Render the Settings modal body as BView data.
pub fn settings_modal_view(_state: &SettingsState, _cx: &App) -> Vec<BView> {
    let mut sidebar: Vec<BView> = Vec::new();
    for &cat in BSettingsType::all() {
        sidebar.push(BView::label(cat.label()));
    }

    vec![BView::Row {
        gap: Some(8.0),
        children: vec![
            BView::Column {
                gap: Some(2.0),
                children: sidebar,
                center: false,
            },
            BView::Column {
                gap: Some(8.0),
                children: vec![BView::label("Select a category from the sidebar")],
                center: false,
            },
        ],
    }]
}
