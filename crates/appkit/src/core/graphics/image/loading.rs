//! Image loading from file, SVG data, and PNG data.

use crate::graphics::NSSize;
use image::GenericImageView;
use std::sync::atomic::{AtomicU32, Ordering};
use usvg::{Options, TreeParsing};

use super::format_types::NSImage;
use super::processing;

// ─── Global Scale Factor ─────────────────────────────────────────────────────
// Stored as fixed-point (×100) so we can use AtomicU32.
// Default = 100 means 1.0× (the softbuffer framebuffer is already at physical resolution).
static SCALE_FACTOR_X100: AtomicU32 = AtomicU32::new(100);

/// Set the global display scale factor. Call this from the window event loop
/// when the window is created or when DPI changes (e.g. moved between monitors).
pub fn set_display_scale_factor(factor: f32) {
    let stored = (factor * 100.0).round() as u32;
    SCALE_FACTOR_X100.store(stored, Ordering::Relaxed);
}

/// Get the current display scale factor (defaults to 1.0 for standard displays).
pub fn display_scale_factor() -> f32 {
    SCALE_FACTOR_X100.load(Ordering::Relaxed) as f32 / 100.0
}

/// Creates an `NSImage` from raw SVG data, rasterised at `target_size × scale_factor`.
///
/// The image stores pixels at the physical resolution for crisp rendering on
/// HiDPI displays. SVGs that use `currentColor` are patched to render using
/// the active theme's `CLR_TEXT_LABEL` token.
pub(super) fn new_with_svg(name: &str, svg_data: &[u8], target_size: NSSize) -> Option<NSImage> {
    let scale = display_scale_factor();
    let physical_w = (target_size.width * scale).ceil();
    let physical_h = (target_size.height * scale).ceil();

    println!(
        "[NSImage] Rendering SVG icon: '{}' ({}x{} @{:.1}x → {}x{} physical)",
        name, target_size.width, target_size.height, scale, physical_w, physical_h
    );

    let patched = patch_svg(svg_data);
    let opt = Options::default();
    let parsed: Result<usvg::Tree, _> = TreeParsing::from_data(patched.as_bytes(), &opt);
    let tree: usvg::Tree = match parsed {
        Ok(mut t) => {
            t.calculate_bounding_boxes();
            t
        }
        Err(e) => {
            println!("[NSImage] Failed to parse SVG '{}': {:?}", name, e);
            let tint = crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL);
            let hex = tint_hex(&tint);
            let fallback = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\">\
                 <circle cx=\"50\" cy=\"50\" r=\"40\" fill=\"{}\" /></svg>",
                hex
            );
            let mut fallback_tree: usvg::Tree =
                TreeParsing::from_data(fallback.as_bytes(), &opt).ok()?;
            fallback_tree.calculate_bounding_boxes();
            fallback_tree
        }
    };

    let (tree_w, tree_h) = (tree.size.width(), tree.size.height());
    if tree_w == 0.0 || tree_h == 0.0 {
        println!(
            "[NSImage] SVG '{}' has zero dimension: {}x{}",
            name, tree_w, tree_h
        );
        return None;
    }

    if tree_w > 200.0 || tree_h > 200.0 {
        processing::render_spritesheet(&tree, physical_w, physical_h, name, target_size)
    } else {
        processing::render_normal(
            &tree,
            physical_w,
            physical_h,
            tree_w,
            tree_h,
            name,
            target_size,
        )
    }
}

/// Load a PNG from raw bytes.
pub(super) fn new_with_png(name: &str, png_data: &[u8]) -> Option<NSImage> {
    let img = image::load_from_memory(png_data).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = img.dimensions();
    let size = NSSize::new(w as f32, h as f32);

    Some(NSImage {
        name: name.to_string(),
        size,
        physical_size: size, // PNGs are already at their native resolution
        pixels: rgba.to_vec(),
    })
}

// ─── SVG Patching ────────────────────────────────────────────────────────────

fn tint_hex(tint: &crate::graphics::NSColor) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (tint.r * 255.0) as u8,
        (tint.g * 255.0) as u8,
        (tint.b * 255.0) as u8
    )
}

fn patch_svg(svg_data: &[u8]) -> String {
    let mut svg_string = String::from_utf8_lossy(svg_data).to_string();

    // Ensure XML namespace exists
    if !svg_string.contains("xmlns=") {
        svg_string = svg_string.replace("<svg", "<svg xmlns=\"http://www.w3.org/2000/svg\"");
    }

    let tint = crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL);
    let hex = tint_hex(&tint);

    // Strip <style> blocks
    while let Some(start) = svg_string.find("<style") {
        if let Some(end_offset) = svg_string[start..].find("</style>") {
            let end = start + end_offset + 8;
            svg_string.replace_range(start..end, "");
        } else {
            break;
        }
    }

    // Strip Inkscape/Sodipodi metadata
    while let Some(start) = svg_string.find("<sodipodi:") {
        if let Some(end_offset) = svg_string[start..].find(">") {
            let end = start + end_offset + 1;
            svg_string.replace_range(start..end, "");
        } else {
            break;
        }
    }

    svg_string
        .replace("currentColor", &hex)
        .replace(
            "class=\"blender-icon-primary\"",
            &format!("fill=\"{}\"", hex),
        )
        .replace(
            "class=\"blender-icon-secondary\"",
            &format!("fill=\"{}\" opacity=\"0.6\"", hex),
        )
        .replace("class=\"blender-icon-text\"", &format!("fill=\"{}\"", hex))
        .replace("fill=\"#cf0f96ff\"", &format!("fill=\"{}\"", hex))
        .replace("fill=\"#00ff4f\"", &format!("fill=\"{}\"", hex))
        .replace("fill=\"#f4ef4e\"", &format!("fill=\"{}\"", hex))
        .replace("fill=\"white\"", &format!("fill=\"{}\"", hex))
        .replace("fill=\"#fff\"", &format!("fill=\"{}\"", hex))
        .replace("fill=\"#ffffff\"", &format!("fill=\"{}\"", hex))
        .replace("enable-background=\"new\"", "")
}
