//! Icon caching — memoizes rasterized icons by (id, size) key.

use super::procedural::render_icon_procedural;
use super::svg_map::{icon_name, svg_data};
use super::super::IconId;
use crate::graphics::image::NSImage;
use std::sync::Arc;

/// Default icon size (toolbar)
const DEFAULT_SIZE: u32 = 20;

/// Get an icon as an NSImage at default size (20x20).
pub fn get(id: IconId) -> Option<Arc<NSImage>> {
    NSIconCache::get(id)
}

/// Get an icon as an NSImage at a custom size.
pub fn get_sized(id: IconId, width: f32, height: f32) -> Option<Arc<NSImage>> {
    NSIconCache::get_sized(id, width, height)
}

/// Get raw RGBA pixel data for an icon.
pub fn get_icon_pixels(id: &IconId, size: u32) -> (Vec<u8>, u32, u32) {
    NSIconCache::get_icon_pixels(id, size)
}

pub struct NSIconCache;

impl NSIconCache {

/// Get an icon as an NSImage at default size (20x20).
pub fn get(id: IconId) -> Option<Arc<NSImage>> {
    get_sized(id, DEFAULT_SIZE as f32, DEFAULT_SIZE as f32)
}

/// Get an icon as an NSImage at a custom size.
/// Rasterizes at 2x the requested dimensions for retina/HiDPI anti-aliased display.
/// Tries SVG file first, falls back to procedural rendering.
/// Results are cached — subsequent calls with the same id+size return instantly.
pub fn get_sized(id: IconId, width: f32, height: f32) -> Option<Arc<NSImage>> {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    // Global icon cache: (icon_name, width_bits, height_bits) → Arc<NSImage>
    static CACHE: OnceLock<Mutex<HashMap<(String, u32, u32), Arc<NSImage>>>> = OnceLock::new();

    let requested = width.max(height) as u32;
    if requested == 0 {
        return None;
    }

    let cache_key = (format!("{:?}", id), width.to_bits(), height.to_bits());

    // Check cache first
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(&cache_key) {
            return Some(cached.clone());
        }
    }

    // Rasterize at 2x for retina display quality
    let retina_width = width * 2.0;
    let retina_height = height * 2.0;

    // Try SVG-based rendering first
    let result = if let Some(svg_bytes) = svg_data(&id) {
        let name = icon_name(&id);
        if let Some(img) = NSImage::new_with_svg(
            name,
            svg_bytes,
            crate::graphics::NSSize::new(retina_width, retina_height),
        ) {
            Some(Arc::new(img))
        } else {
            None
        }
    } else {
        None
    };

    // Fallback to procedural rendering
    let result = result.or_else(|| {
        let render_size = (requested * 2).max(32);
        let pixels = render_icon_procedural(&id, render_size);
        Some(Arc::new(NSImage::from_rgba(
            &pixels,
            render_size,
            render_size,
        )))
    });

    // Store in cache
    if let Some(ref img) = result {
        if let Ok(mut guard) = cache.lock() {
            guard.insert(cache_key, img.clone());
        }
    }

    result
}

/// Get raw RGBA pixel data for an icon.
/// Returns pixels at 2x the requested size for retina display.
/// Results are cached for performance.
pub fn get_icon_pixels(id: &IconId, size: u32) -> (Vec<u8>, u32, u32) {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    static PIXEL_CACHE: OnceLock<Mutex<HashMap<(String, u32), (Vec<u8>, u32, u32)>>> =
        OnceLock::new();

    let cache_key = (format!("{:?}", id), size);
    let cache = PIXEL_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Check cache
    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(&cache_key) {
            return cached.clone();
        }
    }

    let retina_size = size * 2;
    let result = if let Some(svg_bytes) = svg_data(id) {
        let name = icon_name(id);
        if let Some(img) = NSImage::new_with_svg(
            name,
            svg_bytes,
            crate::graphics::NSSize::new(retina_size as f32, retina_size as f32),
        ) {
            (img.pixels().to_vec(), retina_size, retina_size)
        } else {
            let pixels = render_icon_procedural(id, retina_size);
            (pixels, retina_size, retina_size)
        }
    } else {
        let pixels = render_icon_procedural(id, retina_size);
        (pixels, retina_size, retina_size)
    };

    // Store in cache
    if let Ok(mut guard) = cache.lock() {
        guard.insert(cache_key, result.clone());
    }

    result
}

} // impl NSIconCache
