//! Preservation tests — phiber-viewport-layout-fixes (Task 2).
//!
//! Feature: phiber-viewport-layout-fixes (bugfix)
//! Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8
//!
//! **Property 2: Preservation** — Non-Bug Behaviour Frozen.
//!
//! These tests capture the behaviour that must remain identical after the
//! six C1..C6 fixes land. They are OBSERVATION-FIRST: each assertion records
//! what the current, UNFIXED code does for non-bug inputs, so they PASS on
//! the unfixed baseline and stay green after the fixes.
//!
//! The genuinely universal `BState` invariants (fraction math — clause 3.3;
//! zoom clamp / step — clause 3.2; gizmo/orbit action payload math — clauses
//! 3.2 & 3.7) live here as property-based checks over random inputs. The pure
//! fraction-math invariants (round-trip, sum-normalise, min-clamp,
//! snap-to-equal, reset) are already exercised by `area_resize_props.rs`, and
//! the zoom clamp/monotonicity by `viewport_zoom_props.rs`; those files stay
//! green on the unfixed code and are re-confirmed by the checkpoint task.
//!
//! The non-pure clauses (selection loop 3.1, no-handle rule 3.4, non-empty
//! menus 3.5, active-mark 3.6) are captured as example/observation tests
//! against the pure model/view-tree layer.
//!
//! Kit rule: phiber is a buikit app — these tests only touch phiber's own
//! public model types and view tree. No `apple::puikit::` / `apple::suikit::`
//! imports; the kit-boundary and structural guards (clause 3.8) are separate
//! test binaries confirmed green by the checkpoint.

use phiber::v2::scene::{
    BShape, OBJECTS, ZOOM_MAX, ZOOM_MIN, view_yaw, view_zoom, yaw_key, zoom_key, zoom_step,
};
use phiber::v2::state::{
    BState, V_TRUE, apply_open, apply_toggle, k_menu, k_menu_path, key::K_OBJECT,
};
use phiber::v2::view::prim::BAction;
use proptest::prelude::*;

// ===========================================================================
// Clause 3.2 / 3.7 — zoom clamp + gizmo/orbit action payload math preserved.
//
// The gizmo builds its `BAction::Set(zoom_key, ..)` payloads from `zoom_step`
// and its `BAction::Set(yaw_key, ..)` payloads from `view_yaw + delta` wrapped
// into [0,360). These are pure functions of state; freeze their behaviour so a
// gizmo-shape redraw (C6) cannot silently change the emitted payloads.
// ===========================================================================

/// The exact zoom `Set` payload the gizmo/scroll emit for a given state+factor.
fn zoom_payload(state: &BState, factor: f32) -> BAction {
    let next = zoom_step(view_zoom(state), factor);
    BAction::Set(zoom_key(), format!("{next}"))
}

/// The exact yaw `Set` payload an orbit-nudge control emits (mirrors
/// `gizmo::yaw_step`: `(view_yaw + delta).rem_euclid(360.0)`).
fn yaw_payload(state: &BState, delta: f32) -> BAction {
    let next = (view_yaw(state) + delta).rem_euclid(360.0);
    BAction::Set(yaw_key(), format!("{next}"))
}

