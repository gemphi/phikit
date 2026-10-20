//! The menubars' keys — which popover is open, under
//! `ctx.menu.<host>.<Label>[.<Sub>…]`.
//!
//! Every key is scoped to the host whose header owns the menubar. Without
//! the host segment, two hosts that both declare a "View" menu (the Shading
//! workspace shows six such hosts at once) would share one key, so opening
//! View in one header would deploy every other header's View popover too.

use super::BKey;
use crate::v2::state::BPath;

const MENU: &str = "menu";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuKey<'a> {
    /// A menubar label's popover: `ctx.menu.<host>.<Label>`.
    Top(&'static str, &'static str),
    /// A menu at any depth: `ctx.menu.<host>.<Label>.<Sub>…`, the labels
    /// from the top down.
    Nested(&'static str, &'a [&'static str]),
    /// A header panel popover (`layout.popover(panel=…)`, the Timeline's
    /// Playback): `ctx.header.<host>.panel.<Label>`. Not a menu key, so a
    /// box ticked inside it does not close it; a menu opening does.
    Panel(&'static str, &'static str),
}

impl MenuKey<'_> {
    /// What every panel popover's key contains, for closing them.
    pub fn panel_mark() -> &'static str {
        ".panel."
    }

    /// The prefix every menu key starts with, for closing every open menu.
    pub fn prefix() -> String {
        format!("{}.", BPath::owned(vec![MENU]).encode())
    }

    /// The prefix of one host's menu keys.
    pub fn host_prefix(host: &str) -> String {
        format!("{}{host}.", Self::prefix())
    }

    /// `ctx.menu.<host>.<label>` — a menubar label, not a submenu.
    pub fn is_top_level(path: &BPath) -> bool {
        path.segs.len() == 3 && path.segs[0] == MENU
    }

    /// The same test on an encoded key: exactly one dot after `ctx.menu.`.
    pub fn is_top_level_encoded(encoded: &str) -> bool {
        encoded
            .strip_prefix(&Self::prefix())
            .is_some_and(|rest| rest.matches('.').count() == 1)
    }

    /// The editor a menu key's host segment names: the segment itself, or
    /// its slug before the `@` that a second area showing the same editor
    /// adds (`area::current::menu_scope`).
    pub fn host_of(segment: &str) -> &str {
        segment.split('@').next().unwrap_or(segment)
    }

    /// Whether `path` is a menu key at all.
    pub fn owns(path: &BPath) -> bool {
        path.segs.first() == Some(&MENU)
    }
}

impl BKey for MenuKey<'_> {
    fn path(&self) -> BPath {
        match self {
            Self::Top(host, label) => BPath::owned(vec![MENU, host, label]),
            Self::Panel(host, label) => BPath::owned(vec!["header", host, "panel", label]),
            Self::Nested(host, labels) => {
                let mut segs: Vec<&'static str> = Vec::with_capacity(labels.len() + 2);
                segs.push(MENU);
                segs.push(host);
                segs.extend(labels.iter().copied());
                BPath::owned(segs)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_nested_key_extends_the_top_one() {
        assert_eq!(MenuKey::Top("topbar", "File").path().encode(), "ctx.menu.topbar.File");
        assert_eq!(
            MenuKey::Nested("topbar", &["File", "Import"]).path().encode(),
            "ctx.menu.topbar.File.Import"
        );
        assert!(MenuKey::is_top_level(&MenuKey::Top("topbar", "File").path()));
        assert!(!MenuKey::is_top_level(&MenuKey::Nested("topbar", &["File", "Import"]).path()));
        assert!(MenuKey::is_top_level_encoded("ctx.menu.view3d.View"));
        assert!(!MenuKey::is_top_level_encoded("ctx.menu.view3d.View.Cameras"));
        assert_eq!(MenuKey::host_prefix("view3d"), "ctx.menu.view3d.");
    }
}
