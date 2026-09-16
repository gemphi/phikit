//! GPUI path painting for projected meshes.

use super::BMesh;
use gpui::{Bounds, PathBuilder, Pixels, Point, Window, point, px};

pub(super) fn meshes(
    bounds: Bounds<Pixels>,
    meshes: &[BMesh],
    outline_width: f32,
    window: &mut Window,
) {
    for mesh in meshes {
        for triangle in &mesh.triangles {
            let mut builder = PathBuilder::fill();
            builder.move_to(project(bounds, triangle.points[0]));
            builder.line_to(project(bounds, triangle.points[1]));
            builder.line_to(project(bounds, triangle.points[2]));
            builder.close();
            if let Ok(path) = builder.build() {
                window.paint_path(path, triangle.fill);
            }
        }
        if let Some(hull) = &mesh.outline {
            if hull.len() < 3 {
                continue;
            }
            let mut builder = PathBuilder::stroke(px(outline_width));
            builder.move_to(project(bounds, hull[0]));
            for point in hull.iter().skip(1) {
                builder.line_to(project(bounds, *point));
            }
            builder.close();
            if let Ok(path) = builder.build() {
                window.paint_path(path, mesh.outline_color);
            }
        }
    }
}

fn project(bounds: Bounds<Pixels>, point_2d: (f32, f32)) -> Point<Pixels> {
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    point(
        bounds.origin.x + px((0.5 + point_2d.0) * width),
        bounds.origin.y + px((0.5 + point_2d.1) * height),
    )
}
