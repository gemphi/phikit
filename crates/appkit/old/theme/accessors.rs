//! Public theme accessors and switching functions.
//!
//! This is the primary API surface for consumers:
//! - `theme::color(key)` — get a themed color
//! - `theme::f32_or(key, fallback)` — get a geometry value
//! - `theme::set_variant("tesla")` — switch variant layer
//! - `theme::set_mode("dark")` — switch mode layer
//! - `theme::set_material("glass")` — switch material layer
//! - `theme::resolve_system_color(...)` — map NSSystemColor → themed color

use crate::NSColor;
use super::{keys, registry};

// ---------------------------------------------------------------------------
// Primary accessors — the theme ALWAYS has defaults, so these never fail.
// ---------------------------------------------------------------------------

/// Get a themed color. Always returns a value — the registry guarantees
/// every token has a default for both light and dark modes.
pub fn color(key: &str) -> NSColor {
    registry::get_color(key).unwrap_or_else(|| {
        // If a key is truly unknown, fall back to label color
        // so it's at least visible on any background.
        eprintln!("[theme] WARNING: unknown color key '{}'", key);
        NSColor::system_label()
    })
}

/// Get a themed color with a fallback. Returns the themed color if the key
/// exists in the registry, otherwise returns the provided fallback.
pub fn color_or(key: &str, fallback: NSColor) -> NSColor {
    registry::get_color(key).unwrap_or(fallback)
}

/// Get a geometry/layout f32 value from the current theme.
/// Panics if the key doesn't exist — all keys are registered with defaults.
pub fn f32(key: &str) -> f32 {
    registry::get_registry()
        .lock()
        .unwrap()
        .current()
        .layout
        .geometry
        .get(key)
        .copied()
        .unwrap_or_else(|| panic!("[theme] missing geometry key '{}'", key))
}

/// Get a geometry/layout f32 value from the current theme, with explicit fallback.
/// Use `theme::f32()` when the key is guaranteed to exist (registered in registry).
/// Use this only for optional/experimental keys.
pub fn f32_or(key: &str, fallback: f32) -> f32 {
    registry::get_registry()
        .lock()
        .unwrap()
        .current()
        .layout
        .geometry
        .get(key)
        .copied()
        .unwrap_or(fallback)
}

/// Get a boolean flag from geometry (stored as f32: >= 0.5 = true).
pub fn bool_or(key: &str, fallback: bool) -> bool {
    registry::get_registry()
        .lock()
        .unwrap()
        .current()
        .layout
        .geometry
        .get(key)
        .copied()
        .map(|v| v >= 0.5)
        .unwrap_or(fallback)
}

// ---------------------------------------------------------------------------
// Theme switching
// ---------------------------------------------------------------------------

pub fn set_current_theme(key: &str) {
    println!("[theme::set_current_theme] '{}'", key);
    registry::get_registry().lock().unwrap().set_current(key);
    redraw_window();
}

/// Change the variant layer (e.g. "apple", "tesla", "barbie") — recomposes.
pub fn set_variant(variant: &str) {
    println!("[theme::set_variant] '{}'", variant);
    registry::get_registry().lock().unwrap().set_variant(variant);
    redraw_window();
}

/// Change the mode layer ("dark", "light", "ombre") — recomposes.
pub fn set_mode(mode: &str) {
    println!("[theme::set_mode] '{}'", mode);
    registry::get_registry().lock().unwrap().set_mode(mode);
    redraw_window();
}

/// Change the material layer (e.g. "glass", "metal", "onyx") — recomposes.
pub fn set_material(material: &str) {
    println!("[theme::set_material] '{}'", material);
    registry::get_registry()
        .lock()
        .unwrap()
        .set_material(material);
    redraw_window();
}

/// Change the corner radius style (e.g. "razor", "tight", "soft", "pill").
pub fn set_radius(radius: &str) {
    println!("[theme::set_radius] '{}'", radius);
    redraw_window();
}

/// Change the visual effects level (e.g. "none", "subtle", "elevated", "cinematic").
pub fn set_effect(effect: &str) {
    println!("[theme::set_effect] '{}'", effect);
    redraw_window();
}

