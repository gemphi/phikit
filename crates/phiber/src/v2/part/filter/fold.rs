//! Header popovers fold their headings, as Blender's do.
//!
//! Blender's long header popovers are a parent panel whose headings are
//! sub-panels, each with its own collapse triangle: `VIEW3D_PT_overlay` holds
//! `VIEW3D_PT_overlay_guides` / `_text` / `_object` / `_geometry` /
//! `_motion_tracking`, `IMAGE_PT_overlay` holds `_guides` / `_uv_stretch` /
//! `_image`, and `CLIP_PT_display` holds `CLIP_PT_marker_display` /
//! `CLIP_PT_clip_display` (`bl_ui/space_view3d.py`, `space_image.py`,
//! `space_clip.py`). The short ones are a single panel of `col.label`
//! headings with no sub-panels and no triangles: `VIEW3D_PT_gizmo_display`,
//! `NODE_PT_overlay`, `SPREADSHEET_PT_filter`.
//!
//! So the rule is Blender's own, said once instead of per popover: **more
//! than one heading means sub-panels.** A table with one heading is the
//! panel's own name and stays a flat list; a table with two or more draws
//! each heading as a folding card. That lands on Blender's answer for every
//! family with no special case, and a table that grows a second heading folds
//! without anyone remembering to make it.
//!
//! The cards fold on a key scoped to the family the boxes under them are
//! scoped to (`overlay::key`), not to the region drawing them — so the Movie
//! Clip editor's Marker Display is folded in the header exactly when it is
//! folded in the sidebar's View tab. A key per drawing site is how the two
//! would come to disagree, which is the same mistake a second copy of the
//! rows would be.

use crate::v2::part::props::slug;
use crate::v2::state::val::{V_FALSE, V_TRUE};
use crate::v2::state::{BPath, BState};
use crate::v2::view::BView;
use crate::v2::view::prim::{BCard, with_peers};

