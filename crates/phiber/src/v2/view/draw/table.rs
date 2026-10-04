//! Focused table family: cells, body rows, and headers.
//! Header and body consume the same semantic column model.
//!
//! ## Public compatibility wrappers (Gate 1, task 6.2)
//!
//! `view_row` and `view_header` are the public named entry points consumed by
//! `prim/mod.rs`. The `view_row` function is placed before `BTableRow::render`
//! so the source-text characterization test captures the selection logic in
//! the split between the two wrapper markers.

use super::ctx::BRenderCtx;
use super::flex::{
    BAlign, BDim, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BRadius, BSpace, BType,
};
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


// ── Public compatibility wrapper: table header ────────────────────────────

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
        for cell in self.0 {
            element = element.child(BTableCellView(cell).render(ctx));
        }
        element.into_any_element()
    }
}

// ── BFocusedRenderer impls ─────────────────────────────────────────────────

// ── BTableCellView (private) ───────────────────────────────────────────────

impl BTableCellView<'_> {
    fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        BTablePart::Cell(self.0.width, self.0.align)
            .flex(ctx)
            .div(ctx)
            .child(walk(&self.0.view, ctx))
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
    /// A column heading is a card heading: same radius, same fill as a panel
    /// header, so the file list reads as one more rounded section rather than
    /// a table of its own. No rule under it — the tone change is the seam, as
    /// it is under a panel header.
    Header,
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
            Self::Clickable => {
                BFlex::row().width(BLen::Full).height(BLen::Measured(row_band(ctx)))
            }
            Self::Header => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Dim(BDim::CardHead))
                .align(BAlign::Center)
                // `CardInset + PadSm == PadMd`, so a heading's text lands in
                // the same column as the row text under it while its fill
                // still spans the whole column.
                .pad_x(BSpace::CardInset)
                .gap(BSpace::None)
                .radius(BRadius::Top(ctx.style.radius_for(BStyleRole::Card)))
                .fill(BFill::Heading(BSurfaceTone::Card))
                .ink()
                .type_size(BType::Body),
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
