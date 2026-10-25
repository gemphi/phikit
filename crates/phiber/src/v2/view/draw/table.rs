//! Focused table family: cells, body rows, and headers, on one column model.
//! `view_row` and `view_header` are the named entry points; `view_row` sits
//! before `BTableRow::render` so the source-text test sees the selection
//! logic between the two wrappers.

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BRadius, BSpace, BType};
use super::walk;
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::prim::{BAction, BColAlign, BColWidth, BRowShade, BTableCell};
use gpui::prelude::*;
use gpui::AnyElement;

pub struct BTableRow<'a> {
    pub cell_list: &'a [BTableCell],
    pub shade: BRowShade,
    pub action: Option<&'a BAction>,
}

pub struct BTableHeader<'a>(pub &'a [BTableCell]);

struct BTableCellView<'a>(&'a BTableCell);

/// Renders a table body row through `BTableRow::render`, which owns the
/// selection logic, so the dispatcher never owns GPUI composition.
#[allow(dead_code)]
pub fn view_row(
    cell_list: &[BTableCell],
    shade: BRowShade,
    action: Option<&BAction>,
    ctx: &BRenderCtx,
) -> AnyElement {
    BTableRow { cell_list, shade, action }.render(ctx)
}

// ── BTableRow ─────────────────────────────────────────────────────────────

impl BTableRow<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let mut row = BTablePart::Row(self.shade).flex(ctx).div(ctx);
        for cell in self.cell_list {
            row = row.child(BTableCellView(cell).render(ctx));
        }

        let Some(action) = self.action else {
            return row.into_any_element();
        };
        let id = gpui::SharedString::from(table_row_id(self.cell_list, Some(action)));
        let handler = ctx.handler.clone();
        let action = action.clone();
        let dbg = id.clone();
        BTablePart::Clickable
            .flex(ctx)
            .div(ctx)
            .id(id)
            .debug_selector(move || dbg.to_string())
            .cursor_pointer()
            .child(row)
            .on_click(move |_event, window, app| {
                (std::rc::Rc::clone(&handler))(&action, window, app);
            })
            .into_any_element()
    }
}

/// Public compatibility wrapper — renders a table header row.
#[allow(dead_code)]
pub fn view_header(cell_list: &[BTableCell], ctx: &BRenderCtx) -> AnyElement {
    BTableHeader(cell_list).render(ctx)
}

// ── BTableHeader ───────────────────────────────────────────────────────────

impl BTableHeader<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let mut element = BTablePart::Header
            .flex(ctx)
            .div(ctx)
            .debug_selector(|| "table-header".to_string());
        // Each heading cell is its column and the gap before it, with no
        // divider: hovered, the whole cell takes the hover, and its label
        // starts (or, for a right-aligned column, ends) where the values do.
        let last = self.0.len().saturating_sub(1);
        for (n, cell) in self.0.iter().enumerate() {
            let head = BTablePart::HeadCell(cell.width, n == 0, n == last).flex(ctx).div(ctx).child(walk(&cell.view, ctx));
            element = element.child(head);
        }
        element.into_any_element()
    }
}

impl BTableCellView<'_> {
    fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        BTablePart::Cell(self.0.width, self.0.align)
            .flex(ctx)
            .div(ctx)
            .child(walk(&self.0.view, &ctx.clipping()))
            .into_any_element()
    }
}

fn table_row_id(cell_list: &[BTableCell], action: Option<&BAction>) -> String {
    if let Some(action) = action {
        let segment = match action {
            BAction::Set(path, value) => format!("set-{}-{value}", path.encode()),
            BAction::Adjust(path, value) => format!("adjust-{}-{value}", path.encode()),
            BAction::Toggle(path) => format!("toggle-{}", path.encode()),
            BAction::Open(path) => format!("open-{}", path.encode()),
            BAction::Reset(path) => format!("reset-{}", path.encode()),
        };
        return format!("table-row-{segment}");
    }
    let first = cell_list
        .first()
        .map(|cell| match cell.view.as_ref() {
            crate::v2::view::BView::Label { text } => text.clone(),
            crate::v2::view::BView::Button(ctl) => ctl.label.clone().unwrap_or_default(),
            _ => String::new(),
        })
        .unwrap_or_default();
    format!("table-row-{first}")
}