/// Change the accent color (e.g. "blue", "purple", "pink", "red", "green").
pub fn set_accent(accent: &str) {
    println!("[theme::set_accent] '{}'", accent);
    redraw_window();
}

/// Change the tint vibrancy (e.g. "none", "vibrant_light", "vibrant_dark", "sidebar").
pub fn set_tint(tint: &str) {
    println!("[theme::set_tint] '{}'", tint);
    redraw_window();
}

/// Change the shadow style (e.g. "none", "subtle", "elevated", "cinematic").
pub fn set_shadow(shadow: &str) {
    println!("[theme::set_shadow] '{}'", shadow);
    redraw_window();
}

/// Change the gradient style (e.g. "flat", "warm", "cool", "neon", "duotone").
pub fn set_gradient(gradient: &str) {
    println!("[theme::set_gradient] '{}'", gradient);
    redraw_window();
}

/// Change the typography family (e.g. "system", "mono", "serif", "rounded").
pub fn set_typography(typography: &str) {
    println!("[theme::set_typography] '{}'", typography);
    redraw_window();
}

/// Change the layout style (e.g. "compact", "macOS", "modern", "minimal").
pub fn set_layout(layout: &str) {
    println!("[theme::set_layout] '{}'", layout);
    redraw_window();
}

/// Change the density (e.g. "compact", "standard", "relaxed", "spacious").
pub fn set_density(density: &str) {
    println!("[theme::set_density] '{}'", density);
    redraw_window();
}

// ---------------------------------------------------------------------------
// Internal: trigger window redraw after theme change
// ---------------------------------------------------------------------------

fn redraw_window() {
    // Invalidate the viewport pixel cache so the next draw reads fresh theme colors.
    // This also ensures glass effect panels re-render with updated theme tint colors.
    crate::gpu::viewport::invalidate_viewport_cache();
    if let Ok(mut app) = crate::application::NSApplication::shared().try_lock() {
        if let Some(win) = app.window_at_mut(0) {
            // Mark the entire content view tree as needing display so all panels
            // (including those with glass effects) redraw with fresh theme colors.
            if let Some(ref mut content) = win.content_view {
                content.set_needs_display();
            }
            win.draw();
        }
    }
}

// ---------------------------------------------------------------------------
// NSSystemColor → theme token resolution
// ---------------------------------------------------------------------------

/// Resolve an `NSSystemColor` enum to the current themed color value.
pub fn resolve_system_color(color: crate::graphics::NSSystemColor) -> NSColor {
    let key = match color {
        crate::graphics::NSSystemColor::SystemBlue => keys::CLR_ACCENT,
        crate::graphics::NSSystemColor::Label => keys::CLR_TEXT_LABEL,
        crate::graphics::NSSystemColor::SecondaryLabel => keys::CLR_TEXT_SECONDARY,
        crate::graphics::NSSystemColor::TertiaryLabel => keys::CLR_TEXT_TERTIARY,
        crate::graphics::NSSystemColor::QuaternaryLabel => keys::CLR_TEXT_DISABLED,
        crate::graphics::NSSystemColor::WindowBackgroundColor => keys::CLR_BG_WINDOW,
        crate::graphics::NSSystemColor::ControlBackgroundColor => keys::CLR_BG_CONTROL,
        crate::graphics::NSSystemColor::SelectedControlColor => keys::CLR_BG_SELECTED,
        crate::graphics::NSSystemColor::SeparatorColor => keys::CLR_SEPARATOR,
        crate::graphics::NSSystemColor::TrafficClose => keys::CLR_TRAFFIC_CLOSE,
        crate::graphics::NSSystemColor::TrafficMin => keys::CLR_TRAFFIC_MIN,
        crate::graphics::NSSystemColor::TrafficZoom => keys::CLR_TRAFFIC_ZOOM,
        _ => keys::CLR_TEXT_LABEL,
    };
    registry::get_color(key).unwrap_or(NSColor::white())
}
