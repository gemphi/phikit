//! Render-scoped "which area am I inside" context.
//!
//! The Space → Zone → Area → Host → Region → Pane → Part chain passes only
//! the active `BHostType` downward, but the editor-type switcher sitting in a
//! host header has to write back to *its own area's* slot — and an area is
//! identified by its space, its slot and its default host, none of which the
//! host knows. Widening seven `BViewItem::build` signatures to carry a
//! `BAreaSpec` would touch every level; instead the two levels that DO know
//! push their identity onto a thread-local for the duration of their subtree.
//!
//! This mirrors `v2::scene::source`, which scopes the loaded scene to one
//! render pass the same way. Build is depth-first and synchronous, so the
//! value read by a part is always the area that is currently being built.

use std::cell::RefCell;

use super::BAreaSpec;
use crate::v2::host::BHostType;
use crate::v2::space::BSpaceType;
use crate::v2::state::{AreaKey, BKey as _, BPath};

/// Who owns a set of area slots: a workspace, or a dialog that raises
/// editors the same way (`modal::BModalSpec::areas`). Both build their areas
/// through the one zone builder; the scope keeps their slots apart, so the
/// editor chosen in the file dialog's area never overwrites the workspace's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BScope {
    Space(BSpaceType),
    Modal(crate::v2::modal::BModalType),
}

impl BScope {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Space(s) => s.slug(),
            Self::Modal(m) => m.slug(),
        }
    }
}

impl From<BSpaceType> for BScope {
    fn from(s: BSpaceType) -> Self {
        Self::Space(s)
    }
}
impl From<crate::v2::modal::BModalType> for BScope {
    fn from(m: crate::v2::modal::BModalType) -> Self {
        Self::Modal(m)
    }
}

thread_local! {
    static SCOPE: RefCell<Option<BScope>>    = const { RefCell::new(None) };
    static AREA:  RefCell<Option<BAreaSpec>> = const { RefCell::new(None) };
}

/// Run `f` with `scope` recorded as the owner of the areas being built.
pub fn with_scope<R>(scope: impl Into<BScope>, f: impl FnOnce() -> R) -> R {
    let prev = SCOPE.with(|c| c.borrow_mut().replace(scope.into()));
    let out = f();
    SCOPE.with(|c| *c.borrow_mut() = prev);
    out
}

/// Run `f` with `space` recorded as the space being built.
pub fn with_space<R>(space: BSpaceType, f: impl FnOnce() -> R) -> R {
    with_scope(space, f)
}

/// Run `f` with `area` recorded as the area being built.
pub fn with_area<R>(area: BAreaSpec, f: impl FnOnce() -> R) -> R {
    let prev = AREA.with(|c| c.borrow_mut().replace(area));
    let out = f();
    AREA.with(|c| *c.borrow_mut() = prev);
    out
}

pub fn current_scope() -> Option<BScope> {
    SCOPE.with(|c| *c.borrow())
}

/// The space being built, when the scope is a space.
pub fn current_space() -> Option<BSpaceType> {
    match current_scope() {
        Some(BScope::Space(s)) => Some(s),
        _ => None,
    }
}
pub fn current_area() -> Option<BAreaSpec> {
    AREA.with(|c| *c.borrow())
}

/// `ctx.area.<space>.<slot>.<default_host>` — the slot that holds an area's
/// chosen editor type.
///
/// The space segment matters: ten workspaces declare a Center area whose
/// default host is `View3D`, so a key of `ctx.area.view3d` alone would make
/// switching the Center editor in Layout switch it in every other workspace
/// too. The slot segment separates two areas that share a zone (Shading
/// stacks two Left areas and two Center areas).
pub fn area_key(scope: impl Into<BScope>, area: &BAreaSpec) -> BPath {
    AreaKey::Host(scope.into().slug(), area.kind.label(), area.default_host.slug()).path()
}

/// `ctx.switcher.<space>.<slot>.<default_host>` — open/closed flag for that
/// area's editor-type popover.
pub fn switcher_key(scope: impl Into<BScope>, area: &BAreaSpec) -> BPath {
    AreaKey::Switcher(scope.into().slug(), area.kind.label(), area.default_host.slug()).path()
}

/// The host an area is currently showing: its stored choice, or its default.
pub fn active_host(
    scope: impl Into<BScope>,
    area: &BAreaSpec,
    get: impl FnOnce(&BPath) -> Option<String>,
) -> BHostType {
    get(&area_key(scope, area))
        .as_deref()
        .and_then(BHostType::from_slug)
        .unwrap_or(area.default_host)
}

/// The host segment of a header's menu keys (`ctx.menu.<segment>.<Label>`)
/// for `host` in the area being built.
///
/// Menus are keyed by host so two editors' View menus open apart. Two areas
/// showing the same editor — the Animation workspace's camera view and its
/// 3D Viewport — would still share every key, and Object opened in one
/// would open in both, where Blender opens the header that was clicked. So
/// the first area showing `host` keeps the bare slug (a shortcut that opens
/// a menu by host lands there) and any later one adds its slot:
/// `view3d@Left-view3d`. [`MenuKey::host_of`] reads the editor back.
///
/// [`MenuKey::host_of`]: crate::v2::state::MenuKey::host_of
pub fn menu_scope(s: &crate::v2::state::BState, host: BHostType) -> &'static str {
    let (Some(space), Some(area)) = (current_space(), current_area()) else {
        return host.slug();
    };
    let showing = |a: &&BAreaSpec| active_host(space, a, |k| s.get(k).map(str::to_string)) == host;
    match space.spec().areas.iter().find(showing) {
        Some(first) if first.kind != area.kind || first.default_host != area.default_host => {
            crate::v2::state::intern(&format!("{}@{}-{}", host.slug(), area.kind.label(), area.default_host.slug()))
        }
        _ => host.slug(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::{BState, MenuKey};

    /// The Animation workspace shows the 3D Viewport twice; opening Object
    /// in one must not open it in the other.
    #[test]
    fn a_second_area_showing_the_same_editor_keeps_its_own_menus() {
        let s = BState::new();
        let space = BSpaceType::Animation;
        let views: Vec<&BAreaSpec> = space.spec().areas.iter().filter(|a| a.default_host == BHostType::View3D).collect();
        assert!(views.len() >= 2, "Animation shows two 3D Viewports");
        let scope = |a: &BAreaSpec| with_space(space, || with_area(*a, || menu_scope(&s, BHostType::View3D)));
        assert_eq!(scope(views[0]), "view3d", "the first keeps the bare slug a shortcut opens");
        assert_ne!(scope(views[1]), scope(views[0]), "the second has keys of its own");
        assert_eq!(MenuKey::host_of(scope(views[1])), "view3d", "and still names its editor");
        assert_eq!(menu_scope(&s, BHostType::View3D), "view3d", "outside an area: the slug");
    }
}