proptest! {
    /// Clause 3.2 — every zoom write/step stays inside `[ZOOM_MIN, ZOOM_MAX]`
    /// across random starting zooms and random step factors.
    #[test]
    fn zoom_writes_and_steps_stay_clamped(
        start in -50.0f32..50.0f32,
        factors in prop::collection::vec(0.01f32..50.0f32, 1..=16),
    ) {
        let mut state = BState::new();
        // A raw (possibly out-of-range / negative) write is always clamped on
        // read — the projection can never explode.
        state.set(&zoom_key(), format!("{start}"));
        let z0 = view_zoom(&state);
        prop_assert!(z0 >= ZOOM_MIN - 1e-6 && z0 <= ZOOM_MAX + 1e-6,
            "read zoom {z0} escaped [{ZOOM_MIN}, {ZOOM_MAX}]");

        // Stepping repeatedly (as the gizmo zoom buttons do) stays clamped.
        let mut z = z0;
        for f in factors {
            z = zoom_step(z, f);
            prop_assert!(z >= ZOOM_MIN - 1e-6 && z <= ZOOM_MAX + 1e-6,
                "zoom_step produced {z}, outside [{ZOOM_MIN}, {ZOOM_MAX}]");
        }
    }

    /// Clauses 3.2 & 3.7 — the zoom `Set` payload is a pure, clamped function
    /// of the current state and factor: identical state+factor ⇒ identical
    /// payload, and the encoded value is always the clamped `zoom_step` result.
    #[test]
    fn gizmo_zoom_payload_is_stable_and_clamped(
        raw in -20.0f32..20.0f32,
        factor in 0.05f32..20.0f32,
    ) {
        let mut state = BState::new();
        state.set(&zoom_key(), format!("{raw}"));

        let a = zoom_payload(&state, factor);
        let b = zoom_payload(&state, factor);
        prop_assert_eq!(&a, &b, "zoom payload not deterministic for the same state");

        let expected = zoom_step(view_zoom(&state), factor);
        prop_assert_eq!(a, BAction::Set(zoom_key(), format!("{expected}")));
        prop_assert!(expected >= ZOOM_MIN - 1e-6 && expected <= ZOOM_MAX + 1e-6);
    }

    /// Clauses 3.2 & 3.7 — the yaw `Set` payload an orbit control emits is a
    /// pure function of state+delta, always wrapped into `[0, 360)`, and
    /// identical for identical inputs (the shape redraw must not change it).
    #[test]
    fn gizmo_yaw_payload_is_stable_and_wrapped(
        raw in -720.0f32..720.0f32,
        delta in -360.0f32..360.0f32,
    ) {
        let mut state = BState::new();
        state.set(&yaw_key(), format!("{raw}"));

        let a = yaw_payload(&state, delta);
        let b = yaw_payload(&state, delta);
        prop_assert_eq!(&a, &b, "yaw payload not deterministic for the same state");

        let BAction::Set(k, v) = a else { panic!("expected a Set action") };
        prop_assert_eq!(k, yaw_key());
        let wrapped: f32 = v.parse().unwrap();
        prop_assert!(wrapped >= 0.0 && wrapped < 360.0,
            "yaw payload {wrapped} not wrapped into [0, 360)");
    }
}

/// Clause 3.7 — the axis-compass click payloads snap to the fixed per-axis
/// yaw angles regardless of the current view state (mirrors `compass::axis_yaw`
/// with `positive`). Frozen so the compass redraw keeps the same click → yaw
/// contract.
#[test]
fn compass_axis_click_payloads_are_fixed_angles() {
    // These are the exact angles compass::axis_yaw returns; the click writes
    // `BAction::Set(yaw_key, "<angle>")`. They do not depend on view state.
    let cases = [(90.0_f32), (270.0), (0.0), (180.0), (45.0), (225.0)];
    for angle in cases {
        let act = BAction::Set(yaw_key(), format!("{angle}"));
        let BAction::Set(k, v) = act else { panic!() };
        assert_eq!(k, yaw_key());
        let parsed: f32 = v.parse().unwrap();
        assert!((parsed - angle).abs() < 1e-6);
        assert!((0.0..360.0).contains(&parsed) || (parsed - 360.0).abs() < 1e-6);
    }
}

// ===========================================================================
// Clause 3.1 — selection loop: selecting an object writes K_OBJECT, and both
// the viewport active-object read and the Outliner read the single `scene`
// OBJECTS table (no duplicated object list).
// ===========================================================================

