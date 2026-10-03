//! Primitive dispatcher — thin delegation-only router.
//! All GPUI builder bodies live in focused sub-modules.
//! Public signatures are preserved for compatibility with `gpui/mod.rs::walk`.
//!
//! Each focused primitive module defines a named B-prefixed struct that mirrors
//! its `BView` concept and exposes an inherent `render` method. The dispatcher
//! constructs that struct and delegates without owning GPUI composition.

use super::ctx::BRenderCtx;
use crate::v2::view::BView;
use crate::v2::view::prim::{
    BAction, BColAlign, BRowShade, BSortDirection, BTableCell,
};
use gpui::AnyElement;

/// Flex-row bounded primitive.
pub fn row(list: &[BView], ctx: &BRenderCtx) -> AnyElement {
    super::row::BRow(list).render(ctx)
}

/// File-browser style list row with shade.
pub fn list_row(list: &[BView], shade_role: BRowShade, ctx: &BRenderCtx) -> AnyElement {
    super::list::BList {
        list,
        shade: shade_role,
    }
    .render(ctx)
}

/// Table body row with shared column geometry.
pub fn table_row(
    cell_list: &[BTableCell],
    shade_role: BRowShade,
    action: Option<&BAction>,
    ctx: &BRenderCtx,
) -> AnyElement {
    super::table::BTableRow {
        cell_list,
        shade: shade_role,
        action,
    }
    .render(ctx)
}

/// Table header row with shared column geometry.
pub fn table_header(cell_list: &[BTableCell], ctx: &BRenderCtx) -> AnyElement {
    super::table::BTableHeader(cell_list).render(ctx)
}

/// Sort header for a table column.
pub fn sort_header(
    label: &str,
    direction: Option<BSortDirection>,
    align: BColAlign,
    action: &BAction,
    ctx: &BRenderCtx,
) -> AnyElement {
    super::sort::BSort {
        label,
        direction,
        align,
        action,
    }
    .render(ctx)
}

/// Render-pass-aware indent used by the BView dispatcher.
pub(super) fn indent_resolved(level: u16, ctx: &BRenderCtx) -> AnyElement {
    super::tree::BTreeIndent(level).render(ctx)
}

/// Outliner tree row with guides, disclosure, icon, name, and trail.
#[allow(clippy::too_many_arguments)]
pub fn tree_row(
    depth: u16,
    expanded: Option<bool>,
    toggle: Option<&BAction>,
    lead: &[BView],
    icon: &str,
    (list, trail): (&[BView], &[BView]),
    shade_role: BRowShade,
    drag: Option<&crate::v2::view::prim::BTreeDrag>,
    ctx: &BRenderCtx,
) -> AnyElement {
    super::tree::BTreeRow {
        depth,
        expanded,
        toggle,
        lead,
        icon,
        list,
        trail,
        shade: shade_role,
        drag,
    }
    .render(ctx)
}

/// Flex-col filling its slot.
pub fn column(list: &[BView], ctx: &BRenderCtx) -> AnyElement {
    super::column::BColumn(list).render(ctx)
}
