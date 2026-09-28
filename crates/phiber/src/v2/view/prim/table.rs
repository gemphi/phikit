//! Table primitives — cells with shared column geometry so header and body
//! rows line up (Blender `space_file` `draw_details_columns`).

use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BRowShade};

/// Column sizing for a `TableRow` cell. Blender's `space_file`
/// (`draw_details_columns`) gives Name a flexible width and Date / Size
/// fixed widths.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BColWidth {
    /// Grows to fill the leftover width (`flex_1 min_w_0`).
    Flex,
    /// A caller-specified fixed device-independent-pixel width.
    Fixed(f32),
    /// File Browser Date column, resolved from the render-pass style.
    FileDate,
    /// File Browser Size column, resolved from the render-pass style.
    FileSize,
}

/// Horizontal alignment of a `TableRow` cell's content. `Start` for text
/// columns, `End` to right-align the Size column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BColAlign {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSortDirection {
    Ascending,
    Descending,
}

/// One cell of a `TableRow` / table header: a boxed sub-view plus the shared
/// column geometry (width + alignment) that keeps the header and every body
/// row lined up in the same columns.
#[derive(Clone, Debug)]
pub struct BTableCell {
    pub view: Box<BView>,
    pub width: BColWidth,
    pub align: BColAlign,
}

impl BTableCell {
    pub fn new(view: BView, width: BColWidth, align: BColAlign) -> Self {
        Self {
            view: Box::new(view),
            width,
            align,
        }
    }
    /// Flexible, left-aligned cell (the Name column).
    pub fn flex(view: BView) -> Self {
        Self::new(view, BColWidth::Flex, BColAlign::Start)
    }
    /// Fixed-width, left-aligned cell (the Date column).
    pub fn fixed(view: BView, w: f32) -> Self {
        Self::new(view, BColWidth::Fixed(w), BColAlign::Start)
    }
    /// Fixed-width, right-aligned cell (the Size column).
    pub fn fixed_end(view: BView, w: f32) -> Self {
        Self::new(view, BColWidth::Fixed(w), BColAlign::End)
    }
}

impl BView {
    /// A table body row: cells that share the header's column geometry, a
    /// background shade, and an optional `action` that makes the WHOLE row
    /// clickable (Blender rows select on click anywhere).
    pub fn table_row(cell_list: Vec<BTableCell>, shade: BRowShade, action: Option<BAction>) -> Self {
        Self::TableRow {
            cell_list,
            shade,
            action,
        }
    }
    /// The table header row — same column widths as the body rows so columns
    /// align. Header cells carry the sort actions.
    pub fn table_header(cell_list: Vec<BTableCell>) -> Self {
        Self::TableHeader { cell_list }
    }
    pub fn sort_header(
        label: impl Into<String>,
        direction: Option<BSortDirection>,
        align: BColAlign,
        action: BAction,
    ) -> Self {
        Self::SortHeader {
            label: label.into(),
            direction,
            align,
            action,
        }
    }
}
