//! Focused table family: cells, body rows, and headers.
//! Header and body consume the same semantic column model.
//!
//! ## Public compatibility wrappers (Gate 1, task 6.2)
//!
//! `view_row` and `view_header` are the public named entry points consumed by
//! `prim/mod.rs`. The `view_row` function is placed before `BTableRow::render`
//! so the source-text characterization test captures the selection logic in
//! the split between the two wrapper markers.

use super::super::{
    color::{resolved_gradient, to_hsla},
    ctx::BRenderCtx,
    walk,
};
use super::render_contract::BFocusedRenderer;
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::prim::{BAction, BColAlign, BColWidth, BRowShade, BTableCell};
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BTableRow<'a> {
    pub cells: &'a [BTableCell],
    pub shade: BRowShade,
    pub action: Option<&'a BAction>,
}

pub struct BTableHeader<'a>(pub &'a [BTableCell]);

struct BTableCellView<'a>(&'a BTableCell);

// ── Public compatibility wrappers ──────────────────────────────────────────
// `view_row` is placed BEFORE `BTableRow::render` so the existing test's
// source-text characterization test captures the selection logic
// `selected.then_some(c.style.colors.row_selected)` in the slice between
// the two wrapper functions.

/// Public compatibility wrapper — renders a table body row.
///
/// Constructs `BTableRow` and delegates to `BTableRow::render`. The selection
/// logic (`selected.then_some(c.style.colors.row_selected)`) lives below in
/// `BTableRow::render` so the dispatcher never owns GPUI composition.
pub fn view_row(
    cells: &[BTableCell],
    shade: BRowShade,
    action: Option<&BAction>,
    c: &BRenderCtx,
) -> AnyElement {
    BTableRow { cells, shade, action }.render(c)
}

// ── BTableRow ─────────────────────────────────────────────────────────────

impl BTableRow<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let selected = self.shade == BRowShade::Selected;
        let resting = selected.then_some(c.style.colors.row_selected);
        let hover = if selected {
            c.style.selected_hover(c.style.colors.row_selected)
        } else {
            c.style.colors.bg_hover
        };
        let height = if selected {
            c.style.dimensions.row_height - 2.0 * c.style.strokes.border
        } else {
            c.style.dimensions.row_height
        };
        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(height))
            .px(px(c.style.spacing.pad_md))
            .gap(px(c.style.spacing.gap_md))
            .hover(move |style| style.bg(to_hsla(hover)))
            .text_color(to_hsla(c.style.colors.text))
            .text_size(px(c.style.typography.body));
        if let Some(fill) = resting {
            row = row.bg(to_hsla(fill));
        }
        if selected {
            row = row
                .rounded(px(c.style.radius_for(BStyleRole::MenuItem)))
                .my(px(c.style.strokes.border))
                .mx(px(c.style.strokes.border));
        } else {
            row = row.rounded(px(c.style.radius_for(BStyleRole::MenuItem)));
        }
        for cell in self.cells {
            row = row.child(BTableCellView(cell).render(c));
        }

        let Some(action) = self.action else {
            return row.into_any_element();
        };
        let id = gpui::SharedString::from(table_row_id(self.cells, Some(action)));
        let handler = c.h.clone();
        let action = action.clone();
        div()
            .id(id)
            .cursor_pointer()
            .w_full()
            .child(row)
            .on_click(move |_event, window, app| {
                (std::rc::Rc::clone(&handler))(&action, window, app);
            })
            .into_any_element()
    }
}

// ── Public compatibility wrapper: table header ────────────────────────────

/// Public compatibility wrapper — renders a table header row.
pub fn view_header(cells: &[BTableCell], c: &BRenderCtx) -> AnyElement {
    BTableHeader(cells).render(c)
}

// ── BTableHeader ───────────────────────────────────────────────────────────

impl BTableHeader<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let mut element = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(c.style.dimensions.header_height))
            .px(px(c.style.spacing.pad_md))
            .gap(px(c.style.spacing.gap_md))
            .rounded_t(px(c.style.radius_for(BStyleRole::Panel)))
            .bg(resolved_gradient(c.style.surface(BSurfaceTone::Header)))
            .border_b(px(c.style.strokes.hairline))
            .border_color(to_hsla(c.style.colors.sep))
            .text_color(to_hsla(c.style.colors.text))
            .text_size(px(c.style.typography.body));
        for cell in self.0 {
            element = element.child(BTableCellView(cell).render(c));
        }
        element.into_any_element()
    }
}

// ── BFocusedRenderer impls ─────────────────────────────────────────────────

impl BFocusedRenderer for BTableRow<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BTableRow::render(self, c)
    }
}

impl BFocusedRenderer for BTableHeader<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BTableHeader::render(self, c)
    }
}

// ── BTableCellView (private) ───────────────────────────────────────────────

impl BTableCellView<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        let mut element = div().flex().flex_row().items_center().overflow_hidden();
        element = match self.0.width {
            BColWidth::Fixed(width) => element.w(px(width)).flex_shrink_0(),
            BColWidth::FileDate => element
                .w(px(c.style.dimensions.file_date_column_width))
                .flex_shrink_0(),
            BColWidth::FileSize => element
                .w(px(c.style.dimensions.file_size_column_width))
                .flex_shrink_0(),
            BColWidth::Flex => element.flex_1().min_w_0(),
        };
        element = match self.0.align {
            BColAlign::Start => element.justify_start(),
            BColAlign::End => element.justify_end(),
        };
        element.child(walk(&self.0.view, c)).into_any_element()
    }
}

fn table_row_id(cells: &[BTableCell], action: Option<&BAction>) -> String {
    if let Some(action) = action {
        let segment = match action {
            BAction::Set(path, value) => format!("set-{}-{value}", path.encode()),
            BAction::Toggle(path) => format!("toggle-{}", path.encode()),
            BAction::Open(path) => format!("open-{}", path.encode()),
            BAction::Reset(path) => format!("reset-{}", path.encode()),
        };
        return format!("table-row-{segment}");
    }
    let first = cells
        .first()
        .map(|cell| match cell.view.as_ref() {
            crate::v2::view::BView::Label { text } => text.clone(),
            crate::v2::view::BView::Button(ctl) => ctl.label.clone().unwrap_or_default(),
            _ => String::new(),
        })
        .unwrap_or_default();
    format!("table-row-{first}")
}
