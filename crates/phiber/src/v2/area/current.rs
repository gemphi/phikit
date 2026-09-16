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
use crate::v2::host::{BHostType, host_slug};
use crate::v2::space::BSpaceType;
use crate::v2::state::{BPath, key::P_AREA};

thread_local! {
    static SPACE: RefCell<Option<BSpaceType>> = const { RefCell::new(None) };
    static AREA:  RefCell<Option<BAreaSpec>>  = const { RefCell::new(None) };
}

/// Run `f` with `space` recorded as the space being built.
pub fn with_space<R>(space: BSpaceType, f: impl FnOnce() -> R) -> R {
    let prev = SPACE.with(|c| c.borrow_mut().replace(space));
    let out = f();
    SPACE.with(|c| *c.borrow_mut() = prev);
    out
}

/// Run `f` with `area` recorded as the area being built.
pub fn with_area<R>(area: BAreaSpec, f: impl FnOnce() -> R) -> R {
    let prev = AREA.with(|c| c.borrow_mut().replace(area));
    let out = f();
    AREA.with(|c| *c.borrow_mut() = prev);
    out
}

pub fn current_space() -> Option<BSpaceType> {
    SPACE.with(|c| *c.borrow())
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
pub fn area_key(space: BSpaceType, area: &BAreaSpec) -> BPath {
    BPath::owned(vec![
        P_AREA,
        space.slug(),
        area.kind.label(),
        host_slug(area.default_host),
    ])
}

/// `ctx.switcher.<space>.<slot>.<default_host>` — open/closed flag for that
/// area's editor-type popover.
pub fn switcher_key(space: BSpaceType, area: &BAreaSpec) -> BPath {
    BPath::owned(vec![
        "switcher",
        space.slug(),
        area.kind.label(),
        host_slug(area.default_host),
    ])
}

/// The host an area is currently showing: its stored choice, or its default.
pub fn active_host(
    space: BSpaceType,
    area: &BAreaSpec,
    get: impl FnOnce(&BPath) -> Option<String>,
) -> BHostType {
    get(&area_key(space, area))
        .as_deref()
        .and_then(BHostType::from_slug)
        .unwrap_or(area.default_host)
}
