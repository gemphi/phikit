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
mod hit;
mod mesh;
mod object;
mod orbit;
mod xform;
pub mod rendered;
pub use self::rendered::views_posted;
pub(super) use self::object::name as object_name;

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
        let axis_x = ctx.style.axis_stroke(ctx.style.colors.axis_x);
        let axis_y = ctx.style.axis_stroke(ctx.style.colors.axis_y);
        // The canvas shade; its alpha carries the theme's sweep — how much
        // lighter the left edge is — for the tracer's sky (`Camera::sky.a`).
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
        let selected = crate::v2::scene::select::BSelection::now(ctx.state);
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
        };
        let hidden = crate::v2::part::outliner::key::BHidden::now(ctx.state);
        // The shared list, not a fresh copy: the overlay and the outliner read
        // the same one this pass (`scene_objects`).
        let scene: Rc<Vec<_>> = Rc::new(
            crate::v2::scene::scene_objects(ctx.state)
                .iter()
                .filter(|object| !hidden.hides(ctx.state, self::mesh::name(object)))
                .cloned()
                .collect(),
        );
        // The paint pass knows the canvas's bounds; the click, which needs
        // them to turn a pointer into a ray, reads them from here.
        let bounds: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
        let unselectable = crate::v2::part::outliner::key::BHidden::for_select(ctx.state);
        let hit = self::hit::BHit {
            scene: Rc::clone(&scene),
            extras: Rc::new(crate::v2::scene::source::with_loaded(|m| m.map(|m| crate::v2::scene::pick::extra::BPickExtra::list(ctx.state, m)).unwrap_or_default())),
            unselectable: Rc::new(
                scene.iter().map(|object| self::mesh::name(object).to_string()).filter(|name| unselectable.hides(ctx.state, name)).collect(),
            ),
            view: crate::v2::scene::pick::BPickView { yaw, pitch, pan, zoom, setup, aspect: setup.aspect },
            bounds: Rc::clone(&bounds),
        };
        element = element.child(BOrbit { leave }.render(ctx, hit));
        // Every projection below runs through this frame's camera.
        crate::v2::scene::camera::with_view(setup, || {
        let floor_lines = self::floor::lines(
            (yaw, pitch, pan, zoom),
            setup,
            pivot,
            self::floor::BFloorPalette {
                grid: to_hsla(grid.color),
                axis_x: to_hsla(axis_x.color),
                axis_y: to_hsla(axis_y.color),
            },
        );
        let studio = self::draw::studio(palette.shading);
        let self::draw::BPlan { ray, cpu_draws, mode } = self::draw::plan(palette.shading, &scene, &palette);
        let ray_view = self::rendered::BRayView {
            yaw, pitch, pan, zoom, setup,
            sky: vp,
            grid: to_hsla(grid.color),
            studio,
            // Blender draws unselected wires in the theme's wire colour and
            // the active object in `wire_active`; the kernel carries one wire
            // colour, so it takes the neutral one.
            wire: to_hsla(ctx.style.colors.text_dim),
            mode,
            set: crate::v2::scene::render_set(ctx.state),
        };
        // Projecting every vertex is the most expensive thing a frame does,
        // and it depends only on the camera, the palette and the scene — none
        // of which a resize or a space change touches, so `cache` memoises it.
        let active = ctx.state.get(CtxKey::Object).unwrap_or("").to_string();
        let meshes = self::cache::meshes(cpu_draws, &scene, (yaw, pitch, pan, zoom), palette, &selected, &active);
        // No sort. Ordering objects by the depth of their origin is what hid
        // a character's clothes: one number per object cannot say which of
        // two interleaved surfaces is nearer at a given pixel. The depth
        // buffer answers that per pixel, so the order they arrive in stops
        // mattering (`mesh::raster`).
        let outline_width = ctx.style.strokes.border * 2.0;
        let wire = palette.shading == crate::v2::scene::BShading::Wireframe;
        let frame_palette = self::frame::BFramePalette {
            passepartout: to_hsla(ctx.style.camera_passepartout()),
            back: vp,
            overlay: to_hsla(ctx.style.view_overlay()),
        };
        let extra_lines = self::extra::project(
            ctx.state,
            yaw,
            pitch,
            pan,
            zoom,
            self::extra::BExtraPalette::from_style(ctx, &palette),
        );
        let seen = Rc::clone(&bounds);
        let gestures = self::gesture::BGestureInk::now(ctx.state, vp, frame_palette.overlay, crate::v2::scene::pick::BPickView { yaw, pitch, pan, zoom, setup, aspect: setup.aspect });
        element = element.child(
            canvas(
                |_, _, _| {},
                move |bounds, _, window, _| {
                    seen.set(Some(bounds));
                    let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
                    if crate::v2::scene::camera::note_size(w, h) {
                        window.refresh();
                    }
                    if ray {
                        self::rendered::paint(bounds, ray_view, window);
                    } else {
                        self::extra::paint(bounds, &floor_lines, 1.0, window);
                    }
                    BMesh::paint(bounds, &meshes, outline_width, wire, yaw, pitch, window);
                    self::extra::paint(bounds, &extra_lines, outline_width * 0.5, window);
                    if let Some(view) = &through {
                        self::frame::paint(bounds, view, frame_palette, window);
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
            .child(
                self::corner::gizmo_corner(ctx)
                    .place(div().absolute(), ctx)
                    .child(super::gizmo::BGizmo.compass(ctx)),
            )
            .child(
                self::corner::gizmo_corner(ctx)
                    .flipped()
                    .place(div().absolute(), ctx)
                    .child(super::gizmo::BGizmo.controls(ctx, self.camera)),
            )
            .into_any_element()
    }
}
