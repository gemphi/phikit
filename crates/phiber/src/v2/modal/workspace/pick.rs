//! Picking a space from the Spaces dialog closes it — from a tile or from
//! anywhere else that switches the space while the dialog is up.

use crate::v2::state::val::{V_EMPTY, V_MODAL_SPACES};
use crate::v2::state::{BKey, BPath, BState, CtxKey, ModalKey, rule::BRule};

/// `space` → close the Spaces dialog if it is the one up.
pub struct BSpacePickedRule;

impl BRule for BSpacePickedRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == CtxKey::Space.path()
    }
    fn after(&self, _: &BPath, _: &str, s: &mut BState) {
        if s.get(ModalKey::Open) == Some(V_MODAL_SPACES) {
            s.set(ModalKey::Open, V_EMPTY);
        }
    }
}
