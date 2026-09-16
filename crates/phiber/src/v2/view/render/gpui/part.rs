//! Part walker — a leaf composer. Its body is the only place primitives
//! may appear.

use super::{ctx::BRenderCtx, walk};
use crate::v2::view::BView;
use gpui::AnyElement;

pub fn view(body: &BView, c: &BRenderCtx) -> AnyElement {
    walk(body, c)
}
