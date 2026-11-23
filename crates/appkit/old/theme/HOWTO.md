# Theme — How to Use

## Files
- `mod.rs` — orchestrator + re-exports
- `accessors.rs` — public API: `color()`, `f32_or()`, `set_variant()`, etc.
- `keys.rs` — token constants (`CLR_BG_PANEL`, `RADIUS_BUTTON`, etc.)
- `registry.rs` — global singleton, composition engine
- `types.rs` — `ThemeData`, `ThemeFont`, `ThemeLayout`, `ThemeId`
- `styles.rs` — composable layers (Variant, Material, Typography)
- `appearance.rs` — Apple's NSAppearance types (mode, vibrancy)
- `variants/` — per-variant color/font overrides (apple, tesla, etc.)

## Quick Start
```rust
use rust_appkit::theme;

// Read tokens (the primary API)
let bg = theme::color(theme::keys::CLR_BG_PANEL);
let radius = theme::f32_or(theme::keys::RADIUS_PANEL, 12.0);
let font_size = theme::f32_or(theme::keys::SIZE_FONT_UI, 13.0);

// Switch layers at runtime
theme::set_variant("tesla");
theme::set_mode("dark");
theme::set_material("glass");

// Apply theme to a view tree
view.set_theme_token(theme::keys::CLR_BG_PANEL);
view.set_radius_token(theme::keys::RADIUS_PANEL);
view.apply_theme();
```

## Key APIs
- `theme::color(key)` — resolve a color token
- `theme::f32_or(key, default)` — resolve a float token
- `theme::bool_or(key, default)` — resolve a bool token
- `theme::set_variant(name)` — switch visual variant
- `theme::set_mode(name)` — switch light/dark mode
- `theme::set_material(name)` — switch material (glass, opaque)
- `view.set_theme_token(key)` — bind view background to token
- `view.apply_theme()` — recursively resolve all tokens

## Rules
- Never hardcode colors — always use `theme::color(key)`
- Geometry (radii, spacing) comes from `theme::f32_or(key, fallback)`
- Variants compose: variant + mode + material = final ThemeData
- `apply_theme()` walks the entire subview tree
- Custom variants go in `variants/` and implement `NSThemeVariant`
