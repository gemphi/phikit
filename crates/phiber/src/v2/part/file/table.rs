//! Shared File Browser table composition for host and modal clients.

use crate::v2::state::BKey as _;
use crate::v2::state::FileKey;
use super::listing;
use crate::v2::state::BState;
use crate::v2::view::prim::{BCard, BColumn};
use crate::v2::view::{
    BAction, BColAlign, BColWidth, BRowShade, BSortDirection, BTableCell, BView,
};

use super::listing::BFileRow;
use super::listing::BFileSort;

fn sort_action(state: &BState, column: BFileSort) -> BAction {
    if listing::sort_mode(state) == column {
        BAction::Toggle(FileKey::SortInvert.path())
    } else {
        BAction::Set(FileKey::Sort.path(), column.label().to_string())
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
            column.heading(),
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

/// One file's row; `odd` rows take the stripe, as Blender's list does.
pub fn row(state: &BState, file: &BFileRow, selected_path: &str, odd: bool) -> BView {
    let shade = match (file.path == selected_path, odd) {
        (true, _) => BRowShade::Selected,
        (false, true) => BRowShade::Alt,
        (false, false) => BRowShade::Flat,
    };
    let action = if file.is_dir() {
        BAction::Set(FileKey::Path.path(), file.path.clone())
    } else {
        super::op::pick_or_commit(state, &file.path)
    };
    let name = BView::row(vec![
        BView::icon(file.icon()),
        BView::label(file.name.clone()),
    ]);
    BView::table_row(
        vec![
            BTableCell::flex(name),
            BTableCell::new(
                BView::label(BFileRow::format_time(&file)),
                BColWidth::FileDate,
                BColAlign::Start,
            ),
            BTableCell::new(
                BView::label(BFileRow::format_size(&file)),
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
    if super::thumb::active(state) {
        return super::thumb::view(state);
    }
    let selected = super::op::picked(state);
    let mut list: Vec<BView> = vec![header(state)];
    match listing::rows_for(state) {
        Ok(entries) if entries.is_empty() => {
            list.push(single(BView::label("No files in this directory")));
        }
        Ok(entries) => {
            for (n, file) in entries.iter().enumerate() {
                list.push(row(state, file, selected, n % 2 == 1));
            }
        }
        Err(why) => {
            list.push(single(BView::row(vec![
                BView::icon("alert_triangle"),
                BView::label(why),
            ])));
        }
    };
    // The header is a BView::TableHeader — BBody's renderer pulls it out of
    // the children and renders it sticky (inside the outer surface div, above
    // the bounded scroll container) so Name/Date/Size stays visible while
    // file rows scroll. One surface, one border, one radius.
    BView::from(BCard::column(BColumn::Body, list))
}
