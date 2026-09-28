//! Canvas dispatcher — routes each host to one focused named renderer.

pub mod fallback;
pub mod gizmo;
pub mod image;
pub mod node;
pub mod overlay;
pub mod surface;
pub mod time;
pub mod view3d;

pub use self::surface::{base, round};

use super::ctx::BRenderCtx;
use crate::v2::host::BHostType;
use gpui::AnyElement;

pub fn view(host: BHostType, camera: &crate::v2::scene::camera_view::BCameraSlot, ctx: &BRenderCtx) -> AnyElement {
    match host {
        BHostType::View3D => self::view3d::BView3d { camera }.render(ctx),
        BHostType::Node | BHostType::Shader | BHostType::Compositor | BHostType::GeoNode => {
            self::node::BNode.render(ctx)
        }
        BHostType::Image => self::image::BImage(false).render(ctx),
        BHostType::Uv => self::image::BImage(true).render(ctx),
        BHostType::Time | BHostType::Action | BHostType::Ipo | BHostType::Nla | BHostType::Drivers => self::time::BTime(host).render(ctx),
        _ => self::fallback::BFallback(host).render(ctx),
    }
}
