use super::*;
use crate::v2::state::action::apply_action as apply;
use crate::v2::view::prim::BAction;

/// Image ▸ Save As from the render view: the browser opens over it and
/// the render view is back when the browser goes, whichever way it goes.
#[test]
fn the_browser_opens_over_the_render_view_and_gives_it_back() {
    let mut s = BState::new();
    s.set(ModalKey::Open, V_MODAL_RENDER);
    apply(&mut s, &BAction::Set(FileKey::Purpose.path(), "save_image".to_string()));
    assert_eq!(s.get(ModalKey::Open), Some(V_MODAL_FILE_BROWSER), "the browser is up");
    // Cancelled: the action `modal = ""`.
    apply(&mut s, &BAction::Set(ModalKey::Open.path(), String::new()));
    assert_eq!(s.get(ModalKey::Open), Some(V_MODAL_RENDER), "the render view is back");
    assert!(s.get(ModalKey::FileReturn).is_none());
    // Opened from a menu with nothing up: plain.
    s.set(ModalKey::Open, V_EMPTY);
    apply(&mut s, &BAction::Set(FileKey::Purpose.path(), "open".to_string()));
    assert_eq!(s.get(ModalKey::Open), Some(V_MODAL_FILE_BROWSER));
    apply(&mut s, &BAction::Set(ModalKey::Open.path(), String::new()));
    assert_eq!(s.get(ModalKey::Open), Some(""));
}
