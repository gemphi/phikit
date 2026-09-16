//! Panel-family scroll identity helpers.

use crate::v2::view::BView;

/// A stable, content-derived id for a scrollable panel.
pub(super) fn scroll_id(prefix: &str, children: &[BView]) -> gpui::SharedString {
    let tag = children.first().map(first_text).unwrap_or_default();
    gpui::SharedString::from(format!("{prefix}-{tag}"))
}

fn first_text(v: &BView) -> String {
    match v {
        BView::Label { text } | BView::VLabel { text } | BView::Field { text } => text.clone(),
        BView::Button(ctl) => ctl.label.clone().unwrap_or_default(),
        BView::Panel { children, .. } | BView::Column { children } | BView::Row { children } => {
            children.first().map(first_text).unwrap_or_default()
        }
        BView::TableHeader { cells } | BView::TableRow { cells, .. } => cells
            .first()
            .map(|cell| first_text(&cell.view))
            .unwrap_or_default(),
        BView::SortHeader { label, .. } => label.clone(),
        BView::AccordionHeader { title, .. } => title.clone(),
        _ => String::new(),
    }
}
