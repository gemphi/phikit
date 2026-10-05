//! The rule that retires a link-drag's pending source.
//!
//! A link-drag search sets `NodeKey::AddLink` and opens the Add menu. Once the
//! menu has done its job it must clear — otherwise the next plain Add menu would
//! open still narrowed and still wiring to a stale socket. "Done" is one of two
//! `Set`s the menu already makes: a node was picked (the `Added` write carries
//! the induced link) or the menu closed (its panel key went false). This reacts
//! to those, so the pick stays a single `Set` — no second write and no compound
//! action, the clean lifecycle the after-set rule table is for.
//!
//! Scoped to the shader editor, like the Add menu itself (`menu::body`).

use crate::v2::host::BHostType;
use crate::v2::state::rule::BRule;
use crate::v2::state::val::V_TRUE;
use crate::v2::state::{BKey as _, BPath, BState, MenuKey, NodeKey};

pub struct BAddLinkClearRule;

impl BAddLinkClearRule {
    fn added() -> BPath {
        NodeKey::Added(BHostType::Shader).path()
    }
    fn panel() -> BPath {
        MenuKey::Panel(crate::v2::host::shader::SLUG, "Add").path()
    }
}

impl BRule for BAddLinkClearRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == Self::added() || *path == Self::panel()
    }

    fn after(&self, path: &BPath, value: &str, s: &mut BState) {
        // A pick writes `Added` with a link on its last addition; a close writes
        // the panel key anything but open. Either retires the pending source.
        let picked = *path == Self::added() && super::decode(value).last().is_some_and(|a| a.link.is_some());
        let closed = *path == Self::panel() && value != V_TRUE;
        if picked || closed {
            s.remove(NodeKey::AddLink(BHostType::Shader));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::state::val::V_FALSE;
    use crate::v2::view::draw::canvas::node::add::{BNodeAddition, BNodeAdditions, BPendingLink, store};
    use crate::v2::view::prim::BAction;

    fn pending() -> BPendingLink {
        BPendingLink { node: "Principled BSDF".into(), socket: "BSDF".into(), from_output: true }
    }

    fn with_pending() -> BState {
        let mut s = BState::new();
        s.set(NodeKey::AddLink(BHostType::Shader), store::encode_pending(&pending()));
        s
    }

    #[test]
    fn picking_a_wired_node_retires_the_pending_link() {
        let mut s = with_pending();
        let adds = BNodeAdditions {
            added: vec![BNodeAddition {
                idname: "ShaderNodeValue".into(),
                name: "Value".into(),
                at: (0.0, 0.0),
                link: Some(pending()),
            }],
        };
        apply_action(&mut s, &BAction::Set(NodeKey::Added(BHostType::Shader).path(), store::encode(&adds.added)));
        assert_eq!(store::pending_of(&s, BHostType::Shader), None, "pending retired after the pick");
    }

    #[test]
    fn closing_the_menu_retires_the_pending_link() {
        let mut s = with_pending();
        apply_action(&mut s, &BAction::Set(panel_path(), V_FALSE.to_string()));
        assert_eq!(store::pending_of(&s, BHostType::Shader), None);
    }

    #[test]
    fn opening_the_menu_keeps_the_pending_link() {
        let mut s = with_pending();
        apply_action(&mut s, &BAction::Set(panel_path(), V_TRUE.to_string()));
        assert!(store::pending_of(&s, BHostType::Shader).is_some(), "opening does not retire it");
    }

    fn panel_path() -> BPath {
        MenuKey::Panel(crate::v2::host::shader::SLUG, "Add").path()
    }
}
