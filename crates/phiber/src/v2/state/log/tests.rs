use super::*;
use crate::v2::state::{BKey as _, BPath, CtxKey, MenuKey};

fn set(before: &BState, path: &[&'static str], value: &str) -> (BAction, BState) {
    let action = BAction::Set(BPath::owned(path.to_vec()), value.to_string());
    let mut after = before.clone();
    super::super::action::apply_action(&mut after, &action);
    (action, after)
}

#[test]
fn a_zoom_reads_as_a_sentence_with_the_new_value() {
    let before = BState::new();
    let (a, after) = set(&before, &["view3d", "zoom"], "1.7");
    assert_eq!(describe(&a, &before, &after).as_deref(), Some("Zoomed to 1.70×"));
}

#[test]
fn a_menu_opening_is_not_an_action_the_user_took() {
    let before = BState::new();
    let path = MenuKey::Top("topbar", "File").path();
    let action = BAction::Set(path, "true".to_string());
    let mut after = before.clone();
    super::super::action::apply_action(&mut after, &action);
    let a = action;
    assert_eq!(describe(&a, &before, &after), None);
}

#[test]
fn an_action_that_changes_nothing_says_nothing() {
    let mut before = BState::new();
    before.set(CtxKey::Space, "shading");
    let (a, after) = set(&before, &["space"], "shading");
    assert_eq!(describe(&a, &before, &after), None);
}

#[test]
fn the_log_keeps_the_newest_first_and_bounds_itself() {
    clear();
    for i in 0..(CAPACITY + 10) {
        push(format!("line {i}"));
    }
    let all = entries();
    assert_eq!(all.len(), CAPACITY);
    assert_eq!(all[0].text, format!("line {}", CAPACITY + 9));
    assert_eq!(latest().map(|e| e.seq), Some((CAPACITY + 10) as u64));
    clear();
}
