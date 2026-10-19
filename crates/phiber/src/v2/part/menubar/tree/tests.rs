//! Tests for `mod` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;
use crate::v2::host::BHostType;
use crate::v2::part::menubar::editor::image::set::VIEW as IMAGE_VIEW;
use crate::v2::part::sidebar::show::BSidebarShown;
use crate::v2::state::action::apply_action;

fn sidebar_item() -> &'static BMenuItem {
    IMAGE_VIEW.iter().find(|it| it.label == "Sidebar").expect("View ▸ Sidebar in the shared image table")
}

#[test]
fn a_shared_table_s_sidebar_flips_the_showing_editor_s_own_region() {
    // One `image::VIEW` table serves two editors: picked in each, the
    // same row flips that editor's key — never the other's.
    let it = sidebar_item();
    for host in [BHostType::Image, BHostType::Uv] {
        let mut s = BState::new();
        apply_action(&mut s, &leaf_action(host.slug(), &["View"], it));
        assert_eq!(s.get(&BSidebarShown(host).key()), Some(V_TRUE), "{host:?}'s sidebar opened");
        let other = if host == BHostType::Image { BHostType::Uv } else { BHostType::Image };
        assert_eq!(s.get(&BSidebarShown(other).key()), None, "{other:?}'s was left alone");
        // Picked again, it puts the region away.
        apply_action(&mut s, &leaf_action(host.slug(), &["View"], it));
        assert_eq!(s.get(&BSidebarShown(host).key()), Some(V_FALSE));
    }
}

#[test]
fn a_shared_table_s_sidebar_is_checked_against_the_showing_editor() {
    let it = sidebar_item();
    let mut s = BState::new();
    s.set(&BSidebarShown(BHostType::Uv).key(), V_TRUE);
    let checked = |host: BHostType| match item_view(host.slug(), &["View"], it, &s) {
        BView::MenuItem(ctl) => ctl.icon.as_deref() == Some("check"),
        other => panic!("a menu row, got {other:?}"),
    };
    assert!(checked(BHostType::Uv), "the UV editor's row shows its open sidebar");
    assert!(!checked(BHostType::Image), "the Image editor's row shows its closed one");
}

#[test]
fn a_host_the_slug_does_not_name_reports_rather_than_flipping() {
    // The command palette walks every table as the top bar.
    let it = sidebar_item();
    let action = leaf_action("no-such-editor", &["View"], it);
    assert_eq!(action, BAction::Set(OpKey::Report.path(), "View ▸ Sidebar".to_string()));
}

fn frame_all_item() -> &'static BMenuItem {
    use crate::v2::part::menubar::editor::view3d::EDITOR_VIEW;
    EDITOR_VIEW.iter().find(|it| it.label == "Frame All").expect("Frame All in the shared node-editor View table")
}

#[test]
fn frame_all_frames_the_showing_node_editor_s_own_graph() {
    // One `EDITOR_VIEW` table serves the node editors: Home names the
    // editor it was picked in, so `BNodeViewRule` frames that graph.
    let it = frame_all_item();
    for host in [BHostType::Compositor, BHostType::Shader, BHostType::GeoNode] {
        let action = leaf_action(host.slug(), &["View"], it);
        assert_eq!(action, BAction::Set(OpKey::NodeView.path(), format!("{}:all", host.slug())), "{host:?}");
    }
}

#[test]
fn frame_all_in_a_stranger_s_table_reports_rather_than_firing() {
    // The palette walks the table with no editor behind it.
    let it = frame_all_item();
    let action = leaf_action("no-such-editor", &["View"], it);
    assert_eq!(action, BAction::Set(OpKey::Report.path(), "View ▸ Frame All".to_string()));
}
