//! What the Image editor paints — `draw_image_main`: the checkerboard
//! behind a picture shown with its alpha (`SI_USE_ALPHA`), the picture at
//! the view's rect in the chosen pass and display channels, and over it
//! `draw_render_info`'s bar: `ED_region_info_draw` with Blender's own text
//! (`render_internal.cc::make_renderinfo_string`, then the engine's
//! `Scene, ViewLayer | Sample n/N`). The UV editor paints its grid instead.

use super::{BChannels, BImageView, INFO_BAR_HEIGHT, UV_CELLS};
use crate::v2::part::image::header::set::{DEPTH, DIFFUSE_COLOR, NORMAL};
use crate::v2::scene::render_job::{self, BRenderShot};
use crate::v2::scene::render_status;
use crate::v2::theme::{BAnimColors, BCheckerColors, BColor};
use crate::v2::view::draw::color::to_hsla;
use gpui::{App, Bounds, Corners, Pixels, RenderImage, SharedString, TextRun, Window, fill, point, px, size};
use std::sync::Arc;

/// `ED_region_info_draw_multiline`: the text starts 0.6 widget units in.
const INFO_INSET: f32 = 12.0;
/// `UI_FSTYLE_WIDGET` points at scale 1.
const FONT: f32 = 11.0;
/// `imm_draw_box_checker_2d`: the checker's square.
const CHECKER: f32 = 8.0;
/// `draw_image_grid`: the UV grid's lines are the one grid line every canvas
/// draws (`BAnimColors::grid_line`), a grid stroke wide.
const UV_GRID_LINE: f32 = 1.0;

thread_local! {
    /// The picture painted last, keyed by the shot it came from and the
    /// pass and channels it was built for. GPUI keeps every `RenderImage`
    /// until told otherwise, so the previous one goes as the next comes.
    static SHOWN: std::cell::RefCell<Option<((u32, String, BChannels), Arc<RenderImage>)>> = const { std::cell::RefCell::new(None) };
}

fn byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// A depth the Depth pass writes where a ray met nothing (`BStill::depth`).
const NO_HIT: f32 = 1e9;
/// The byte a data pass shows where the ray met nothing: a mid grey, so the
/// background reads as the editor's own field rather than as data, and
/// clear, so with Color & Alpha the checker shows through it as it does
/// behind a transparent Combined.
const MISS: [u8; 4] = [128, 128, 128, 0];

/// The shot's pixels as the pass and channels ask for them, BGRA rows.
///
/// Combined is the bytes as rendered. The data passes are read for the eye,
/// not raw: Blender shows a raw Depth clipped to white past one unit, which
/// is every pixel of a scene more than a metre from the camera, so Depth is
/// spread from the nearest hit (black) to the farthest (white), as
/// `draw_image_buffer`'s z-buffer path scaled it by the camera's clip range;
/// Normal and Diffuse Color are as rendered where a ray hit and `MISS`
/// where it did not, since the buffers hold whatever the kernel left there
/// behind the scene — deliberate, recorded.
pub fn pixels(shot: &BRenderShot, pass: &str, channels: BChannels) -> Vec<u8> {
    let count = (shot.w * shot.h) as usize;
    let missed = |i: usize| shot.depth.get(i).is_some_and(|d| *d >= NO_HIT);
    let rgb = |values: &[[f32; 3]]| -> Vec<u8> {
        (0..count)
            .flat_map(|i| {
                if missed(i) {
                    return MISS;
                }
                let c = values.get(i).copied().unwrap_or([0.0; 3]);
                [byte(c[2]), byte(c[1]), byte(c[0]), 255]
            })
            .collect()
    };
    let mut out = match pass {
        DEPTH if !shot.depth.is_empty() => {
            let hits = shot.depth.iter().copied().filter(|d| *d < NO_HIT);
            let (near, far) = hits.fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), d| (lo.min(d), hi.max(d)));
            let span = (far - near).max(1e-6);
            (0..count)
                .flat_map(|i| {
                    if missed(i) {
                        return MISS;
                    }
                    let v = byte((shot.depth[i] - near) / span);
                    [v, v, v, 255]
                })
                .collect()
        }
        NORMAL if !shot.normal.is_empty() => rgb(&shot.normal),
        DIFFUSE_COLOR if !shot.albedo.is_empty() => rgb(&shot.albedo),
        _ => shot.bgra.clone(),
    };
    for p in out.chunks_exact_mut(4) {
        let (b, g, r, a) = (p[0], p[1], p[2], p[3]);
        p.copy_from_slice(&match channels {
            BChannels::ColorAlpha => [b, g, r, a],
            BChannels::Color => [b, g, r, 255],
            BChannels::Alpha => [a, a, a, 255],
            BChannels::Red => [r, r, r, 255],
            BChannels::Green => [g, g, g, 255],
            BChannels::Blue => [b, b, b, 255],
        });
    }
    out
}

/// `imm_draw_box_checker_2d` behind the picture, only when its alpha shows.
fn paint_checker(at: Bounds<Pixels>, checker: BCheckerColors, window: &mut Window) {
    let (w, h) = (f32::from(at.size.width), f32::from(at.size.height));
    let (cols, rows) = ((w / CHECKER).ceil() as i32, (h / CHECKER).ceil() as i32);
    for row in 0..rows {
        for col in 0..cols {
            let color = if (row + col) % 2 == 0 { checker.light } else { checker.dark };
            let x = at.origin.x + px(col as f32 * CHECKER);
            let y = at.origin.y + px(row as f32 * CHECKER);
            let cell = Bounds::new(point(x, y), size(px(CHECKER).min(at.origin.x + at.size.width - x), px(CHECKER).min(at.origin.y + at.size.height - y)));
            window.paint_quad(fill(cell, to_hsla(color)));
        }
    }
}

