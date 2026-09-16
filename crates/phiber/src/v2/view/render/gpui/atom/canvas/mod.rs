//! Canvas dispatcher — routes each host to one focused named renderer.

pub mod fallback;
pub mod gizmo;
pub mod image;
pub mod node;
pub mod overlay;
pub mod view3d;

use super::super::ctx::BRenderCtx;
use crate::v2::host::BHostType;
use gpui::AnyElement;

pub fn view(host: BHostType, c: &BRenderCtx) -> AnyElement {
    match host {
        BHostType::View3D => view3d::BView3d.render(c),
        BHostType::Node | BHostType::Shader | BHostType::Compositor | BHostType::GeoNode => {
            node::BNode.render(c)
        }
        BHostType::Image => image::BImage(false).render(c),
        BHostType::Uv => image::BImage(true).render(c),
        _ => fallback::BFallback(host).render(c),
    }
}
