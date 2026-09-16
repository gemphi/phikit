//! Modal view builder — header, sidebar navigation, and content.
//! Corresponds to store/modals/ in the store hierarchy.

use super::bview::{BAction, CardShade};
use super::theme::BPalette;
use super::{BKey, BState, BView};
use crate::store::modals::{BHelpType, BModalType, BProfileType, BSettingsType};

pub struct ModalView<'a> {
    pub modal: BModalType,
    pub state: &'a BState,
}

impl<'a> ModalView<'a> {
    pub fn new(modal: BModalType, state: &'a BState) -> Self {
        Self { modal, state }
    }

    pub fn build(self) -> BView {
        let modal = self.modal;
        let state = self.state;
        let title = modal.label();

        let sidebar_items: Vec<BView> = match modal {
            BModalType::Settings => BSettingsType::all()
                .iter()
                .map(|&cat| {
                    BView::button()
                        .with_icon("settings")
                        .with_label(cat.label())
                })
                .collect(),
            BModalType::Help => BHelpType::all()
                .iter()
                .map(|&h| BView::button().with_icon(h.icon()).with_label(h.label()))
                .collect(),
            BModalType::Profile => BProfileType::all()
                .iter()
                .map(|&p| BView::button().with_icon(p.icon()).with_label(p.label()))
                .collect(),
            BModalType::FileBrowser => vec![
                BView::button().with_icon("folder").with_label("Assets"),
                BView::button().with_icon("folder").with_label("Home"),
                BView::button().with_icon("folder").with_label("Desktop"),
            ],
        };

        let content_items: Vec<BView> = match modal {
            BModalType::Settings => BSettingsType::all()
                .iter()
                .flat_map(|&cat| {
                    let children: Vec<BView> = cat
                        .rows()
                        .iter()
                        .map(|(k, v)| BView::prop_row(*k, *v))
                        .collect();
                    vec![BView::section(cat.label(), None, true, children)]
                })
                .collect(),
            BModalType::Help => BHelpType::all()
                .iter()
                .map(|&h| BView::prop_row(h.label(), h.icon()))
                .collect(),
            BModalType::Profile => BProfileType::all()
                .iter()
                .map(|&p| BView::prop_row(p.label(), p.icon()))
                .collect(),
            BModalType::FileBrowser => Self::file_browser_items(state),
        };

        let sidebar = BView::column(None, sidebar_items);
        let content_area = BView::scroll(content_items);
        let body = BView::row(None, vec![sidebar, content_area]);

        BView::card()
            .with_shade(CardShade::App)
            .with_header(title)
            .with_tool(
                BView::button()
                    .with_icon("x")
                    .with_action(BAction::Toggle(BKey::Modal)),
            )
            .with_body(body)
    }

    fn file_browser_items(state: &BState) -> Vec<BView> {
        let default_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let current_path = state
            .get(&BKey::FilePath)
            .unwrap_or(default_path.to_str().unwrap_or("."));

        let mut items: Vec<BView> = Vec::new();
        let header_row = vec![
            BView::button()
                .with_icon("arrow-up")
                .with_tooltip("Go up")
                .with_action(BAction::Set(BKey::FilePath, Self::parent_dir(current_path))),
            BView::label(current_path),
        ];
        items.push(BView::row(
            Some(BPalette::from_state(state).gap_sm),
            header_row,
        ));

        let dir = std::path::Path::new(current_path);
        let mut entries: Vec<(String, bool, u64)> = match std::fs::read_dir(dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') {
                        return None;
                    }
                    let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                    Some((name, is_dir, size))
                })
                .collect(),
            Err(_) => {
                items.push(BView::label("(cannot read directory)"));
                return items;
            }
        };

        entries.sort_by(|a, b| match (a.1, b.1) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        });

        let mut file_rows: Vec<BView> = Vec::new();
        for (name, is_dir, _) in &entries {
            let icon = if *is_dir { "folder" } else { "file" };
            let full_path = dir.join(name).to_string_lossy().to_string();
            if *is_dir {
                file_rows.push(
                    BView::button()
                        .with_icon(icon)
                        .with_label(name)
                        .with_action(BAction::Set(BKey::FilePath, full_path)),
                );
            } else {
                let selected = state.get(&BKey::FileSelected) == Some(full_path.as_str());
                file_rows.push(
                    BView::button()
                        .with_icon(icon)
                        .with_label(name)
                        .with_active(selected)
                        .with_action(BAction::Set(BKey::FileSelected, full_path)),
                );
            }
        }

        if file_rows.is_empty() {
            items.push(BView::label("(empty directory)"));
        } else {
            items.push(BView::column(
                Some(BPalette::from_state(state).gap_xs),
                file_rows,
            ));
        }

        let selected_path = state.get(&BKey::FileSelected).unwrap_or("");
        items.push(BView::row(
            Some(BPalette::from_state(state).gap_sm),
            vec![
                BView::button()
                    .with_icon("folder-open")
                    .with_label("Open")
                    .with_action(BAction::OpenFile(selected_path.to_string())),
                BView::button()
                    .with_icon("x")
                    .with_label("Cancel")
                    .with_action(BAction::Toggle(BKey::Modal)),
            ],
        ));

        items
    }

    fn parent_dir(path: &str) -> String {
        std::path::Path::new(path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string())
    }
}
