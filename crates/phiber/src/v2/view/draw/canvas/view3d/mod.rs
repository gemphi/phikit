//! View3D canvas composition; interaction and mesh painting live in focused helpers.

mod boxsel;
mod circlesel;
mod gesture;
mod lasso;
pub mod cache;
pub mod corner;
mod draw;
mod extra;
mod floor;
mod frame;
mod guide;
mod hit;
mod identity;
pub mod mesh;
mod outline;
mod orbit;
mod render_border;
mod shown;
mod xform;
pub mod rendered;
pub use self::rendered::views_posted;

use crate::v2::state::{CtxKey};
pub use self::draw::on_gpu;
pub use self::mesh::rasterised;
use self::mesh::{BMesh, BMeshPalette};
use self::orbit::BOrbit;
use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use crate::v2::scene::{view_zoom, zoom_key, zoom_step};
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{
    AnyElement, Bounds, Pixels, ScrollDelta, ScrollWheelEvent, SharedString, canvas, div,
};
use std::cell::Cell;
use std::rc::Rc;

/// The viewport canvas; `camera` is its camera view — whether it looks
/// through the scene camera now, with its frame and passepartout
/// (`RegionView3D.persp == RV3D_CAMOB`), and the key that toggles it.
pub struct BView3d<'a> {
    pub camera: &'a crate::v2::scene::camera_view::BCameraSlot,
}

