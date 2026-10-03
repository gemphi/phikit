//! Painting a projected scene: one rasterised image, then the overlays.
//!
//! The scene goes through [`raster::BFrame`] — a colour buffer with a depth
//! buffer beside it — and reaches GPUI as a single image. Selection outlines
//! are drawn after, as paths, because they are overlay: they sit on top of
//! the scene rather than inside its depth order, which is how Blender's
//! overlay engine treats them.
//!
//! Wireframe shading keeps the vector path per triangle. It strokes edges
//! rather than filling faces, so there is nothing for a depth buffer to
//! resolve and a stroked path is the better primitive.

use super::BMesh;
use super::raster::BFrame;
use gpui::{Bounds, PathBuilder, Pixels, Point, RenderImage, Window, point, px};
use std::sync::Arc;

/// Rasterise at this multiple of the canvas size, then average down. Triangle
/// fills are not antialiased, so the extra resolution is what keeps a
/// silhouette from stepping.
const SUPERSAMPLE: usize = 2;

/// Ceiling on the rasterised buffer, in pixels, so a large canvas on a
/// high-density display cannot turn into an unbounded allocation.
const MAX_PIXELS: usize = 8_000_000;

thread_local! {
    /// The image painted last frame. GPUI keeps every `RenderImage` it is
    /// handed in its atlas until told to drop it, so each frame's image has
    /// to be released when the next replaces it — the ray tracer's painter
    /// does the same (`rendered::paint`). Without this, every frame of the
    /// CPU painter left one more viewport-sized texture behind.
    static SHOWN: std::cell::RefCell<Option<Arc<RenderImage>>> =
        const { std::cell::RefCell::new(None) };
}