/// The painters hang off the view (`tests/a_renderer_hangs_off_its_type.rs`):
/// what a view draws is reached through it, not from anywhere.
impl BImageView {
    /// The picture for the view's pass and channels, built when the image or
    /// the choice changes.
    fn render_image(&self, window: &mut Window) -> Option<Arc<RenderImage>> {
            let view = self;
        let key = (view.shown.seq(), format!("{}/{}", view.shown.name(), view.pass), view.channels);
        SHOWN.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.as_ref().is_none_or(|(k, _)| *k != key) {
                // A named image is its own picture; the render carries passes.
                let shot = match &view.shown {
                    crate::v2::part::image::shown::BShownImage::RenderResult => render_job::shot().0.filter(BRenderShot::has_image)?,
                    named => named.picture().map(|p| BRenderShot { w: p.w, h: p.h, bgra: p.bgra, done: true, ..Default::default() })?,
                };
                let buffer = image::RgbaImage::from_raw(shot.w, shot.h, pixels(&shot, &view.pass, view.channels))?;
                let made = Arc::new(RenderImage::new(smallvec::smallvec![image::Frame::new(buffer)]));
                if let Some((_, old)) = slot.replace((key, made)) {
                    let _ = window.drop_image(old);
                }
            }
            slot.as_ref().map(|(_, i)| i.clone())
        })
    }

    /// The picture in the window `at`, at the view's zoom and offset.
    pub fn paint_image(&self, checker: BCheckerColors, at: Bounds<Pixels>, window: &mut Window) {
            let view = self;
        let Some((w, h)) = view.image() else { return };
        let rect = view.rect(w as f32, h as f32, at);
        if view.channels == BChannels::ColorAlpha {
            paint_checker(rect, checker, window);
        }
        if let Some(image) = view.render_image(window) {
            let _ = window.paint_image(rect, rect, Corners::default(), image, 0, false);
        }
    }

    /// `make_renderinfo_string` then the engine's status: `Frame:1 | Time:00:12.34
    /// | Mem:12M, Peak: 12M | Scene, ViewLayer | Sample 12/128`, `Finished` once
    /// the render is done.
    pub fn info_text(&self, scene: &(String, String), frame: &str) -> Option<String> {
            let view = self;
        let status = view.status?;
        let time = render_status::timecode(render_status::elapsed());
        // The tail is the phase's own line, so the bar cannot disagree with
        // the canvas or the header about what the screen is showing
        // (`scene::render_phase` lists the eight and what each is grounded
        // in). It read `(done, paused())` here and had three answers for
        // eight states.
        let progress = view.phase.info(status.taken, status.target);
        Some(format!("Frame:{frame} | Time:{time} | Mem:{m}M, Peak: {m}M | {}, {} | {progress}", scene.0, scene.1, m = status.megabytes()))
    }

    /// `draw_render_info`: the bar across the top of the window.
    ///
    /// `clear` is how far in from each edge the text must start to clear
    /// the floating panes — left and right, measured from the panes
    /// themselves (`pane::floating_width`), not from the minimum-width
    /// tokens. The token says 160; an open Sidebar is over 650, which is
    /// what put the right-hand group off the end of the bar.
    ///
    /// The older note follows: how far in the text must start to clear the floating tool
    /// strip. Blender draws this into the image *region*, which does not
    /// include the Toolbar; phiber floats the Toolbar over the canvas, so
    /// without the inset the strip is painted over the front of the line and
    /// the render window read `1 | Time:…` with `Frame:` hidden behind it.
    pub fn paint_info(&self, scene: &(String, String), frame: &str, text: BColor, dim: BColor, at: Bounds<Pixels>, clear: (f32, f32), window: &mut Window, cx: &mut App) {
            let view = self;
        let Some(line) = view.info_text(scene, frame) else { return };
        let bar = Bounds::new(at.origin, size(at.size.width, px(INFO_BAR_HEIGHT)));
        // `draw_render_info` lays a quarter-black over the picture; here that
        // scrim is the theme's dim, the same one a modal lays over the
        // workspace, so it follows the palette instead of being black whatever
        // the theme.
        window.paint_quad(fill(bar, to_hsla(dim)));
        let run = TextRun { len: line.len(), font: window.text_style().font(), color: to_hsla(text), background_color: None, underline: None, strikethrough: None };
        let shaped = window.text_system().shape_line(SharedString::from(line), px(FONT), &[run], None);
        let line_height = px(FONT * 1.3);
        let origin = point(at.origin.x + px(INFO_INSET + clear.0), at.origin.y + px(INFO_BAR_HEIGHT / 2.0) - line_height / 2.0);
        let _ = shaped.paint(origin, line_height, gpui::TextAlign::Left, None, window, cx);
    }

    /// `draw_image_grid`: the UV editor's 0..1 square, eight cells a side, at
    /// the view's zoom, centred as an image would be.
    pub fn paint_uv_grid(&self, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window) {
            let view = self;
        let rect = view.uv_square(at);
        let side = f32::from(rect.size.width);
        window.paint_quad(fill(rect, to_hsla(colors.grid)));
        let line = to_hsla(colors.grid_line);
        let step = side / UV_CELLS as f32;
        for i in 0..=UV_CELLS {
            let offset = px((i as f32 * step).round());
            window.paint_quad(fill(Bounds::new(point(rect.origin.x + offset, rect.origin.y), size(px(UV_GRID_LINE), rect.size.height)), line));
            window.paint_quad(fill(Bounds::new(point(rect.origin.x, rect.origin.y + offset), size(rect.size.width, px(UV_GRID_LINE))), line));
        }
    }
}

#[cfg(test)]
#[path = "set_tests.rs"]
mod tests;
