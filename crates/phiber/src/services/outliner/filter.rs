//! Outliner filter application — pure, allocates result vector.
//!
//! Feature: phiber-blender-space-parity
//! Requirements: 15.2, 15.3, 15.4, 15.5

use super::types::*;

impl BOutlineTree {
    /// Apply `f` to `self` and return the flat list of rows that pass
    /// (in depth-first order). Callers turn this into the visible tree.
    ///
    /// Empty search + every type filter on returns every row (Requirement 15.3).
    pub fn apply_filter<'a>(&'a self, f: &BOutlineFilter) -> Vec<&'a BOutlineRow> {
        let mut out = Vec::new();
        walk(&self.root, f, &mut out);
        out
    }
}

fn walk<'a>(row: &'a BOutlineRow, f: &BOutlineFilter, out: &mut Vec<&'a BOutlineRow>) {
    if row_passes(row, f) {
        out.push(row);
    }
    // Descend when the parent passes the object-children filter (Req 15.5)
    // or the row is not an object.
    let descend = match row.kind {
        BOutlineRowKind::Object => f.show_object_children,
        _ => true,
    };
    if descend {
        for c in &row.children {
            walk(c, f, out);
        }
    }
}

fn row_passes(row: &BOutlineRow, f: &BOutlineFilter) -> bool {
    // Type filter — Requirements 15.4.
    match row.kind {
        BOutlineRowKind::Collection | BOutlineRowKind::SceneCollection
            if !f.show_collections => return false,
        BOutlineRowKind::Object if !f.show_objects => return false,
        _ => {}
    }

    // Search filter — Requirement 15.2.
    if f.search.is_empty() {
        return true;
    }
    let (label, needle) = if f.case_sensitive {
        (row.label.clone(), f.search.clone())
    } else {
        (row.label.to_lowercase(), f.search.to_lowercase())
    };
    if f.exact_match {
        label == needle
    } else {
        label.contains(&needle)
    }
}
