//! Which corner the viewport's gizmo cluster sits in, and how far in.
//!
//! Blender lets the navigation gizmo live in any corner. Wherever it is, it
//! keeps clear of the floating toolbar and sidebar, which move and resize
//! with the strips themselves — so the gizmo reads its distance from the same
//! place the strips do (`pane::floating_strip_room`).

use crate::v2::view::draw::ctx::BRenderCtx;
use crate::v2::view::draw::pane::floating_strip_room;
use crate::v2::state::BPath;
use gpui::{Div, Styled, px};

/// `ctx.view3d.gizmo.corner` — one of the four corners.
pub fn k_corner() -> BPath {
    use crate::v2::state::BKey as _;
    crate::v2::state::View3dKey::GizmoCorner.path()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl BCorner {
    pub fn slug(self) -> &'static str {
        match self {
            BCorner::TopLeft => "top_left",
            BCorner::TopRight => "top_right",
            BCorner::BottomLeft => "bottom_left",
            BCorner::BottomRight => "bottom_right",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        [
            BCorner::TopLeft,
            BCorner::TopRight,
            BCorner::BottomLeft,
            BCorner::BottomRight,
        ]
        .into_iter()
        .find(|corner| corner.slug() == slug)
    }

    pub fn is_right(self) -> bool {
        matches!(self, BCorner::TopRight | BCorner::BottomRight)
    }

    /// The corner on the same side, at the other end. The compass sits in the
    /// chosen corner and the square control clusters sit here, so the two
    /// never stack into one tall column over the scene.
    pub fn flipped(self) -> Self {
        match self {
            BCorner::TopLeft => BCorner::BottomLeft,
            BCorner::TopRight => BCorner::BottomRight,
            BCorner::BottomLeft => BCorner::TopLeft,
            BCorner::BottomRight => BCorner::TopRight,
        }
    }

    fn is_top(self) -> bool {
        matches!(self, BCorner::TopLeft | BCorner::TopRight)
    }

    /// Pin an overlay into this corner, clear of the strip on that side.
    pub fn place(self, element: Div, ctx: &BRenderCtx) -> Div {
        let edge = px(ctx.style.spacing.pad_md);
        let side = px(ctx.style.spacing.pad_md + floating_strip_room(self.is_right(), ctx));
        let element = if self.is_top() { element.top(edge) } else { element.bottom(edge) };
        if self.is_right() {
            element.right(side)
        } else {
            element.left(side)
        }
    }
}

/// The corner the gizmo currently sits in.
pub fn gizmo_corner(ctx: &BRenderCtx) -> BCorner {
    ctx.state.get(&k_corner())
        .and_then(BCorner::from_slug)
        .unwrap_or(BCorner::TopRight)
}

/// Give an opaque canvas surface the corners it inherits from its host.
/// One rule for every canvas, so the viewport and the node editors round the
/// same way — see [`super::super::round`].
pub fn round_surface<E: Styled>(element: E, ctx: &BRenderCtx) -> E {
    super::super::round(element, ctx)
}