/// The parts of a table — one enum for the level, as every other level
/// states its parts (`BCardPart`, `BModalPart`, `BLogPart`).
enum BTablePart {
    /// The band a row is drawn in.
    ///
    /// One geometry, selected or not: a row used to shrink by a border and
    /// gain a margin when it became selected, so selecting or hovering one
    /// nudged it and everything below it — the "dancing" on hover. State may
    /// change what a row paints; it may never change what it measures, which
    /// is why the inset is a `Stroke` margin the row always carries.
    Row(BRowShade),
    /// The clickable is the row: its height is the row's, so the hover and
    /// selection fills cover the whole band rather than a strip of it.
    Clickable,
    /// A column heading is a band over the rows, laid as they are: the same
    /// inset from the list's edge, the same rounding, a heading's fill.
    Header,
    /// A column's heading cell: the body cell's column, the gap before it
    /// (after the first), and the row's inset at either end (first, last) —
    /// so its hover covers all of it and its label sits over its values.
    HeadCell(BColWidth, bool, bool),
    /// A cell fills the row's height and centres its content in it. Sizing to
    /// content left the columns short of the row, so a value sat at whatever
    /// height its own text happened to be.
    Cell(BColWidth, BColAlign),
}

impl BFlexed for BTablePart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match *self {
            Self::Row(shade) => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Measured(row_band(ctx)))
                .align(BAlign::Center)
                .pad_x(BSpace::CardInset)
                .gap(BSpace::GapMd)
                // The fill, the hover and the shade mapping are the tree's
                // and the list's too (`tone::row_fill` / `tone::row_hover`).
                .fill(BFill::Row(shade))
                .hovered(BFill::RowHover(shade))
                .radius(BRadius::Role(BStyleRole::MenuItem))
                .margin_x(BSpace::Stroke)
                .margin_y(BSpace::Stroke)
                .ink()
                .type_size(BType::Body),
            Self::Clickable => BFlex::row().width(BLen::Full).height(BLen::Measured(row_band(ctx))),
            Self::Header => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Dim(BDim::CardHead))
                .align(BAlign::Center)
                .margin_x(BSpace::Stroke)
                .gap(BSpace::None)
                .radius(BRadius::Role(BStyleRole::MenuItem))
                .fill(BFill::Heading(BSurfaceTone::Card))
                .ink()
                .type_size(BType::Body),
            Self::HeadCell(width, lead, last) => {
                let cell = BFlex::row()
                    .align(BAlign::Center)
                    .height(BLen::Full)
                    .min_height(BLen::Zero)
                    .overflow(BOverflow::Clip)
                    .radius(BRadius::Role(BStyleRole::MenuItem))
                    .hovered(BFill::ItemHover)
                    .hover_group(super::sort::HEADING);
                let w = match width {
                    BColWidth::Fixed(w) => Some(w),
                    BColWidth::FileDate => Some(ctx.style.dimensions.file_date_column_width),
                    BColWidth::FileSize => Some(ctx.style.dimensions.file_size_column_width),
                    BColWidth::Flex => None,
                };
                let inset = BSpace::CardInset.px(ctx);
                let gap = ctx.style.spacing.gap_md;
                let cell = match lead {
                    true => cell.pad_left(BSpace::CardInset),
                    false => cell.pad_left(BSpace::GapMd),
                };
                let cell = if last { cell.pad_right(BSpace::CardInset) } else { cell };
                // The cell's span beyond its column: gap before, row's inset.
                let extra = [(!lead, gap), (lead, inset), (last, inset)].iter().filter(|(on, _)| *on).map(|(_, px)| px).sum::<f32>();
                match w {
                    None => cell.fills().min_width(BLen::Zero),
                    Some(w) => cell.width(BLen::Measured(w + extra)).keeps_size(),
                }
            }
            Self::Cell(width, align) => {
                let cell = BFlex::row()
                    .align(BAlign::Center)
                    .height(BLen::Full)
                    .min_height(BLen::Zero)
                    .overflow(BOverflow::Clip)
                    .justify(match align {
                        BColAlign::Start => BJustify::Start,
                        BColAlign::End => BJustify::End,
                    });
                // A right-aligned column ends where its heading's word does,
                // short of the heading's sort arrow.
                let cell = if align == BColAlign::End { cell.pad_right(BSpace::ArrowSlot) } else { cell };
                let fixed = |w: f32| cell.width(BLen::Measured(w)).keeps_size();
                match width {
                    BColWidth::Fixed(w) => fixed(w),
                    BColWidth::FileDate => fixed(ctx.style.dimensions.file_date_column_width),
                    BColWidth::FileSize => fixed(ctx.style.dimensions.file_size_column_width),
                    BColWidth::Flex => cell.fills().min_width(BLen::Zero),
                }
            }
        }
    }
}

/// A row's own height: the row rung less the border it insets itself by at
/// top and bottom, so the band it occupies is exactly one row.
fn row_band(ctx: &BRenderCtx) -> f32 {
    ctx.style.dimensions.row_height - 2.0 * ctx.style.strokes.border
}