thread_local! {
    /// How many frames the CPU painter has rasterised on this thread.
    static RASTERISED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// How many frames the CPU painter has rasterised — with the GPU drawing,
/// this must not move at all.
pub fn rasterised() -> usize {
    RASTERISED.with(std::cell::Cell::get)
}

/// Release the image painted last frame, if there was one.
fn release_shown(window: &mut Window) {
    if let Some(old) = SHOWN.with(|s| s.borrow_mut().take()) {
        let _ = window.drop_image(old);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn meshes(
    bounds: Bounds<Pixels>,
    meshes: &[BMesh],
    outline_width: f32,
    wire: bool,
    yaw: f32,
    pitch: f32,
    // `corners`: the canvas's own (`canvas::surface::corners`). The raster
    // is what reaches them, and `overflow_hidden` cannot round it, so the
    // pixels are drawn rounded.
    corners: gpui::Corners<Pixels>,
    // `fill`: false when something else already put the surface on the canvas
    // (the tracer's image). The outlines below still draw — they are the
    // selection, not the surface.
    fill: bool,
    window: &mut Window,
) {
    if wire {
        strokes(bounds, meshes, outline_width, yaw, pitch, window);
    } else if fill {
        fills(bounds, meshes, yaw, pitch, corners, window);
    } else {
        release_shown(window);
    }
    // Selection outlines sit on top of the whole scene.
    for mesh in meshes {
        let Some(hull) = &mesh.outline else { continue };
        if hull.len() < 3 {
            continue;
        }
        let mut builder = PathBuilder::stroke(px(outline_width));
        builder.move_to(project(bounds, hull[0]));
        for corner in hull.iter().skip(1) {
            builder.line_to(project(bounds, *corner));
        }
        builder.close();
        if let Ok(path) = builder.build() {
            window.paint_path(path, mesh.outline_color);
        }
    }
}

/// Solid shading: rasterise every triangle with a depth test, lighting each
/// pixel from its own normal, and paint the result once.
fn fills(
    bounds: Bounds<Pixels>,
    meshes: &[BMesh],
    yaw: f32,
    pitch: f32,
    corners: gpui::Corners<Pixels>,
    window: &mut Window,
) {
    // Nothing to fill — the GPU is drawing, or the scene is empty. This used
    // to allocate a supersampled buffer the size of the whole viewport,
    // average it down, upload the (fully transparent) result as a fresh
    // texture and paint it over the real render — on every frame. In a debug
    // build that was three quarters of the entire frame; it is also why a
    // zone drag or a window resize crawled, since both ask for a frame per
    // mouse move.
    if meshes.iter().all(|mesh| mesh.triangles.is_empty()) {
        release_shown(window);
        return;
    }
    let Some((mut frame, scale)) = frame_for(bounds) else { return };
    RASTERISED.with(|n| n.set(n.get() + 1));
    for mesh in meshes {
        for triangle in &mesh.triangles {
            let (base, rough, metal) = (triangle.base, triangle.roughness, triangle.metallic);
            match &triangle.texture {
                // Textured: the surface colour is the texel under this pixel.
                Some(texture) => {
                    let image = &texture.image;
                    frame.shaded(triangle.points, triangle.normals, texture.uvs, |normal, uv| {
                        let texel = super::kernel::linear_rgba(image.sample_smooth(uv));
                        super::kernel::lit(texel, rough, metal, normal, yaw, pitch)
                    })
                }
                None => frame.shaded(
                    triangle.points,
                    triangle.normals,
                    [[0.0; 2]; 3],
                    |normal, _| super::kernel::lit(base, rough, metal, normal, yaw, pitch),
                ),
            }
        }
    }
    let frame = frame.downsample(scale);
    let Some(buffer) = image::RgbaImage::from_raw(
        frame.width as u32,
        frame.height as u32,
        frame.color,
    ) else {
        return;
    };
    let image = Arc::new(RenderImage::new(smallvec::smallvec![image::Frame::new(buffer)]));
    let _ = window.paint_image(bounds, bounds, corners, Arc::clone(&image), 0, false);
    release_shown(window);
    SHOWN.with(|s| *s.borrow_mut() = Some(image));
}

/// A buffer sized to the canvas, supersampled as far as the pixel budget
/// allows, with the factor it settled on.
///
/// The factor used to be computed and then thrown away — the buffer was built
/// at `SUPERSAMPLE` regardless, so a canvas over the budget drew *nothing*
/// instead of dropping to 1x. Returning the factor is what makes the fallback
/// a fallback.
fn frame_for(bounds: Bounds<Pixels>) -> Option<(BFrame, usize)> {
    let width = f32::from(bounds.size.width).floor().max(1.0) as usize;
    let height = f32::from(bounds.size.height).floor().max(1.0) as usize;
    let mut scale = SUPERSAMPLE;
    while scale > 1 && width * height * scale * scale > MAX_PIXELS {
        scale -= 1;
    }
    let (w, h) = (width * scale, height * scale);
    if w == 0 || h == 0 || w * h > MAX_PIXELS {
        return None;
    }
    Some((BFrame::new(w, h), scale))
}

/// Wireframe shading (`V3D_SHADING_WIREFRAME`) strokes the edges.
fn strokes(
    bounds: Bounds<Pixels>,
    meshes: &[BMesh],
    outline_width: f32,
    yaw: f32,
    pitch: f32,
    window: &mut Window,
) {
    for mesh in meshes {
        for triangle in &mesh.triangles {
            let mut builder = PathBuilder::stroke(px(outline_width * 0.5));
            let flat = triangle.points.map(|(x, y, _)| (x, y));
            builder.move_to(project(bounds, flat[0]));
            builder.line_to(project(bounds, flat[1]));
            builder.line_to(project(bounds, flat[2]));
            builder.close();
            if let Ok(path) = builder.build() {
                let lit = super::kernel::lit(
                    triangle.base,
                    triangle.roughness,
                    triangle.metallic,
                    triangle.normals[0],
                    yaw,
                    pitch,
                );
                window.paint_path(path, lit);
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
