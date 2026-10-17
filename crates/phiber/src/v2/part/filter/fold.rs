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
            .map(|(title, rows)| {
                let open = is_open(s, scope, title, closed(title));
                let body = match open {
                    true => rows,
                    false => Vec::new(),
                };
                BCard::panel(title).fold(expand_key(scope, title), open).body(body).into()
            })
            .collect(),
    )
}

/// What a popover draws: its headings as folding cards when Blender makes
/// them sub-panels, else the flat rows the caller builds for a single panel.
pub fn body(s: &BState, scope: &[&'static str], groups: Vec<BGroup>, closed: fn(&str) -> bool, flat: impl FnOnce() -> Vec<BView>) -> Vec<BView> {
    match folds(&groups) {
        true => cards(s, scope, groups, closed),
        false => flat(),
    }
}

/// Every heading open, which is what Blender ships for these sub-panels —
/// only `VIEW3D_PT_overlay_motion_tracking` carries `DEFAULT_CLOSED`, and
/// phiber has no motion-tracking overlays to put under it yet.
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
        let BView::Card(second) = &shut[1] else { panic!("a card") };
        assert_eq!(second.body.len(), 6, "and only that one");
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
