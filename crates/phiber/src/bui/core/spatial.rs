//! SpatialView — 3D wireframe canvas element with Blender-accurate default scene.
//! Encapsulates mesh projection, wireframe canvas painting, 3D grid, default cube, and orientation gizmo.

use super::super::bview::{ActionHandler, BAction};
use super::super::state::{BKey, WireframeMesh};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, Stateful, div, px};
use std::sync::Arc;

pub struct SpatialView {
    pub mesh: Option<Arc<WireframeMesh>>,
    pub filename: String,
    pub objects: usize,
    pub vertices: usize,
    /// Viewport camera: yaw°, pitch°, zoom factor.
    pub camera: (f32, f32, f32),
}

impl SpatialView {
    pub fn new(
        mesh: Option<Arc<WireframeMesh>>,
        filename: String,
        objects: usize,
        vertices: usize,
    ) -> Self {
        Self {
            mesh,
            filename,
            objects,
            vertices,
            camera: (45.0, 30.0, 1.0),
        }
    }

    pub fn with_camera(mut self, camera: (f32, f32, f32)) -> Self {
        self.camera = camera;
        self
    }

    pub fn with_elm(&self, p: &BPalette, on_action: ActionHandler) -> Stateful<gpui::Div> {
        // Blender shows view/scene info as overlay text inside the viewport
        // (top-left), not as a header bar — the region Header owns titles.
        let info_str = if self.mesh.is_some() {
            format!(
                "{} — {} objects, {} vertices",
                self.filename, self.objects, self.vertices
            )
        } else {
            "User Perspective  |  Collection | Cube".to_string()
        };

        let mesh_opt = self.mesh.clone();
        let camera = self.camera;
        let active_wire = p.wire_active;
        let text_color = p.text;
        let dim_color = p.text_dim;
        let grid_color = p.viewport_grid;
        let axis_x_color = p.axis_x;
        let axis_y_color = p.axis_y;

        let viewport = gpui::canvas(
            move |bounds, _window, _cx| bounds,
            move |bounds, _state, window, _cx| {
                let cx: f32 = f32::from(bounds.origin.x) + f32::from(bounds.size.width) / 2.0;
                let cy: f32 = f32::from(bounds.origin.y) + f32::from(bounds.size.height) / 2.0;
                let half_w: f32 = f32::from(bounds.size.width) / 2.0;
                let half_h: f32 = f32::from(bounds.size.height) / 2.0;

                // Camera rotation from state (Blender default: yaw 45°, pitch 30°).
                let (yaw_deg, pitch_deg, zoom) = camera;
                let yaw = yaw_deg.to_radians();
                let pitch = pitch_deg.to_radians();
                let cos_y = yaw.cos();
                let sin_y = yaw.sin();
                let cos_p = pitch.cos();
                let sin_p = pitch.sin();

                let rot = |v: &[f32; 3]| -> [f32; 3] {
                    // Rotate around Z (yaw) then X (pitch)
                    let x1 = cos_y * v[0] - sin_y * v[1];
                    let y1 = sin_y * v[0] + cos_y * v[1];
                    let z1 = v[2];

                    let x2 = x1;
                    let y2 = cos_p * y1 - sin_p * z1;
                    let z2 = sin_p * y1 + cos_p * z1;
                    [x2, y2, z2]
                };

                if let Some(mesh_arc) = &mesh_opt {
                    if mesh_arc.vertices.is_empty() {
                        return;
                    }
                    let center = [
                        (mesh_arc.bounds_min[0] + mesh_arc.bounds_max[0]) / 2.0,
                        (mesh_arc.bounds_min[1] + mesh_arc.bounds_max[1]) / 2.0,
                        (mesh_arc.bounds_min[2] + mesh_arc.bounds_max[2]) / 2.0,
                    ];
                    let extent = [
                        (mesh_arc.bounds_max[0] - mesh_arc.bounds_min[0]).max(0.001),
                        (mesh_arc.bounds_max[1] - mesh_arc.bounds_min[1]).max(0.001),
                        (mesh_arc.bounds_max[2] - mesh_arc.bounds_min[2]).max(0.001),
                    ];
                    let max_extent = extent[0].max(extent[1]).max(extent[2]);
                    let scale = (half_w.min(half_h) * 0.75 * zoom) / max_extent;
                    let fov_dist = 6.0 * max_extent;

                    let project = |v: &[f32; 3]| -> gpui::Point<gpui::Pixels> {
                        let rel = [v[0] - center[0], v[1] - center[1], v[2] - center[2]];
                        let r = rot(&rel);
                        let dz = r[2] + fov_dist;
                        let persp = fov_dist / dz.max(0.1);
                        gpui::point(px(cx + r[0] * scale * persp), px(cy - r[1] * scale * persp))
                    };

                    let mut path_builder = gpui::PathBuilder::stroke(px(0.5));
                    for face in &mesh_arc.faces {
                        let v0 = mesh_arc.vertices.get(face[0] as usize);
                        let v1 = mesh_arc.vertices.get(face[1] as usize);
                        let v2 = mesh_arc.vertices.get(face[2] as usize);
                        if v0.is_none() || v1.is_none() || v2.is_none() {
                            continue;
                        }
                        let p0 = project(v0.unwrap());
                        let p1 = project(v1.unwrap());
                        let p2 = project(v2.unwrap());
                        path_builder.move_to(p0);
                        path_builder.line_to(p1);
                        path_builder.line_to(p2);
                        path_builder.line_to(p0);
                    }
                    if let Ok(path) = path_builder.build() {
                        window.paint_path(path, text_color);
                    }
                } else {
                    // Default Blender Scene: 3D Grid + Axes + Default Cube
                    let scale = ((half_w.min(half_h) * 0.45).max(60.0)) * zoom;
                    let fov_dist = 8.0;

                    let project = |v: &[f32; 3]| -> gpui::Point<gpui::Pixels> {
                        let r = rot(v);
                        let dz = r[2] + fov_dist;
                        let persp = fov_dist / dz.max(0.1);
                        gpui::point(px(cx + r[0] * scale * persp), px(cy - r[1] * scale * persp))
                    };

                    // 1. Grid Floor
                    let grid_size = 4;
                    let grid_step = 0.5_f32;
                    let mut grid_path = gpui::PathBuilder::stroke(px(0.5));
                    for i in -grid_size..=grid_size {
                        let val = i as f32 * grid_step;
                        let extent = grid_size as f32 * grid_step;
                        // X parallel lines
                        grid_path.move_to(project(&[-extent, val, -1.0]));
                        grid_path.line_to(project(&[extent, val, -1.0]));
                        // Y parallel lines
                        grid_path.move_to(project(&[val, -extent, -1.0]));
                        grid_path.line_to(project(&[val, extent, -1.0]));
                    }
                    if let Ok(path) = grid_path.build() {
                        window.paint_path(path, grid_color);
                    }

                    // 2. Axes (Red for X, Green for Y)
                    let extent = grid_size as f32 * grid_step;
                    let mut x_axis = gpui::PathBuilder::stroke(px(1.0));
                    x_axis.move_to(project(&[-extent, 0.0, -1.0]));
                    x_axis.line_to(project(&[extent, 0.0, -1.0]));
                    if let Ok(path) = x_axis.build() {
                        window.paint_path(path, axis_x_color);
                    }

                    let mut y_axis = gpui::PathBuilder::stroke(px(1.0));
                    y_axis.move_to(project(&[0.0, -extent, -1.0]));
                    y_axis.line_to(project(&[0.0, extent, -1.0]));
                    if let Ok(path) = y_axis.build() {
                        window.paint_path(path, axis_y_color);
                    }

                    // 3. Default Cube (Centered at origin)
                    let s = 1.0_f32;
                    let cube_verts = [
                        [-s, -s, -s],
                        [s, -s, -s],
                        [s, s, -s],
                        [-s, s, -s],
                        [-s, -s, s],
                        [s, -s, s],
                        [s, s, s],
                        [-s, s, s],
                    ];
                    let edges = [
                        (0, 1),
                        (1, 2),
                        (2, 3),
                        (3, 0),
                        (4, 5),
                        (5, 6),
                        (6, 7),
                        (7, 4),
                        (0, 4),
                        (1, 5),
                        (2, 6),
                        (3, 7),
                    ];

                    let mut cube_path = gpui::PathBuilder::stroke(px(1.2));
                    for &(i, j) in &edges {
                        cube_path.move_to(project(&cube_verts[i]));
                        cube_path.line_to(project(&cube_verts[j]));
                    }
                    if let Ok(path) = cube_path.build() {
                        window.paint_path(path, active_wire);
                    }

                    // 4. Origin indicator point
                    let mut origin_path = gpui::PathBuilder::stroke(px(2.5));
                    let o = project(&[0.0, 0.0, 0.0]);
                    origin_path.move_to(o);
                    origin_path.line_to(gpui::point(o.x + px(1.0), o.y));
                    if let Ok(path) = origin_path.build() {
                        window.paint_path(path, dim_color);
                    }
                }
            },
        )
        .flex_1()
        .size_full()
        .bg(p.fill_solid(p.bg_viewport))
        .into_any_element();

        // In-canvas overlay: view label top-left (Blender viewport text).
        let overlay = div()
            .absolute()
            .top(px(p.pad_sm))
            .left(px(p.pad_sm))
            .text_size(px(11.0))
            .text_color(p.text_dim)
            .child(SharedString::from(info_str));

        // Viewport interaction (Blender: MMB orbits, wheel zooms).
        let down_handler = on_action.clone();
        let move_handler = on_action.clone();
        let wheel_handler = on_action.clone();
        div()
            .id("viewport")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .relative()
            .child(viewport)
            .child(overlay)
            .on_mouse_down(gpui::MouseButton::Middle, move |ev, window, cx| {
                let x = f32::from(ev.position.x) as i32;
                let y = f32::from(ev.position.y) as i32;
                down_handler(&BAction::Set(BKey::CamDragX, x.to_string()), window, cx);
                down_handler(&BAction::Set(BKey::CamDragY, y.to_string()), window, cx);
            })
            .on_mouse_move(move |ev, window, cx| {
                if ev.pressed_button == Some(gpui::MouseButton::Middle) {
                    let x = f32::from(ev.position.x) as i32;
                    let y = f32::from(ev.position.y) as i32;
                    move_handler(&BAction::Orbit { x, y }, window, cx);
                }
            })
            .on_scroll_wheel(move |ev, window, cx| {
                let dy = match ev.delta {
                    gpui::ScrollDelta::Pixels(pt) => f32::from(pt.y),
                    gpui::ScrollDelta::Lines(pt) => pt.y * 20.0,
                };
                if dy.abs() > 0.01 {
                    wheel_handler(&BAction::ZoomDelta(dy as i32), window, cx);
                }
            })
    }

    pub fn into_elm(&self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        self.with_elm(p, on_action).into_any_element()
    }
}
