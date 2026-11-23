# Theme System — Visual Styling

## Apple Header: `NSAppearance.h`, `NSColor.h`

## Overview

`theme` module manages application appearance with **first-class support for both macOS and Windows** themes. Implements [`NSAppearance`](https://developer.apple.com/documentation/appkit/nsappearance) from AppKit.

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 Theme System                                   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Theme Data                                        │
│  ├─ ThemeId: OS, version, material, variant                 │
│  ├─ Colors: Background, accent, text, control               │
│  ├─ Fonts: UI font, size, weights                          │
│  └─ Metrics: Corner radius, padding, button height           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Theme Registry                                 │
│  ├─ Register: Add custom themes                              │
│  ├─ Set current: Apply theme to app                         │
│  └─ Access: Get colors/metrics for current                    │
└─────────────────────────────────────────────────────────────┘
```

## Built-in Themes

### macOS Themes (Primary - AppKit Native)

| Theme ID | Description | Features |
|----------|-------------|----------|
| `macos_aqua` | macOS Classic Aqua | Glossy buttons, pinstripe backgrounds |
| `macos_ventura` | macOS Ventura | Vibrancy, SF font, Big Sur styling |
| `macos_dark` | macOS Dark Mode | Dark Aqua, vibrancy support |

### Windows Themes

| Theme ID | Description | Features |
|----------|-------------|----------|
| `win11_light` | Windows 11 | Acrylic, Mica, rounded corners |
| `win11_dark` | Windows 11 Dark | Acrylic dark mode |
| `winxp_luna` | Windows XP Luna | Blue theme, classic Windows |

## Theme Keys

```rust
// OS
theme_keys::OS_WINDOWS
theme_keys::OS_MACOS
theme_keys::OS_CUSTOM

// Versions
theme_keys::WIN_11
theme_keys::WIN_XP
theme_keys::MAC_AQUA
theme_keys::MAC_VENTURA

// Materials
theme_keys::MAT_ACRYLIC
theme_keys::MAT_LUNA
theme_keys::MAT_GLASS
theme_keys::MAT_VIBRANCY

// Variants
theme_keys::VAR_LIGHT
theme_keys::VAR_DARK
```

## Connection to Other Modules

```
Theme System
├── Used by: All views (colors, metrics)
├── Used by: RenderContext (draw colors)
├── Used by: NSWindow (frame styling)
└── Integrates with: NSAppearance (system theme)
```

## Usage Examples

### Set Theme (macOS)
```rust
use rust_appkit::theme;

// macOS themes are first-class
theme::set_current_theme("macos_ventura");
// or
theme::set_current_theme("macos_dark");
```

### Set Theme (Windows)
```rust
// Windows themes also fully supported
theme::set_current_theme("win11_dark");
// or
theme::set_current_theme("win11_light");
```

### Access Colors (Actual API)
```rust
use rust_appkit::theme;

// Get color from current theme
let bg = theme::color(theme::keys::CLR_BG_WINDOW);
let accent = theme::color(theme::keys::CLR_ACCENT);
let text = theme::color(theme::keys::CLR_TEXT_LABEL);

// With fallback
let custom = theme::color_or("custom_key", NSColor::white());

// Get scalar values (stored in color channels)
let spacing = theme::f32_or(theme::keys::SPACING_MD, 8.0);
let flag = theme::bool_or(theme::keys::ENABLE_ANIMATION, true);
```

### Custom Theme
```rust
let custom = ThemeData {
    id: ThemeId::new(
        theme_keys::OS_CUSTOM,
        "v1",
        theme_keys::MAT_ACRYLIC,
        theme_keys::VAR_DARK
    ),
    colors: {
        let mut c = HashMap::new();
        c.insert(theme_keys::CLR_BG_WINDOW.to_string(), 
                 NSColor::new(0.1, 0.1, 0.15, 1.0));
        c.insert(theme_keys::CLR_ACCENT.to_string(), 
                 NSColor::new(0.0, 0.8, 1.0, 1.0));
        c
    },
    fonts: ThemeFonts::default(),
    metrics: ThemeMetrics::default(),
    supports_transparency: true,
};

registry.register("my_theme", custom);
registry.set_current("my_theme");
```

### View Theming
```rust
fn apply_theme_to_button(button: &mut NSButton, theme: &ThemeData) {
    let accent = theme.color(theme_keys::CLR_ACCENT);
    let bg = theme.color(theme_keys::CLR_BG_CONTROL);
    
    button.set_bezel_color(Some(accent));
    button.control.view_mut().set_background_color(Some(bg));
}
```

## Theme Structure

```
ThemeData
├── id: ThemeId
│   ├── os: OS_WINDOWS | OS_MACOS | OS_CUSTOM
│   ├── version: "11" | "xp" | "ventura"
│   ├── material: MAT_ACRYLIC | MAT_LUNA | MAT_GLASS
│   └── variant: VAR_LIGHT | VAR_DARK
├── colors: HashMap<String, NSColor>
│   ├── "CLR_BG_WINDOW" → NSColor
│   ├── "CLR_ACCENT" → NSColor
│   └── ...
├── fonts: ThemeFonts
│   ├── ui_font: "Segoe UI" | "San Francisco"
│   └── ui_font_size: 12.0 | 13.0
└── metrics: ThemeMetrics
    ├── button_height: 32.0 | 23.0
    ├── button_corner_radius: 4.0 | 2.0
    └── control_padding: 8.0
```

## Testing & Verification

```rust
let registry = ThemeRegistry::new();

// Test color API (actual functions from mod.rs)
let bg = theme::color(theme::keys::CLR_BG_WINDOW);
assert!(bg.alpha() == 1.0);

// Test with fallback
let custom = theme::color_or("unknown_key", NSColor::black());

// Test scalar access
let spacing = theme::f32_or(theme::keys::SPACING_MD, 8.0);
let enabled = theme::bool_or(theme::keys::ENABLE_ANIMATION, false);

// System color resolution
let system_blue = theme::resolve_system_color(NSSystemColor::SystemBlue);
```

## File Structure

```
src/theme/
├── mod.rs          # ThemeRegistry, ThemeData, ThemeId
├── keys.rs         # Theme key constants
├── colors.rs       # Color definitions
├── fonts.rs        # Font definitions
└── metrics.rs      # Spacing, sizing metrics
```

## Missing from Theme System

- **Live switching**: Update all views on change
- **Animation**: Smooth transitions between themes
- **System detection**: Auto-match OS theme
- **User customization**: Runtime color editing
- **CSS-like themes**: External theme files

These are stubbed for current use cases.

## Related

- **NSAppearance**: System appearance integration
- **Graphics module**: NSColor definitions
- **All views**: Consume theme for rendering
