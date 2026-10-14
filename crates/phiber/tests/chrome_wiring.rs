//! Window chrome + activity bar + modal wiring.

use phiber::v2::state::{BKey as _, ActivityKey, CtxKey, ModalKey};
use phiber::v2::host::BHostType;
use phiber::v2::modal::{BModalItem, BModalType};
use phiber::v2::part::caption::{BCaption, BCaptionSide, CAPTION};
use phiber::v2::state::{BState, action::apply_action};
use phiber::v2::view::prim::BAction;
use phiber::v2::view::{BView, BViewItem};

/// Every primary opens its panel beside the rail, as Slack's do; the panel's
/// rows are what raise a dialog — a channel opens the collaboration window,
/// New File the splash.
#[test]
fn every_primary_opens_its_panel_and_its_rows_raise_the_dialogs() {
    use phiber::v2::part::activity::option;
    for label in ["Home", "Chat", "Stream", "Profile"] {
        let mut s = BState::default();
        apply_action(&mut s, &BAction::Set(ActivityKey::Tab.path(), label.into()));
        assert_eq!(s.get(ActivityKey::Group), Some(label), "{label} opens its panel");
        assert_eq!(s.get(ModalKey::Open), None, "{label} raises no dialog by itself");
    }
    let mut s = BState::default();
    apply_action(&mut s, &BAction::Set(ActivityKey::Pick.path(), "Home:new".into()));
    assert_eq!(s.get(ModalKey::Open), Some("splash"));
    s.set(phiber::v2::state::AppKey::CollabChannel("0"), "renders\tann");
    apply_action(&mut s, &BAction::Set(ActivityKey::Tab.path(), "Chat".into()));
    let chat = option::open_group(&s).unwrap();
    assert_eq!(chat.options_for(&s)[0].label.as_deref(), Some("renders"));
    apply_action(&mut s, &BAction::Set(ActivityKey::Pick.path(), "Chat:renders".into()));
    assert_eq!(s.get(ModalKey::Open), Some("collab"));
}

#[test]
fn a_grouping_entry_opens_its_secondary_strip_instead_of_a_dialog() {
    use phiber::v2::part::activity::option;
    
    for label in ["Spaces", "Settings"] {
        let mut s = BState::default();
        apply_action(&mut s, &BAction::Set(ActivityKey::Tab.path(), label.into()));
        assert_eq!(s.get(ActivityKey::Group), Some(label), "{label} opens its strip");
        assert_ne!(s.get(ModalKey::Open), Some("spaces"), "{label} raises no dialog");
        let entry = option::open_group(&s).expect("a group is open");
        assert!(
            !entry.options_for(&s).is_empty(),
            "{label} must carry options"
        );
        // Clicking the same icon again puts it away.
        apply_action(&mut s, &BAction::Set(ActivityKey::Tab.path(), label.into()));
        assert_eq!(s.get(ActivityKey::Group), None, "{label} closes on a second click");
    }
}

#[test]
fn an_option_from_the_strip_does_what_it_names() {
    use phiber::v2::part::activity::option;
    
    let mut s = BState::default();
    apply_action(&mut s, &BAction::Set(ActivityKey::Pick.path(), "Spaces:Shading".into()));
    assert_eq!(s.get(CtxKey::Space), Some(phiber::v2::space::BSpaceType::Shading.slug()));
    assert_eq!(s.get(ActivityKey::Pick), None, "the pick does not linger");

    let mut s = BState::default();
    apply_action(&mut s, &BAction::Set(ActivityKey::Pick.path(), "Settings:Appearance".into()));
    assert_eq!(s.get(&option::prefs_tab_key()), Some("Appearance"));
    assert_eq!(s.get(ModalKey::Open), Some("settings"), "and raises the screen showing it");
}

#[test]
fn picking_a_space_closes_the_spaces_dialog() {
    let mut s = BState::default();
    apply_action(&mut s, &BAction::Set(ModalKey::Open.path(), "spaces".into()));
    apply_action(&mut s, &BAction::Set(CtxKey::Space.path(), "Shading".into()));
    assert_eq!(s.get(CtxKey::Space), Some("Shading"));
    assert_eq!(s.get(ModalKey::Open), Some(""));
}

#[test]
fn settings_modal_is_the_preferences_host() {
    // A dialog that raises an editor declares it as an area (`spec.areas`)
    // and is built by the zone builder, exactly as a space is: Zone ▸ Area ▸
    // Host. The Preferences dialog is the Prefs editor in a Center area.
    let v = BModalItem::new(BModalType::Settings).build(&BState::default());
    let BView::Modal { body, .. } = v else { panic!("modal") };
    let BView::Zone { kind: phiber::v2::zone::BZoneType::Center, list } = *body else {
        panic!("the dialog's body is its Center zone, got {body:?}")
    };
    let [BView::Area { child, .. }] = list.as_slice() else { panic!("one area") };
    assert!(matches!(**child, BView::Host { kind: BHostType::Prefs, .. }));
    // Every editor dialog is declared the same way.
    for kind in [BModalType::FileBrowser, BModalType::Render] {
        assert!(!kind.spec().areas.is_empty(), "{kind:?} raises an editor through its areas");
    }
}

#[test]
fn command_palette_searches_every_menu_leaf() {
    let hits = phiber::v2::modal::command::palette::hits("open");
    assert!(hits.iter().any(|h| h.path == "File ▸ Open…" || h.path.starts_with("File ▸ Open")), "{:?}", hits.iter().map(|h| &h.path).collect::<Vec<_>>());
    assert!(phiber::v2::modal::command::palette::hits("").len() > 30);
    // Render ▸ Render Turnaround is a leaf the palette finds, firing the op.
    let turnaround = phiber::v2::modal::command::palette::hits("turnaround");
    let hit = turnaround.iter().find(|h| h.path == "Render ▸ Render Turnaround").expect("the turnaround command is in the Render menu");
    assert_eq!(hit.action, BAction::Set(phiber::v2::state::OpKey::Render.path(), "turnaround".to_string()));
}

#[test]
fn caption_controls_only_in_the_topbar_on_the_platform_side() {
    let s = BState::default();
    let lead = BCaption { host: BHostType::Topbar, side: BCaptionSide::Lead }.build(&s);
    let trail = BCaption { host: BHostType::Topbar, side: BCaptionSide::Trail }.build(&s);
    let shown = if CAPTION == BCaptionSide::Lead { (&lead, &trail) } else { (&trail, &lead) };
    assert!(matches!(shown.0, BView::WindowControls { .. }));
    assert!(shown.1.is_empty());
    assert!(BCaption { host: BHostType::View3D, side: CAPTION }.build(&s).is_empty());
}
