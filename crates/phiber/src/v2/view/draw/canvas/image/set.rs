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

/// The shot's pixels as the pass and channels ask for them, BGRA rows.
pub fn pixels(shot: &BRenderShot, pass: &str, channels: BChannels) -> Vec<u8> {
    let count = (shot.w * shot.h) as usize;
    let grey = |values: &[f32]| -> Vec<u8> { (0..count).flat_map(|i| { let v = byte(values.get(i).copied().unwrap_or(0.0)); [v, v, v, 255] }).collect() };
    let rgb = |values: &[[f32; 3]]| -> Vec<u8> {
        (0..count).flat_map(|i| { let c = values.get(i).copied().unwrap_or([0.0; 3]); [byte(c[2]), byte(c[1]), byte(c[0]), 255] }).collect()
    };
    let mut out = match pass {
        DEPTH if !shot.depth.is_empty() => grey(&shot.depth),
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
        let progress = if status.done { "Finished".to_string() } else { format!("Sample {}/{}", status.taken, status.target) };
        Some(format!("Frame:{frame} | Time:{time} | Mem:{m}M, Peak: {m}M | {}, {} | {progress}", scene.0, scene.1, m = status.megabytes()))
    }

    /// `draw_render_info`: the bar across the top of the window.
    pub fn paint_info(&self, scene: &(String, String), frame: &str, text: BColor, dim: BColor, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
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
        let origin = point(at.origin.x + px(INFO_INSET), at.origin.y + px(INFO_BAR_HEIGHT / 2.0) - line_height / 2.0);
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
mod tests {
    use super::*;

    #[test]
    fn channels_and_passes_pick_the_bytes_blender_shows() {
        let shot = BRenderShot { w: 1, h: 1, bgra: vec![10, 20, 30, 128], depth: vec![0.5], normal: vec![[1.0, 0.0, -1.0]], albedo: vec![[0.0, 1.0, 0.0]], ..Default::default() };
        assert_eq!(pixels(&shot, "Combined", BChannels::Color), vec![10, 20, 30, 255]);
        assert_eq!(pixels(&shot, "Combined", BChannels::ColorAlpha), vec![10, 20, 30, 128]);
        assert_eq!(pixels(&shot, "Combined", BChannels::Alpha), vec![128, 128, 128, 255]);
        assert_eq!(pixels(&shot, "Combined", BChannels::Red), vec![30, 30, 30, 255]);
        assert_eq!(pixels(&shot, DEPTH, BChannels::Color), vec![128, 128, 128, 255], "depth is grey, raw and clipped");
        assert_eq!(pixels(&shot, NORMAL, BChannels::Color), vec![0, 0, 255, 255], "a negative component clips to black");
        assert_eq!(pixels(&shot, DIFFUSE_COLOR, BChannels::Green), vec![255, 255, 255, 255]);
    }

    #[test]
    fn the_info_bar_reads_as_blenders() {
        let mut view = BImageView { zoom: None, pan: (0.0, 0.0), channels: BChannels::Color, pass: "Combined".into(), slot_filled: true, status: None, shown: crate::v2::part::image::shown::BShownImage::RenderResult };
        assert_eq!(view.info_text(&("Scene".into(), "ViewLayer".into()), "1"), None, "no render, no bar");
        view.status = Some(render_status::BRenderStatus { w: 4, h: 4, taken: 3, target: 8, done: false, has_image: true, depth: false, normal: false, albedo: false, bytes: 64, seq: 1 });
        let text = view.info_text(&("Scene".into(), "ViewLayer".into()), "12").unwrap();
        assert!(text.starts_with("Frame:12 | Time:") && text.ends_with("| Mem:1M, Peak: 1M | Scene, ViewLayer | Sample 3/8"), "{text}");
        view.status.as_mut().unwrap().done = true;
        assert!(view.info_text(&("Scene".into(), "ViewLayer".into()), "12").unwrap().ends_with("| Finished"));
    }
}
