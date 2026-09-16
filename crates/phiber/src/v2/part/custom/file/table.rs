//! Shared File Browser table composition for host and modal clients.

use super::{keys, listing};
use crate::v2::state::BState;
use crate::v2::view::{
    BAction, BColAlign, BColWidth, BPanelRole, BRowShade, BSortDirection, BTableCell, BView,
};

use super::listing::{BFileRow, BFileSort};

fn sort_action(state: &BState, column: BFileSort) -> BAction {
    if listing::sort_mode(state) == column {
        BAction::Toggle(keys::k_sort_invert())
    } else {
        BAction::Set(keys::k_sort(), column.label().to_string())
    }
}

fn sort_direction(state: &BState, column: BFileSort) -> Option<BSortDirection> {
    (listing::sort_mode(state) == column).then(|| {
        if listing::sort_inverted(state) {
            BSortDirection::Descending
        } else {
            BSortDirection::Ascending
        }
    })
}

fn header_cell(
    state: &BState,
    column: BFileSort,
    width: BColWidth,
    align: BColAlign,
) -> BTableCell {
    BTableCell::new(
        BView::sort_header(
            column.label(),
            sort_direction(state, column),
            align,
            sort_action(state, column),
        ),
        width,
        align,
    )
}

pub fn header(state: &BState) -> BView {
    BView::table_header(vec![
        header_cell(state, BFileSort::Name, BColWidth::Flex, BColAlign::Start),
        header_cell(
            state,
            BFileSort::Time,
            BColWidth::FileDate,
            BColAlign::Start,
        ),
        header_cell(state, BFileSort::Size, BColWidth::FileSize, BColAlign::End),
    ])
}

pub fn row(file: &BFileRow, selected_path: &str) -> BView {
    let shade = if file.path == selected_path {
        BRowShade::Selected
    } else {
        BRowShade::Flat
    };
    let action = if file.is_dir() {
        BAction::Set(keys::k_path(), file.path.clone())
    } else {
        BAction::Set(keys::k_selected(), file.path.clone())
    };
    let name = BView::row(vec![
        BView::icon(file.icon()),
        BView::label(file.name.clone()),
    ]);
    BView::table_row(
        vec![
            BTableCell::flex(name),
            BTableCell::new(
                BView::label(listing::format_time(file)),
                BColWidth::FileDate,
                BColAlign::Start,
            ),
            BTableCell::new(
                BView::label(listing::format_size(file)),
                BColWidth::FileSize,
                BColAlign::End,
            ),
        ],
        shade,
        Some(action),
    )
}

fn single(view: BView) -> BView {
    BView::table_row(vec![BTableCell::flex(view)], BRowShade::Flat, None)
}

pub fn view(state: &BState) -> BView {
    let selected = state.get(&keys::k_selected()).unwrap_or("");
    let mut children: Vec<BView> = vec![header(state)];
    match listing::rows_for(state) {
        Ok(entries) if entries.is_empty() => {
            children.push(single(BView::label("No files in this directory")));
        }
        Ok(entries) => {
            for file in &entries {
                children.push(row(file, selected));
            }
        }
        Err(why) => {
            children.push(single(BView::row(vec![
                BView::icon("alert_triangle"),
                BView::label(why),
            ])));
        }
    };
    // The header is a BView::TableHeader — BBody's renderer pulls it out of
    // the children and renders it sticky (inside the outer surface div, above
    // the bounded scroll container) so Name/Date/Size stays visible while
    // file rows scroll. One surface, one border, one radius.
    BView::panel(BPanelRole::Body, children)
}