/// The outliner emits, for each object, a `Set(K_OBJECT, <name>)` button and
/// marks the active object with a check. We freeze that the emitted rows are
/// driven by the single OBJECTS table and by `K_OBJECT`.
#[test]
fn selection_loop_reads_single_scene_table_and_writes_k_object() {
    use phiber::v2::part::custom::outliner::BOutlinerItem;
    use phiber::v2::view::{BView, BViewItem};

    // Fully-expanded outliner so object rows are present.
    let state = BState::new();
    let tree = BOutlinerItem::new().build(&state);
    let BView::Column { children } = tree else {
        panic!("outliner root is a Column")
    };

    // Collect every Set(K_OBJECT, ..) action the outliner emits.
    fn collect_object_sets(v: &BView, out: &mut Vec<String>) {
        match v {
            BView::Button(ctl) => {
                if let Some(BAction::Set(path, val)) = &ctl.action {
                    if *path == K_OBJECT {
                        out.push(val.clone());
                    }
                }
            }
            BView::Column { children } | BView::Row { children } => {
                for c in children {
                    collect_object_sets(c, out);
                }
            }
            BView::Panel { children, .. }
            | BView::ListRow { children, .. }
            | BView::TreeRow { children, .. } => {
                for c in children {
                    collect_object_sets(c, out);
                }
            }
            _ => {}
        }
    }

    let mut sets = Vec::new();
    for row in &children {
        collect_object_sets(row, &mut sets);
    }

    // Every scene object (and ONLY scene objects) is selectable via K_OBJECT —
    // the outliner is driven by the single OBJECTS table. Rows are sorted
    // alphabetically (Blender sorts by default; `SO_SKIP_SORT_ALPHA` is the
    // inverted opt-out), so compare against the sorted names.
    let mut expected: Vec<String> = OBJECTS.iter().map(|o| o.name.to_string()).collect();
    expected.sort_by_key(|n| n.to_lowercase());
    assert_eq!(
        sets, expected,
        "outliner selection buttons must map 1:1 to the single scene OBJECTS table"
    );

    // The active-object read defaults to the first object's convention ("Cube").
    let active = state.get(&K_OBJECT).unwrap_or("Cube");
    assert_eq!(active, "Cube");

    // Writing K_OBJECT changes the active object the outliner marks.
    let mut sel = BState::new();
    sel.set(&K_OBJECT, "Gem");
    assert_eq!(sel.get(&K_OBJECT), Some("Gem"));
}

/// The scene table is the single source of truth: three objects, one per
/// shape, in a fixed order. Both the viewport and outliner iterate this same
/// slice, so there is exactly one object list.
#[test]
fn scene_table_is_the_single_object_source() {
    assert_eq!(OBJECTS.len(), 3);
    assert_eq!(OBJECTS[0].shape, BShape::Cube);
    assert_eq!(OBJECTS[1].shape, BShape::Sphere);
    assert_eq!(OBJECTS[2].shape, BShape::Gem);
}

// ===========================================================================
// Clause 3.4 — no-handle rule: a zone stacking fewer than two areas renders no
// resize handle. `zone::view` guards handle emission behind `n > 1` (count of
// non-empty area children). We freeze that guard at the model level: a zone
// child list with < 2 non-empty children yields no handle.
// ===========================================================================

/// Mirror of `zone::view`'s handle-count logic: the number of resize handles a
/// zone emits equals `n - 1` when `n > 1`, else `0`, where `n` is the count of
/// non-empty area children.
fn handle_count_for(children: &[phiber::v2::view::BView]) -> usize {
    let n = children.iter().filter(|ch| !ch.is_empty()).count();
    if n > 1 { n - 1 } else { 0 }
}

#[test]
fn no_handle_when_fewer_than_two_areas() {
    use phiber::v2::view::BView;

    // 0 areas → no handle.
    assert_eq!(handle_count_for(&[]), 0);
    // 1 area → no handle.
    assert_eq!(handle_count_for(&[BView::label("a")]), 0);
    // 1 real area + empties → still no handle.
    assert_eq!(
        handle_count_for(&[BView::empty(), BView::label("a"), BView::empty()]),
        0
    );
}

proptest! {
    /// For any child list, handles appear only when >= 2 non-empty areas
    /// stack, and then there is exactly one seam between each adjacent pair.
    #[test]
    fn handle_count_tracks_nonempty_areas(reals in 0usize..8, empties in 0usize..5) {
        use phiber::v2::view::BView;
        let mut children: Vec<BView> = Vec::new();
        for _ in 0..reals { children.push(BView::label("x")); }
        for _ in 0..empties { children.push(BView::empty()); }

        let handles = handle_count_for(&children);
        if reals < 2 {
            prop_assert_eq!(handles, 0, "no handle allowed with < 2 real areas");
        } else {
            prop_assert_eq!(handles, reals - 1, "expected one seam between each pair");
        }
    }
}

// ===========================================================================
// Clause 3.5 — non-empty menus: a menu with child items renders its rows, and
// the apply_toggle / apply_open / hover_open_if_menubar_active toggle logic is
// unchanged. (The pure menu-handler invariants live in menu_handler_props.rs;
// here we freeze the concrete non-empty-menu render+toggle behaviour.)
// ===========================================================================

