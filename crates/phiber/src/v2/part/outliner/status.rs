//! The Outliner's **Status** display mode: the event tree (`state::event`)
//! as rows — each event a row that opens onto its key/value pairs and its
//! children, newest first. Clicking the status bar opens this mode on the
//! event the bar was showing (`part::statuslog`).

use super::tree::{BOutlineRow, BRowKind, intern_name, is_expanded_or};
use crate::v2::state::BState;
use crate::v2::state::event::{BEvent, BEvents};

/// The row id an event's expansion is kept under.
pub fn event_row_id(id: u64) -> &'static str {
    intern_name(&format!("event/{id}"))
}

/// Every row, roots newest first, each followed by its pairs and children
/// when open.
pub fn rows(s: &BState) -> Vec<BOutlineRow> {
    let all = BEvents::all();
    let mut out = Vec::new();
    // The newest event opens by itself — it is the one the bar is showing;
    // older ones stay shut until opened.
    for (k, root) in all.iter().rev().filter(|e| e.parent.is_none()).enumerate() {
        push(s, &all, root, 0, k == 0, &mut out);
    }
    out
}

fn push(s: &BState, all: &[BEvent], e: &BEvent, depth: u16, newest: bool, out: &mut Vec<BOutlineRow>) {
    let id = event_row_id(e.id);
    let children: Vec<&BEvent> = all.iter().filter(|c| c.parent == Some(e.id)).collect();
    let has_more = !e.pairs.is_empty() || !children.is_empty();
    let open = has_more && is_expanded_or(s, id, newest);
    out.push(BOutlineRow {
        id,
        kind: BRowKind::Event,
        depth,
        icon: "info",
        label: e.title.clone(),
        expanded: has_more.then_some(open),
        object: None,
        matched: false,
        summary: Vec::new(),
    });
    if !open {
        return;
    }
    for (k, v) in &e.pairs {
        out.push(BOutlineRow {
            id: intern_name(&format!("event/{}/{k}", e.id)),
            kind: BRowKind::Field,
            depth: depth + 1,
            icon: "circle",
            label: format!("{k}: {v}"),
            expanded: None,
            object: None,
            matched: false,
            summary: Vec::new(),
        });
    }
    // Only the newest event itself opens by default; its parts stay shut
    // until asked for, so the pairs that answer "how did it go" are what
    // shows.
    for c in children {
        push(s, all, c, depth + 1, false, out);
    }
}
