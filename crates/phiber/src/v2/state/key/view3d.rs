//! The 3D viewport's keys — `RegionView3D` and `View3DShading`, under
//! `ctx.view3d.*`.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum View3dKey {
    /// Orbit yaw, degrees.
    Yaw,
    /// Orbit pitch, degrees, clamped to ±90.
    Pitch,
    /// Dolly factor, 1 = the framed distance.
    Zoom,
    /// `"false"` for orthographic.
    Perspective,
    PanX,
    PanY,
    /// `View3DShading::type` (Wireframe, Solid, Material, Rendered).
    Shading,
    /// X-ray in Solid (`View3DShading.show_xray`, off by default) and its
    /// alpha (`xray_alpha`, 0.5); X-ray in Wireframe (`show_xray_wireframe`,
    /// on) and its alpha (`xray_alpha_wireframe`, 0) — `scene::shading::BXray`.
    Xray,
    XrayAlpha,
    XrayWireframe,
    XrayAlphaWireframe,
    /// One of `View3DShading`'s Solid settings by its RNA name: the
    /// effects (`show_object_outline`, `object_outline_color`,
    /// `show_cavity`, `cavity_type`, the four factors —
    /// `scene::solid_fx::BSolidFx`) and Lighting / Color / Background
    /// (`light`, `color_type`, `single_color`, `background_type`,
    /// `background_color` — `scene::solid_shade::BSolidShade`).
    Shade(&'static str),
    /// A named viewpoint request (front, top, …), consumed by `follow`.
    View,
    /// The orbit pivot, three floats.
    Pivot,
    /// `View3D.lens` — the viewport's focal length, mm (50 until set):
    /// the sidebar's View ▸ Focal Length, and what a file's view restores.
    Lens,
    /// Which corner the navigation gizmo sits in.
    GizmoCorner,
    /// `view3d.select_box` armed (B): the next drag on the canvas is the
    /// box, not an orbit.
    BoxSelect,
    /// `view3d.render_border` armed: the next drag is the render region.
    RenderBorder,
    /// The view's own render region outside the camera view
    /// (`View3D.render_border`, `V3D_RENDER_BORDER`): whether it is on
    /// (`use_render_border`) and its rectangle, `xmin,xmax,ymin,ymax`
    /// (`scene::view_border::BViewBorder`).
    BorderUse,
    BorderRect,
    /// The box while it is dragged, `x0,y0,x1,y1` in canvas pixels.
    BoxRect,
    /// `view3d.select_circle` on (C) — until the right button or Escape.
    CircleSelect,
    /// The circle brush's centre, `x,y` in canvas pixels.
    CircleAt,
    /// The brush's radius in pixels, kept between uses.
    CircleRadius,
    /// The lasso's outline while it is drawn, `x,y;x,y;…` in canvas pixels.
    LassoPath,
    /// G / R / S under way (`transform.translate` / `rotate` / `resize`):
    /// what it started from (`scene::xform::BTransformModal`).
    Transform,
    /// Whether one area's viewport looks through the scene camera
    /// (`RegionView3D.persp == RV3D_CAMOB`): `ctx.view3d.<scope>.<slot>.camera`.
    /// The first slot of the view that is kept per area rather than shared.
    Camera(&'static str, &'static str),
    /// One area's own value of a view field (`yaw`, `pitch`, `zoom`,
    /// `perspective`, `pan_x`, `pan_y`, `pivot`, `lens`):
    /// `ctx.view3d.<scope>.<slot>.<field>` — each 3D Viewport is a
    /// `RegionView3D` of its own (`scene::view_slot`).
    Slot(&'static str, &'static str, &'static str),
}

impl View3dKey {
    /// The name a view field has under an area, for the keys an area keeps
    /// its own copy of; `None` for the rest.
    pub const fn field(self) -> Option<&'static str> {
        Some(match self {
            Self::Yaw => "yaw",
            Self::Pitch => "pitch",
            Self::Zoom => "zoom",
            Self::Perspective => "perspective",
            Self::PanX => "pan_x",
            Self::PanY => "pan_y",
            Self::Pivot => "pivot",
            Self::Lens => "lens",
            _ => return None,
        })
    }
}

impl BKey for View3dKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Yaw => "yaw",
            Self::Pitch => "pitch",
            Self::Zoom => "zoom",
            Self::Perspective => "perspective",
            Self::PanX => return BPath::owned(vec!["view3d", "pan", "x"]),
            Self::PanY => return BPath::owned(vec!["view3d", "pan", "y"]),
            Self::Shading => "shading",
            Self::Xray => "xray",
            Self::XrayAlpha => "xray_alpha",
            Self::XrayWireframe => "xray_wireframe",
            Self::XrayAlphaWireframe => "xray_alpha_wireframe",
            Self::Shade(rna) => return BPath::owned(vec!["view3d", "shade", rna]),
            Self::View => "view",
            Self::Pivot => "pivot",
            Self::Lens => "lens",
            Self::GizmoCorner => return BPath::owned(vec!["view3d", "gizmo", "corner"]),
            Self::BoxSelect => return BPath::owned(vec!["view3d", "box", "select"]),
            Self::RenderBorder => return BPath::owned(vec!["view3d", "box", "render_border"]),
            Self::BorderUse => return BPath::owned(vec!["view3d", "render_border", "use"]),
            Self::BorderRect => return BPath::owned(vec!["view3d", "render_border", "rect"]),
            Self::BoxRect => return BPath::owned(vec!["view3d", "box", "rect"]),
            Self::CircleSelect => return BPath::owned(vec!["view3d", "circle", "select"]),
            Self::CircleAt => return BPath::owned(vec!["view3d", "circle", "at"]),
            Self::CircleRadius => return BPath::owned(vec!["view3d", "circle", "radius"]),
            Self::LassoPath => return BPath::owned(vec!["view3d", "lasso", "path"]),
            Self::Transform => return BPath::owned(vec!["view3d", "transform", "modal"]),
            Self::Camera(scope, slot) => return BPath::owned(vec!["view3d", scope, slot, "camera"]),
            Self::Slot(scope, slot, field) => return BPath::owned(vec!["view3d", scope, slot, field]),
        };
        BPath::owned(vec!["view3d", tail])
    }
}
