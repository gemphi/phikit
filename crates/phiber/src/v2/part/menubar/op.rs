//! What a menu item does when it is not an operator of its own.
//!
//! Blender ref: `WM_operator_name_call` — a menu item calls an operator, and
//! an operator that cannot run reports why (`BKE_report`) to the status bar
//! and the Info editor. Here every item that has no operator yet fires
//! `ctx.op.report` with its `Menu ▸ Item` path, so nothing a menu offers is
//! ever silent: the status bar says what was asked and that it is not here
//! yet. A checkable item (`tog`) fires `ctx.op.flip` with the key it flips —
//! a `Set`, so the menu closes as Blender's does on a toggle.

use crate::v2::part::text::info::{self, BReportLevel};
use crate::v2::state::{BKey as _, BPath, BState, OpKey, log, rule::BRule, val::{V_FALSE, V_TRUE}};

/// What the status bar says of an item that has no operator.
pub fn not_available(item: &str) -> String {
    format!("{item}: not available yet")
}

/// `ctx.op.report` → one line in the status bar and the Info editor.
pub struct BReportRule;

impl BRule for BReportRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Report.path()
    }
    fn after(&self, _: &BPath, item: &str, state: &mut BState) {
        let text = not_available(item);
        log::push(text.clone());
        info::push(state, BReportLevel::Warning, text);
        state.remove(OpKey::Report);
    }
}

/// `ctx.op.flip` → the named key toggled between `true` and `false`. A
/// key written `!ctx.…` is on by default (a zone's "Show …"): it reads as
/// on unless `false`, so the first flip turns it off.
pub struct BFlipRule;

impl BRule for BFlipRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Flip.path()
    }
    fn after(&self, _: &BPath, spelt: &str, state: &mut BState) {
        let (on_by_default, key) = match spelt.strip_prefix('!') {
            Some(key) => (true, key),
            None => (false, spelt),
        };
        let on = if on_by_default { state.get_encoded(key) != Some(V_FALSE) } else { state.get_encoded(key) == Some(V_TRUE) };
        let next = if on { V_FALSE } else { V_TRUE };
        state.set_encoded(key, next);
        log::push(format!("{}: {}", title_of(key), if next == V_TRUE { "on" } else { "off" }));
        state.remove(OpKey::Flip);
    }
}

/// `ctx.screen.lock_interface` → "Lock Interface".
fn title_of(key: &str) -> String {
    let last = key.rsplit('.').next().unwrap_or(key);
    let mut out = String::new();
    for word in last.split('_') {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push(first.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    #[test]
    fn an_item_without_an_operator_is_reported_and_the_intent_is_not_kept() {
        let mut state = BState::new();
        apply_action(&mut state, &BAction::Set(OpKey::Report.path(), "File ▸ Revert".into()));
        assert!(state.get(OpKey::Report).is_none());
        let reports = info::reports(&state);
        assert_eq!(reports.last().map(|(_, m)| m.as_str()), Some("File ▸ Revert: not available yet"));
        assert!(log::entries().iter().any(|e| e.text == "File ▸ Revert: not available yet"));
    }

    #[test]
    fn a_flip_toggles_the_named_key_and_reads_as_a_sentence() {
        let mut state = BState::new();
        let key = crate::v2::state::ScreenKey::LockInterface.path().encode();
        apply_action(&mut state, &BAction::Set(OpKey::Flip.path(), key.clone()));
        assert_eq!(state.get_encoded(&key), Some(V_TRUE));
        apply_action(&mut state, &BAction::Set(OpKey::Flip.path(), key.clone()));
        assert_eq!(state.get_encoded(&key), Some(V_FALSE));
        assert!(state.get(OpKey::Flip).is_none());
        assert_eq!(title_of(&key), "Lock Interface");
        // On by default: the first flip turns it off.
        let zone = crate::v2::state::ZoneKey::Shown(crate::v2::zone::BZoneType::Bottom).path().encode();
        apply_action(&mut state, &BAction::Set(OpKey::Flip.path(), format!("!{zone}")));
        assert_eq!(state.get_encoded(&zone), Some(V_FALSE));
        apply_action(&mut state, &BAction::Set(OpKey::Flip.path(), format!("!{zone}")));
        assert_eq!(state.get_encoded(&zone), Some(V_TRUE));
    }
}