/// One heading and the rows under it, as a table splits them.
pub type BGroup = (&'static str, Vec<BView>);

/// The most rows one group shows. A group with more is split into
/// sub-groups, each a folding card inside it (`sub_cards`), so no list in a
/// popover runs past eight before it is grouped.
pub const MOST: usize = 8;

/// `ctx.<scope…>.<group>.expanded` — the key one heading folds on.
pub fn expand_key(scope: &[&'static str], group: &str) -> BPath {
    let mut segs: Vec<&'static str> = scope.to_vec();
    segs.push(slug(group));
    segs.push("expanded");
    BPath::owned(segs)
}

/// Whether a heading is open: what the user last chose, or — until they
/// choose — Blender's default for that sub-panel (`DEFAULT_CLOSED` or not).
pub fn is_open(s: &BState, scope: &[&'static str], group: &str, closed: bool) -> bool {
    match s.get(&expand_key(scope, group)) {
        Some(V_FALSE) => false,
        Some(V_TRUE) => true,
        _ => !closed,
    }
}

/// True when a table's headings are sub-panels in Blender — when there is
/// more than one. One heading is the panel's own name, not a sub-panel.
pub fn folds(groups: &[BGroup]) -> bool {
    groups.len() > 1
}

/// The headings as folding cards, each over its own key and told its peers
/// so the column draws them as one strip (`with_peers`).
pub fn cards(s: &BState, scope: &[&'static str], groups: Vec<BGroup>, closed: fn(&str) -> bool) -> Vec<BView> {
    with_peers(
        groups
            .into_iter()
            .enumerate()
            .map(|(n, (title, rows))| {
                // A popover opens short: its first heading open, the rest
                // shut until asked for, so a long filter set is a column of
                // titles to choose from rather than a wall of boxes. What
                // the user opens or shuts is remembered over that.
                let open = is_open(s, scope, title, closed(title) || n > 0);
                let body = match open {
                    true => grouped(s, scope, title, rows),
                    false => Vec::new(),
                };
                BCard::panel(title).fold(expand_key(scope, title), open).accordion().body(body).into()
            })
            .collect(),
    )
}

/// `groups` as folding sub-panels of the card `parent` heads — an
/// accordion, the first open, each on a key under its parent's.
pub fn sub_cards(s: &BState, scope: &[&'static str], parent: &str, groups: Vec<(String, Vec<BView>)>) -> Vec<BView> {
    let mut scope = scope.to_vec();
    scope.push(slug(parent));
    let cards = groups
        .into_iter()
        .enumerate()
        .map(|(n, (title, rows))| {
            let open = is_open(s, &scope, &title, n > 0);
            let body = if open { rows } else { Vec::new() };
            BCard::panel(title.clone()).fold(expand_key(&scope, &title), open).accordion().body(body).into()
        })
        .collect();
    with_peers(cards)
        .into_iter()
        .map(|view| match view {
            BView::Card(card) => BView::from(card.sub()),
            view => view,
        })
        .collect()
}

/// A group's rows as they are drawn: as they come when there are at most
/// `MOST` (or they are already sub-groups), else split into runs of at most
/// `MOST`, each named by its first and last row.
fn grouped(s: &BState, scope: &[&'static str], title: &str, rows: Vec<BView>) -> Vec<BView> {
    if rows.len() <= MOST || rows.iter().any(|row| matches!(row, BView::Card(_))) {
        return rows;
    }
    let runs = rows.len().div_ceil(MOST);
    let size = rows.len().div_ceil(runs);
    let groups = rows
        .chunks(size)
        .map(|run| {
            let name = |row: Option<&BView>| row.and_then(label_of).unwrap_or_default();
            (format!("{} \u{2013} {}", name(run.first()), name(run.last())), run.to_vec())
        })
        .collect();
    sub_cards(s, scope, title, groups)
}

/// The first label a row carries: its control's, else its own text.
fn label_of(row: &BView) -> Option<String> {
    match row {
        BView::Ctrl(ctrl) => ctrl.label.clone(),
        BView::Button(ctl) => ctl.label.clone(),
        BView::Label { text } => Some(text.clone()),
        BView::Split { list } | BView::Row { list } => list.iter().rev().find_map(label_of),
        _ => None,
    }
}

/// What a popover draws: its headings as folding cards when Blender makes
/// them sub-panels, else the flat rows the caller builds for a single panel.
pub fn body(s: &BState, scope: &[&'static str], groups: Vec<BGroup>, closed: fn(&str) -> bool, flat: impl FnOnce() -> Vec<BView>) -> Vec<BView> {
    match folds(&groups) {
        true => cards(s, scope, groups, closed),
        false => flat(),
    }
}

/// No heading closed on its own account (Blender's `DEFAULT_CLOSED` is only
/// on `VIEW3D_PT_overlay_motion_tracking`, which phiber has no rows for);
/// `cards` still opens only the first, so the popover opens short.
pub fn all_open(_: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCOPE: [&str; 2] = ["overlay", "test"];

    fn rows(n: usize) -> Vec<BView> {
        (0..n).map(|_| BView::label("row")).collect()
    }

    #[test]
    fn one_heading_stays_flat_and_two_fold_on_their_own_keys() {
        let s = BState::new();
        // `NODE_PT_overlay`: one panel, no sub-panels — the flat rows stand.
        let one = body(&s, &SCOPE, vec![("Overlays", rows(6))], all_open, || rows(6));
        assert_eq!(one.len(), 6, "one heading draws its rows, not a card");
        assert!(!one.iter().any(|v| matches!(v, BView::Card(_))));
        // `VIEW3D_PT_overlay`: two headings, so two sub-panels.
        let two = body(&s, &SCOPE, vec![("Guides", rows(8)), ("Objects", rows(6))], all_open, || rows(14));
        assert_eq!(two.len(), 2, "one card per heading, not fourteen rows");
        assert!(two.iter().all(|v| matches!(v, BView::Card(_))));
        assert_eq!(expand_key(&SCOPE, "Guides").encode(), "ctx.overlay.test.guides.expanded");
        assert_ne!(expand_key(&SCOPE, "Guides"), expand_key(&SCOPE, "Objects"), "a key each, or one triangle shuts both");
    }

    #[test]
    fn a_shut_heading_drops_its_rows_and_the_user_s_choice_beats_the_default() {
        let mut s = BState::new();
        let built = |s: &BState, closed: fn(&str) -> bool| cards(s, &SCOPE, vec![("Guides", rows(8)), ("Objects", rows(6))], closed);
        let open = built(&s, all_open);
        let BView::Card(first) = &open[0] else { panic!("a card") };
        assert_eq!(first.body.len(), 8, "open: its rows are drawn");
        // `DEFAULT_CLOSED` leaves the header and drops the body.
        let shut = built(&s, |g| g == "Guides");
        let BView::Card(first) = &shut[0] else { panic!("a card") };
        assert!(first.body.is_empty(), "closed: the header alone");
        // The headings after the first open shut, so the popover opens short.
        let BView::Card(second) = &open[1] else { panic!("a card") };
        assert!(second.body.is_empty(), "only the first heading opens by default");
        s.set(&expand_key(&SCOPE, "Objects"), V_TRUE);
        let asked = built(&s, all_open);
        let BView::Card(second) = &asked[1] else { panic!("a card") };
        assert_eq!(second.body.len(), 6, "opened, it shows its rows");
        // Opening it is remembered over the default.
        s.set(&expand_key(&SCOPE, "Guides"), V_TRUE);
        let reopened = built(&s, |g| g == "Guides");
        let BView::Card(first) = &reopened[0] else { panic!("a card") };
        assert_eq!(first.body.len(), 8, "the user's choice stands");
    }

    #[test]
    fn each_card_knows_its_peers_so_the_column_draws_one_strip() {
        let s = BState::new();
        let built = cards(&s, &SCOPE, vec![("A", rows(1)), ("B", rows(1)), ("C", rows(1))], all_open);
        for view in &built {
            let BView::Card(card) = view else { panic!("a card") };
            let fold = card.fold.as_ref().expect("folding");
            assert_eq!(fold.peers.len(), 2, "its two neighbours, not itself");
            assert!(!fold.peers.contains(&fold.key));
        }
    }
}
