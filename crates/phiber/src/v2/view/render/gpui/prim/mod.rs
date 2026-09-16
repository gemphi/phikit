//! Primitive dispatcher — thin delegation-only router.
//! All GPUI builder bodies live in focused sub-modules.
//! Public signatures are preserved for compatibility with `gpui/mod.rs::walk`.
//!
//! Each focused primitive module defines a named B-prefixed struct that mirrors
//! its `BView` concept and exposes an inherent `render` method. The dispatcher
//! constructs that struct and delegates without owning GPUI composition.

pub mod accordion;
pub mod sort;

mod column;
mod list;
mod panel;
pub(crate) mod render_contract;
mod row;
mod table;
mod tree;

use super::ctx::BRenderCtx;
use crate::v2::view::BView;
use crate::v2::view::prim::{
    BAction, BColAlign, BPanelRole, BRowShade, BSortDirection, BTableCell,
};
use gpui::AnyElement;

/// Flex-row bounded primitive.
pub fn row(children: &[BView], c: &BRenderCtx) -> AnyElement {
    row::BRow(children).render(c)
}

/// File-browser style list row with shade.
pub fn list_row(children: &[BView], shade_role: BRowShade, c: &BRenderCtx) -> AnyElement {
    list::BList {
        children,
        shade: shade_role,
    }
    .render(c)
}

/// Table body row with shared column geometry.
pub fn table_row(
    cells: &[BTableCell],
    shade_role: BRowShade,
    action: Option<&BAction>,
    c: &BRenderCtx,
) -> AnyElement {
    table::BTableRow {
        cells,
        shade: shade_role,
        action,
    }
    .render(c)
}

/// Table header row with shared column geometry.
pub fn table_header(cells: &[BTableCell], c: &BRenderCtx) -> AnyElement {
    table::BTableHeader(cells).render(c)
}

/// Sort header for a table column.
pub fn sort_header(
    label: &str,
    direction: Option<BSortDirection>,
    align: BColAlign,
    action: &BAction,
    c: &BRenderCtx,
) -> AnyElement {
    sort::BSort {
        label,
        direction,
        align,
        action,
    }
    .render(c)
}

/// Accordion section header.
pub fn accordion_header(
    title: &str,
    expanded: bool,
    action: &BAction,
    c: &BRenderCtx,
) -> AnyElement {
    accordion::BAccordion {
        title,
        expanded,
        action,
    }
    .render(c)
}

/// Default-compatible fixed-width indent entry point.
pub fn indent(level: u16) -> AnyElement {
    tree::BTreeIndent(level).render_default()
}

/// Render-pass-aware indent used by the BView dispatcher.
pub(super) fn indent_resolved(level: u16, c: &BRenderCtx) -> AnyElement {
    tree::BTreeIndent(level).render(c)
}

/// Outliner tree row with guides, disclosure, icon, name, and trail.
#[allow(clippy::too_many_arguments)]
pub fn tree_row(
    depth: u16,
    expanded: Option<bool>,
    toggle: Option<&BAction>,
    icon: &str,
    children: &[BView],
    trail: &[BView],
    shade_role: BRowShade,
    c: &BRenderCtx,
) -> AnyElement {
    tree::BTreeRow {
        depth,
        expanded,
        toggle,
        icon,
        children,
        trail,
        shade: shade_role,
    }
    .render(c)
}

/// Flex-col filling its slot.
pub fn column(children: &[BView], c: &BRenderCtx) -> AnyElement {
    column::BColumn(children).render(c)
}

/// Panel roled by `BPanelRole` — delegates to the focused panel sub-module.
pub fn panel(role: BPanelRole, children: &[BView], c: &BRenderCtx) -> AnyElement {
    panel::view(role, children, c)
}