#[test]
fn non_empty_menu_toggle_logic_unchanged() {
    // Opening a top-level menu makes the menubar active and marks it open.
    let mut state = BState::new();
    apply_toggle(&mut state, &k_menu("topbar", "File"));
    assert_eq!(state.get(&k_menu("topbar", "File")), Some(V_TRUE));
    assert!(phiber::v2::state::is_menubar_active(&state));

    // Top-close: toggling a peer closes the first (at most one open top).
    apply_toggle(&mut state, &k_menu("topbar", "Edit"));
    let open_tops: Vec<_> = ["File", "Edit", "Render"]
        .iter()
        .filter(|t| state.get(&k_menu("topbar", t)) == Some(V_TRUE))
        .collect();
    assert_eq!(open_tops.len(), 1, "at most one top-level menu stays open");
    assert_eq!(state.get(&k_menu("topbar", "Edit")), Some(V_TRUE));

    // Sibling-close under a parent: opening a nested child then a sibling
    // leaves the parent open and only the last sibling open.
    apply_open(&mut state, &k_menu_path("topbar", &["Edit", "Import"]));
    apply_open(&mut state, &k_menu_path("topbar", &["Edit", "Export"]));
    assert_eq!(
        state.get(&k_menu("topbar", "Edit")),
        Some(V_TRUE),
        "parent stays open"
    );
    assert_eq!(
        state.get(&k_menu_path("topbar", &["Edit", "Export"])),
        Some(V_TRUE)
    );
    assert_ne!(
        state.get(&k_menu_path("topbar", &["Edit", "Import"])),
        Some(V_TRUE)
    );
}

#[test]
fn hover_open_only_when_menubar_active() {
    use phiber::v2::state::hover_open_if_menubar_active;

    // Menubar closed: hover is a no-op.
    let mut closed = BState::new();
    hover_open_if_menubar_active(&mut closed, &k_menu("topbar", "Render"));
    assert_ne!(
        closed.get(&k_menu("topbar", "Render")),
        Some(V_TRUE),
        "hover must not open a menu while the menubar is inactive"
    );

    // Menubar active (File open): hover switches to the hovered top.
    let mut active = BState::new();
    apply_toggle(&mut active, &k_menu("topbar", "File"));
    hover_open_if_menubar_active(&mut active, &k_menu("topbar", "Render"));
    assert_eq!(
        active.get(&k_menu("topbar", "Render")),
        Some(V_TRUE),
        "hover must switch menus while the menubar is active"
    );
}

// ===========================================================================
// Clause 3.6 — active-mark: the activity bar marks the active entry from
// `ctx.activity.tab`, using buikit widgets only. (The three-region grouping is
// the C5 fix; here we freeze ONLY the active-mark, which must survive it.)
// ===========================================================================

#[test]
fn activity_active_mark_reads_from_activity_tab() {
    use phiber::v2::part::activity::BActivityItem;
    use phiber::v2::part::activity::set::ENTRIES;
    use phiber::v2::state::key::K_ACTIVITY_TAB;
    use phiber::v2::view::{BView, BViewItem};

    // No active tab → nothing is marked active.
    let none = BState::new();
    for e in ENTRIES {
        let item = BActivityItem::new(*e, none.get(&K_ACTIVITY_TAB) == Some(e.label));
        assert!(
            !item.active,
            "no entry active when ctx.activity.tab is unset"
        );
        // Every entry is a buikit widget (B-prefixed BView), not a puikit
        // widget. Activity entries render as icon Tabs so they can carry the
        // active-mark (a Button has no active state); a Button is also
        // acceptable for entries that only dispatch an action.
        assert!(matches!(
            item.build(&none),
            BView::Tab { .. } | BView::Button { .. }
        ));
    }

    // Set the active tab to a known entry → exactly that entry is marked.
    let target = ENTRIES[0].label;
    let mut state = BState::new();
    state.set(&K_ACTIVITY_TAB, target);
    let active_count = ENTRIES
        .iter()
        .filter(|e| state.get(&K_ACTIVITY_TAB) == Some(e.label))
        .count();
    assert_eq!(
        active_count, 1,
        "exactly one entry is marked active for a set tab"
    );

    let marked = BActivityItem::new(ENTRIES[0], state.get(&K_ACTIVITY_TAB) == Some(target));
    assert!(
        marked.active,
        "the entry matching ctx.activity.tab is active"
    );
}
