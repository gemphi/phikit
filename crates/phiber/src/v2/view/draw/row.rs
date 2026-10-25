//! Focused renderer for `BView::Row` — flex-row with bounded height.

use super::ctx::BRenderCtx;
use super::{walk, strip};
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::AnyElement;

pub struct BRow<'a>(pub &'a [BView]);

impl BRow<'_> {
    pub fn render(&self, ctx: &BRenderCtx<'_>) -> AnyElement {
        // A row is a strip along the horizontal axis. In a header its panes
        // already divide the strip three ways, so its groups hug; elsewhere
        // the groups divide the slack themselves.
        // A row inside any chrome strip — a header region, or a toolbar /
        // footer panel in a modal — holds controls, and controls sit on the
        // strip's line. Everywhere else a row holds columns.
        let in_header = ctx.strip
            || matches!(
                ctx.region,
                Some(crate::v2::region::BRegionType::Header)
                    | Some(crate::v2::region::BRegionType::ToolHeader)
                    | Some(crate::v2::region::BRegionType::Footer)
            );
        // A strip row that holds a field — the file browser's directory and
        // search, the execute bar's file name — gives the field the slack:
        // its group grows and the field with it, the way Blender's path
        // strip stretches the directory across the region.
        // The file browser's execute bar spans its strip the same way: it is
        // three groups across the dialog (`file::execute`) — what is picked,
        // what the dialog does, Cancel and the action — not a run of
        // controls hugging its start.
        let has_field = self.0.iter().any(|v| matches!(v, BView::SearchField { .. }))
            || ctx.region == Some(crate::v2::region::BRegionType::Execute);
        // A content row holds columns (panels, a column, a grid) and stretches
        // them across its height. A row of atoms — an icon beside a name in a
        // file row, a label beside a value — is a line of controls: they sit
        // on its centre line and run from its start, not stretched to its
        // height with the icon pinned mid-cell.
        let holds_columns = self.0.iter().any(|v| {
            matches!(
                v,
                BView::Card(_)
                    | BView::Column { .. }
                    | BView::Row { .. }
                    | BView::Grid { .. }
                    | BView::Canvas { .. }
            )
        });
        let fit = if in_header && has_field {
            strip::BStripFit::Divide
        } else if in_header {
            strip::BStripFit::Hug
        } else if holds_columns {
            strip::BStripFit::Fill
        } else {
            strip::BStripFit::Divide
        };
        let groups: Vec<Vec<AnyElement>> = groups(self.0)
            .into_iter()
            .map(|views| views.iter().map(|v| walk(v, ctx)).collect())
            .collect();
        let mut element = strip::place(strip::BStripAxis::Row, fit, groups, ctx);
        element = if in_header {
            // A header row passes the strip's height straight through and
            // centres its controls on the strip's line; a cap's `self_end`
            // still measures against the strip rather than against the row.
            // A row its author split into groups (`Spacer`) spans its pane,
            // so the groups land at its start, middle and end — the file
            // browser's display icons across the details column.
            let spreads = self.0.iter().any(|v| matches!(v, BView::Spacer));
            let element = element.h_full().items_center();
            if has_field || spreads { element.w_full() } else { element }
        } else {
            element.items_stretch().w_full().h_full()
        };
        element.into_any_element()
    }
}

/// A row is a set of groups separated by `BView::Spacer`: whatever the author
/// wanted kept together stays together, and the groups are pushed to the
/// strip's start, centre and end.
fn groups(list: &[BView]) -> Vec<&[BView]> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for (n, child) in list.iter().enumerate() {
        if matches!(child, BView::Spacer) {
            out.push(&list[start..n]);
            start = n + 1;
        }
    }
    out.push(&list[start..]);
    out.retain(|group| !group.is_empty());
    out
}