impl BView3d<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let through = self::frame::through(self.camera.through);
        let grid = ctx.style.viewport_grid_stroke();
        // The tracer's sky: the canvas shade, with the sweep lift in its alpha
        // — a LIFT, not an opacity, so nothing else may be painted with it
        // (`tests/viewport_lightness_regression.rs`).
        let mut vp = to_hsla(ctx.style.viewport_surface());
        vp.a = ctx.style.viewport_sweep_lift();
        let zoom_handler = ctx.handler.clone();
        let current_zoom = view_zoom(ctx.state);
        // This viewport's own keys (`scene::view_slot`): the wheel zooms it,
        // and the pointer over it is what the view keys and menus act on.
        let (zoom, slot) = (zoom_key(), crate::v2::scene::view_slot::BViewSlot::current());
        let mut element = div()
            .id(SharedString::from("view3d-canvas"))
            .debug_selector(|| "view3d-canvas".to_string())
            .flex_1()
            .min_h_0()
            .size_full()
            .relative()
            .overflow_hidden()
            .bg(super::super::color::resolved_gradient(ctx.style.viewport_gradient()))
            .on_mouse_move(move |_, _, _| slot.into_iter().for_each(|slot| slot.hover()))
            .on_scroll_wheel(move |event: &ScrollWheelEvent, window, app| {
                // Wheel thresholds and factors are input mechanics, not UI design.
                let delta = match event.delta {
                    ScrollDelta::Pixels(point) => f32::from(point.y),
                    ScrollDelta::Lines(point) => point.y * 20.0,
                };
                if delta.abs() < 0.5 {
                    return;
                }
                let factor = if delta > 0.0 { 1.1 } else { 1.0 / 1.1 };
                let next = zoom_step(current_zoom, factor);
                (Rc::clone(&zoom_handler))(
                    &BAction::Set(zoom.clone(), next.to_string()),
                    window,
                    app,
                );
            });

        // An orbit while looking through the camera leaves the camera view
        // and carries its view over (`BCameraView::leave`).
        let leave = through.as_ref().zip(self.camera.key.as_ref()).map(|(view, key)| view.leave(key));

        let self::frame::BViewParams { yaw, pitch, pan, zoom, pivot, setup } =
            self::frame::BViewParams::resolve(ctx.state, through.as_ref());
        // Blender outlines every selected object (`OVERLAY_outline`); the set
        // is the active object until a selection is made (`scene::select`).
        let selected = self::guide::BGuide::outlined(ctx.state);
        // Solid shading: every mesh — a primitive, a loaded model — in the
        // theme's one solid shade (`solid_shade`), so the viewport follows
        // the theme rather than each object bringing a grey of its own.
        let solid = to_hsla(ctx.style.solid_shade());
        let palette = BMeshPalette {
            shading: crate::v2::scene::view_shading(ctx.state),
            cube: solid,
            sphere: solid,
            gem: solid,
            loaded: solid,
            accent: to_hsla(ctx.style.colors.accent),
            accent_active: to_hsla(ctx.style.step(ctx.style.colors.accent, 2)),
            matcap: crate::v2::scene::BSolidShade::now(ctx.state).matcap_for_build(),
            texture: crate::v2::scene::BSolidShade::now(ctx.state).texture_for_build(),
        };
        // The shown objects and the ones a click may not pick, kept between
        // frames (`shown`); the overlay and the outliner read the same
        // objects this pass (`scene_objects`).
        let self::shown::BShown { scene, unselectable } = self::shown::BShown::now(ctx.state);
        // The paint pass knows the canvas's bounds; the click, which needs
        // them to turn a pointer into a ray, reads them from here.
        let bounds: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
        let extras = Rc::new(crate::v2::scene::source::with_loaded(|m| m.map(|m| crate::v2::scene::pick::extra::BPickExtra::list(ctx.state, m)).unwrap_or_default()));
        let hit = self::hit::BHit {
            scene: Rc::clone(&scene),
            extras: Rc::clone(&extras),
            unselectable,
            view: crate::v2::scene::pick::BPickView { yaw, pitch, pan, zoom, setup, aspect: setup.aspect },
            bounds: Rc::clone(&bounds),
        };
        element = element.child(BOrbit { leave, frame: through.as_ref().map(|view| view.frame) }.render(ctx, hit));
        // Every projection below runs through this frame's camera.
        crate::v2::scene::camera::with_view(setup, || {
        let guide = self::guide::BGuide::build(ctx, (yaw, pitch, pan, zoom), setup, pivot, &palette, &scene, &extras);
        let studio = self::draw::studio(palette.shading);
        let self::draw::BPlan { ray, cpu_draws, mode } = self::draw::plan(palette.shading, &scene, &palette);
        // The viewport's own render region, outside the camera view: what
        // the Rendered shading fills (`ED_view3d_calc_render_border`) and
        // the dashed box drawn over the region (`view3d_draw_border`).
        let own_border = through.is_none().then(|| crate::v2::scene::view_border::BViewBorder::now(ctx.state)).filter(|b| b.on);
        let engine = ctx.state.get(crate::v2::state::ButsKey::Value("engine")).unwrap_or("");
        let filled = own_border.and_then(|_| crate::v2::scene::view_border::BViewBorder::filled_by(ctx.state, engine)).filter(|_| !studio);
        let active = ctx.state.get(CtxKey::Object).unwrap_or("").to_string();
        let ray_view = self::rendered::BRayView::of(
            (yaw, pitch, pan, zoom),
            setup,
            vp,
            to_hsla(grid.color),
            studio,
            mode,
            self::frame::dof_words(ctx.state, through.as_ref()),
            ctx,
        )
        .bordered(filled)
        // The objects the tracer's own overlay pass outlines (`outline.rs`).
        .outlining(self::outline::BOutlined::outline_class(&scene[..], &selected, &active));
        // Projecting every vertex is the most expensive thing a frame does,
        // so `cache` memoises it and the GPU path projects nothing at all
        // (`BCpuDraw`). The CPU painter stands in for a frame the tracer
        // has not delivered (`tests/the_viewport_always_draws_something.
        // rs`), so its triangles are projected *then* — in the paint, the
        // moment it turns out there is no image — not on every frame it
        // could be needed, which projected every scene on every orbit
        // frame with the image already on screen.
        let can_fall_back = self::draw::cpu_can_draw(crate::v2::scene::scene_metrics(&scene).2);
        let draws = self::cache::BCpuDraw::of(cpu_draws, ray_view.see_through > 0.0);
        let meshes = self::cache::meshes(draws, &scene, (yaw, pitch, pan, zoom), palette, &selected, &active);
        let fall_back = {
            let (scene, selected) = (Rc::clone(&scene), selected.clone());
            move || self::cache::meshes(self::cache::BCpuDraw::Whole, &scene, (yaw, pitch, pan, zoom), palette, &selected, &active)
        };
        // No sort: one number per object cannot say which of two interleaved
        // surfaces is nearer at a pixel. The depth buffer answers per pixel,
        // so arrival order stops mattering (`mesh::raster`).
        let outline_width = ctx.style.strokes.border * 2.0;
        let wire = palette.shading == crate::v2::scene::BShading::Wireframe;
        let region = crate::v2::scene::source::with_loaded(|m| m.map(|m| m.render.border).unwrap_or_default());
        let region = crate::v2::scene::render_border::BRenderBorder::now(ctx.state, region);
        let frame_palette = self::frame::BFramePalette {
            passepartout: to_hsla(ctx.style.camera_passepartout()),
            back: to_hsla(ctx.style.viewport_surface()),
            overlay: to_hsla(ctx.style.view_overlay()),
            region: region.on.then_some(region.rect),
        };
        let corners = super::surface::corners(ctx);
        let seen = Rc::clone(&bounds);
        let gestures = self::gesture::BGestureInk::now(ctx.state, to_hsla(ctx.style.viewport_surface()), frame_palette.overlay, crate::v2::scene::pick::BPickView { yaw, pitch, pan, zoom, setup, aspect: setup.aspect });
        // The Solid/Wireframe rasteriser oversamples by the Viewport AA preference.
        let supersample = crate::v2::scene::viewport_aa::BViewportAa::supersample(ctx.state);
        element = element.child(
            canvas(
                |_, _, _| {},
                move |bounds, _, window, _| {
                    seen.set(Some(bounds));
                    let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
                    if crate::v2::scene::camera::note_size(w, h) {
                        window.refresh();
                    }
                    // The tracer paints the surface and its own grid; with
                    // nothing yet, the floor and the shaded meshes stand in.
                    let traced = ray && self::rendered::paint(bounds, ray_view, corners, window);
                    if !traced {
                        guide.paint_floor(bounds, window);
                    }
                    let shown = if traced || cpu_draws || !can_fall_back { Rc::clone(&meshes) } else { fall_back() };
                    BMesh::paint(bounds, &shown, outline_width, wire, yaw, pitch, corners, !traced, supersample, window);
                    guide.paint_over(bounds, outline_width * 0.5, window);
                    if let Some(view) = &through {
                        self::frame::paint(bounds, view, frame_palette, window);
                    } else if let Some(own) = own_border {
                        self::frame::paint_region(bounds, own.rect, frame_palette.back, window);
                    }
                    gestures.paint(bounds, window);
                },
            )
            .absolute()
            .inset_0()
            .size_full(),
        );
        // The canvas is the opaque surface that reaches the host's corners, so
        // it paints them itself. GPUI's clip is a rectangle, not a rounded
        // one, so `overflow_hidden` cannot round a child for us — the blue
        // surface was squaring off the zone's corners.
        self::corner::round_surface(element, ctx)
        })
            // Both overlays clear the floating strips, so neither collides
            // with the toolbar or the sidebar as those move or expand.
            .child(
                self::corner::BCorner::TopLeft
                    .place(div().absolute(), ctx)
                    .child(super::overlay::BOverlay { view_name: super::overlay::view_name(through.as_ref().map(|view| view.perspective), setup.perspective) }.render(ctx)),
            )
            // The orientation gizmo and the view buttons, unless Navigate or
            // the gizmos are off (`V3D_GIZMO_HIDE_NAVIGATE`, `V3D_GIZMO_HIDE`).
            .when(crate::v2::scene::BOverlays::now(ctx.state).navigate, |element| {
                element
                    .child(self::corner::gizmo_corner(ctx).place(div().absolute(), ctx).child(super::gizmo::BGizmo.compass(ctx)))
                    .child(self::corner::gizmo_corner(ctx).flipped().place(div().absolute(), ctx).child(super::gizmo::BGizmo.controls(ctx, self.camera)))
            })
            .into_any_element()
    }
}
