//! `screen.screen_full_area` (Ctrl Space, every editor's View ▸ Toggle
//! Maximize Area): one area grown to fill the whole workspace, and put back
//! by the same press.
//!
//! Blender maximizes the area under the pointer. A Phiber menu entry knows
//! the editor it belongs to, not the pointer, so the op carries that
//! editor's slug and the rule maximizes the area *showing* it — the same
//! area the menu came from, since a menu is drawn in its area's header; two
//! areas showing one editor maximize the first — deliberate, recorded. The
//! key records the space with the area, so each workspace keeps its own
//! maximize and switching workspaces shows the others whole.

use crate::v2::area::current::active_host;
use crate::v2::host::BHostType;
use crate::v2::space::{BSpaceList, BSpaceType};
use crate::v2::state::{AreaKey, BKey as _, BPath, BState, OpKey, log, rule::BRule};

/// `ctx.area.maximized`: `<space slug>|<area label>` while an area fills its
/// workspace. Not resumed (`app::session::is_transient`): a window reopens
/// whole, as Blender's does.
pub fn key() -> BPath {
    AreaKey::Maximized.path()
}

/// The area label maximized in `space`, if one is.
pub fn maximized_in(s: &BState, space: BSpaceType) -> Option<String> {
    let value = s.get(&key())?;
    let (owner, label) = value.split_once('|')?;
    (owner == space.slug()).then(|| label.to_string())
}

pub struct BAreaMaximizeRule;

impl BRule for BAreaMaximizeRule {
    fn wants(&self, path: &BPath) -> bool {
        OpKey::of(path) == Some(OpKey::AreaMaximize)
    }

    fn after(&self, _path: &BPath, value: &str, s: &mut BState) {
        let space = BSpaceList::active(s);
        let said = if maximized_in(s, space).is_some() {
            // The same press puts the area back, whichever menu it came from.
            s.remove_encoded(&key().encode());
            "Areas restored"
        } else {
            match BHostType::from_slug(value).and_then(|host| {
                space.spec().areas.iter().find(|area| active_host(space, area, |k| s.get(k).map(str::to_string)) == host)
            }) {
                Some(area) => {
                    s.set(&key(), format!("{}|{}", space.slug(), area.kind.label()));
                    "Area maximized (Ctrl Space to restore)"
                }
                // The palette fires menu tables as the top bar; nothing to grow.
                None => "No area shows that editor here",
            }
        };
        log::push(said.to_string());
        s.remove(OpKey::AreaMaximize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::{CtxKey, action::apply_action};
    use crate::v2::view::prim::BAction;

    fn fire(s: &mut BState, host: &str) {
        apply_action(s, &BAction::Set(OpKey::AreaMaximize.path(), host.into()));
    }

    #[test]
    fn ctrl_space_grows_the_editors_area_and_the_same_press_puts_it_back() {
        let mut s = BState::new();
        s.set(CtxKey::Space, BSpaceType::Layout.slug());
        fire(&mut s, "view3d");
        assert_eq!(maximized_in(&s, BSpaceType::Layout).as_deref(), Some("Center"));
        assert!(maximized_in(&s, BSpaceType::Shading).is_none(), "another workspace stays whole");
        assert!(s.get(OpKey::AreaMaximize).is_none(), "the op is consumed");
        // The outliner lives in the Right area of Layout.
        fire(&mut s, "outliner");
        assert!(maximized_in(&s, BSpaceType::Layout).is_none(), "the second press restores, whichever menu it came from");
        // A slug that is no editor changes nothing (the palette's top bar).
        fire(&mut s, "no-such-editor");
        assert!(maximized_in(&s, BSpaceType::Layout).is_none());
    }

    #[test]
    fn a_maximized_space_draws_one_area_the_whole_window() {
        use crate::v2::view::{BView, BViewList as _};
        let mut s = BState::new();
        s.set(CtxKey::Space, BSpaceType::Layout.slug());
        // The zone list reads the space being built (`with_space`), as the
        // space renderer sets it.
        let zones = |s: &BState| crate::v2::area::current::with_space(BSpaceType::Layout, || crate::v2::zone::BZoneList::new(BSpaceType::Layout.spec()).items(s));
        let whole = zones(&s);
        fire(&mut s, "view3d");
        let one = zones(&s);
        assert!(one.len() == 1 && whole.len() > 1, "{} zones maximized, {} whole", one.len(), whole.len());
        let BView::Zone { kind, list } = &one[0] else { panic!("a zone") };
        assert_eq!(*kind, crate::v2::zone::BZoneType::Center, "the one area takes the filling zone");
        assert_eq!(list.len(), 1);
        // Another workspace is untouched.
        s.set(CtxKey::Space, BSpaceType::Shading.slug());
        assert!(crate::v2::zone::BZoneList::new(BSpaceType::Shading.spec()).items(&s).len() > 1);
    }
}
