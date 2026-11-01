# appkit: what every app stands on

Phiber is an app. A phiecs robotics app is another. Neither carries a GPU or a
UI kit of its own; they call this.

```
phiber   phiecs robotics   ...        apps
   \          |            /
             appkit                   the common layer
   gpu/       gpui and its platform renderers (DirectX, Metal, wgpu, web)
   pui/       declarative widgets
   src/glass  the glass material
   src/       the AppKit views, controls and panels
```

`gpu/` is built by the phikit workspace today. `src/` and `pui/` were
brought from `phi/crates` on winit 0.29 and wgpu 0.19, and move to the
workspace's stack as they are ported; they are not members until then.

# Rust Windows AppKit

A native Windows UI framework that mirrors the macOS AppKit API, built on top of `winit` for windowing and `softbuffer` for software rendering.

## Overview

Windows AppKit provides a **high-fidelity port** of Apple's macOS AppKit API for building native Windows applications. Every module directly mirrors Apple's headers from `AppKit.framework`.

### Module Documentation

| Module | Apple Header | Description |
|--------|--------------|-------------|
| **Base Classes** |||
| [NSResponder](src/responder/README.md) | `NSResponder.h` | Event handling chain |
| [NSView](src/view/README.md) | `NSView.h` | Base view class, hierarchy, drawing |
| [NSControl](src/control/README.md) | `NSControl.h` | Interactive control base |
| **Controls** |||
| [NSButton](src/button/README.md) | `NSButton.h` | Push buttons, checkboxes, radios |
| [NSTextField](src/text_field/README.md) | `NSTextField.h` | Text input and labels |
| [NSSlider](src/slider/README.md) | `NSSlider.h` | Value selection sliders |
| [NSSwitch](src/switch/README.md) | `NSSwitch.h` | Modern toggle switches |
| **Containers** |||
| [NSBox](src/box_view/README.md) | `NSBox.h` | Containers, separators, groups |
| [NSImageView](src/image_view/README.md) | `NSImage.h` | Image display, scaling |
| [NSScrollView](src/scroll_view/README.md) | `NSScrollView.h` | Scrollable content |
| [NSCollectionView](src/collection_view/README.md) | `NSCollectionView.h` | Grid of items |
| [NSOutlineView](src/outline_view/README.md) | `NSOutlineView.h` | Hierarchical lists |
| [NSStackView](src/stack_view/README.md) | `NSStackView.h` | Automatic layout |
| [NSSplitView](src/split_view/README.md) | `NSSplitView.h` | Divided panes |
| **Application** |||
| [NSMenu](src/menu/README.md) | `NSMenu.h` | Application menus |
| [NSToolbar](src/toolbar/README.md) | `NSToolbar.h` | Window toolbars |
| [NSAlert](src/alert/README.md) | `NSAlert.h` | Modal dialogs |
| [NSWindow](src/window/README.md) | `NSWindow.h` | Top-level windows |
| **App System** |||
| [NSApplication](src/application/README.md) | `NSApplication.h` | App lifecycle |
| [NSAppearance](src/appearance/README.md) | `NSAppearance.h` | Visual styles |
| [NSEvent](src/event/README.md) | `NSEvent.h` | Input events |
| [NSAnimation](src/animation/README.md) | `NSAnimation.h` | Property animation |
| **Specialized** |||
| [NSTabView](src/tab_view/README.md) | `NSTabView.h` | Tabbed interface |
| [NSSearchField](src/search_field/README.md) | `NSSearchField.h` | Search input |
| [NSSegmentedControl](src/segmented_control/README.md) | `NSSegmentedControl.h` | Mode selector |
| [NSTimelineView](src/timeline/README.md) | Custom | Animation timeline |
| **Foundation** |||
| [Graphics](src/graphics/README.md) | `NSColor.h`, `NSGeometry.h` | Colors, rects, images |
| [Math](src/math/README.md) | SceneKit | Vectors, matrices, quaternions |
| **Infrastructure** |||
| [RenderContext](src/render/README.md) | Custom | 2D software rendering |
| [GPU](src/gpu/README.md) | wgpu | Hardware acceleration |
| [Shaders](src/shaders/README.md) | WGSL | GPU shader programs |
| [Theme System](src/theme/README.md) | Custom | Visual styling |
| [ECS](src/ecs/README.md) | Custom | Entity Component System |
| [ShaderGraphView](src/shader_graph_view/README.md) | Custom | Visual shader editor |

```rust
use rust_appkit::*;

fn main() {
    // Create window with current Windows theme
    let mut window = NSWindow::standard("My App", 800.0, 600.0);
    
    // Build UI
    let mut stack = NSStackView::vertical();
    
    let button = NSButton::push_button("Click Me");
    stack.add_arranged_subview(button.as_view().clone());
    
    let text = NSTextField::new(NSRect::new(0.0, 0.0, 200.0, 22.0));
    stack.add_arranged_subview(text.as_view().clone());
    
    window.set_content_view(stack.view().clone());
    window.make_key_and_order_front();
}
```

## Features

### Core Controls
- **NSButton** - Push buttons, checkboxes, radio buttons
- **NSTextField** - Text input and labels
- **NSSlider** - Horizontal and vertical sliders
- **NSStackView** - Automatic layout container
- **NSScrollView** - Scrollable content
- **NSSplitView** - Resizable split panes

### Window Management
- **NSWindow** - Window with full styling
- **NSApplication** - Application lifecycle
- **NSView** - Base view class with hierarchy
- **NSControl** - Base control with actions

### Themes (Dynamic Theme System)

**New:** Data-driven themes with constant-based keys. No hardcoded theme names.

#### Theme Registry (Recommended)
```rust
use rust_appkit::*;

let mut registry = ThemeRegistry::new();
registry.set_current("win11_dark");

// Access colors by key (not hardcoded fields)
let bg = registry.color(keys::CLR_BG_WINDOW);
let accent = registry.color(keys::CLR_ACCENT);
let text = registry.color(keys::CLR_TEXT_LABEL);
```

#### Theme Key Constants
- **OS**: `OS_WINDOWS`, `OS_MACOS`, `OS_CUSTOM`
- **Versions**: `WIN_11`, `WIN_XP`, `WIN_7`, `MAC_AQUA`, `MAC_VENTURA`
- **Materials**: `MAT_ACRYLIC`, `MAT_LUNA`, `MAT_GLASS`, `MAT_WINAMP`
- **Variants**: `VAR_LIGHT`, `VAR_DARK`
- **Colors**: `CLR_BG_WINDOW`, `CLR_ACCENT`, `CLR_TEXT_LABEL`, etc.

#### Create Custom Themes
```rust
let custom = ThemeData {
    id: ThemeId::new(keys::OS_CUSTOM, "v1", keys::MAT_RETRO, keys::VAR_DARK),
    colors: {
        let mut c = standard_colors();
        c.insert(keys::CLR_BG_WINDOW.to_string(), NSColor::new(0.1, 0.1, 0.2, 1.0));
        c.insert(keys::CLR_ACCENT.to_string(), NSColor::new(1.0, 0.5, 0.0, 1.0));
        c
    },
    fonts: ThemeFonts { ... },
    supports_transparency: true,
    supports_animations: true,
};
registry.register("my_theme", custom);
```

#### Legacy ThemeManager (Backward Compatible)
```rust
let theme = WindowsTheme::current(); // Auto-detects OS
let manager = ThemeManager::new();
let def = manager.current_definition();
```

### Predefined Themes
| Key | OS | Version | Material | Variant |
|-----|-----|---------|----------|---------|
| `win11_light` | windows | 11 | acrylic | light |
| `win11_dark` | windows | 11 | acrylic | dark |
| `winxp_light` | windows | xp | luna | light |
| `macos_aqua` | macos | aqua | glass | light |
| `macos_ventura` | macos | ventura | vibrancy | light |
| `winamp_classic` | custom | classic | winamp | light |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-appkit = { path = "path/to/rust-appkit" }
```

Or use from the workspace:

```toml
[dependencies]
rust-appkit = { path = "phi/crates/rust-appkit" }
```

## Quick Start

### Basic Window
```rust
use rust_appkit::*;

fn main() {
    let app = NSApplication::shared();
    
    let mut window = NSWindow::standard("Hello", 400.0, 300.0);
    window.center();
    window.make_key_and_order_front();
    
    // Run event loop
    app.lock().unwrap().run().unwrap();
}
```

### Button with Action
```rust
let mut button = NSButton::push_button("Click Me");
button.set_action(|| {
    println!("Button clicked!");
});
```

### Checkbox
```rust
let checkbox = NSButton::checkbox("Enable Feature");
// checkbox.state() returns 0 (off) or 1 (on)
```

### Text Field
```rust
let mut text = NSTextField::new(NSRect::new(0.0, 0.0, 200.0, 22.0));
text.set_placeholder("Enter text...");
text.set_editable(true);
```

### Stack Layout
```rust
let mut stack = NSStackView::vertical();
stack.set_spacing(10.0);
stack.set_edge_insets(NSEdgeInsets::uniform(20.0));

stack.add_arranged_subview(button.as_view().clone());
stack.add_arranged_subview(text.as_view().clone());
```

### Themed Colors
```rust
let theme = ThemeManager::new();
let def = theme.current_definition();

// Use theme colors
view.set_background_color(Some(def.colors.window_background));
text.set_text_color(Some(def.colors.text_primary));
```

## API Reference

### Views & Controls

#### NSView
Base class for all UI elements.
```rust
let view = NSView::new(NSRect::new(x, y, width, height));
view.set_background_color(Some(NSColor::white()));
view.add_subview(child_view);
```

#### NSButton
```rust
// Push button
let button = NSButton::push_button("OK");

// Checkbox
let checkbox = NSButton::checkbox("Check me");

// Radio button
let radio = NSButton::radio_button("Option 1");

// Actions
button.set_action(|| println!("Clicked!"));
```

#### NSTextField
```rust
// Input field
let mut text = NSTextField::new(frame);
text.set_editable(true);
text.set_placeholder("Type here...");

// Label
let label = NSTextField::label("Hello");

// Get/set text
let value = text.text();
text.set_text("New value");
```

#### NSSlider
```rust
// Horizontal slider (0-100)
let slider = NSSlider::horizontal(0.0, 100.0, 50.0);

// Vertical slider
let slider = NSSlider::vertical(0.0, 100.0, 50.0);

// Get value
let value = slider.value();
```

#### NSStackView
```rust
// Vertical stack
let mut vstack = NSStackView::vertical();

// Horizontal stack
let mut hstack = NSStackView::horizontal();

// Configuration
vstack.set_spacing(8.0);
vstack.set_distribution(NSStackViewDistribution::FillEqually);
vstack.add_arranged_subview(view);
```

### Windows

#### NSWindow
```rust
// Standard window
let mut window = NSWindow::standard("Title", 800.0, 600.0);

// Custom window
let mut window = NSWindow::new(frame, &[NSWindowStyle::Titled, NSWindowStyle::Resizable]);

// Show window
window.make_key_and_order_front();
window.center();
```

### Themes

#### ThemeManager
```rust
let manager = ThemeManager::new();

// Get current theme
let current = manager.current_theme();

// Switch theme
manager.set_theme(WindowsTheme::WindowsXP);

// Get theme definition
let def = manager.current_definition();
println!("Font: {} {}pt", def.fonts.ui_font, def.fonts.ui_font_size);
```

#### ThemeColors
```rust
let colors = theme.colors;
let bg = colors.window_background;
let fg = colors.text_primary;
let accent = colors.accent;
```

#### ThemeMetrics
```rust
let metrics = theme.metrics;
let button_height = metrics.button_height;      // 32.0 (Win11) vs 23.0 (XP)
let button_radius = metrics.button_corner_radius; // 4.0 (Win11) vs 2.0 (XP)
```

## Architecture

### Component Hierarchy (Apple Parity)

```
AppKit Framework Hierarchy
├── NSResponder (event handling)
│   └── NSView (visual elements)
│       ├── NSControl (interactive)
│       │   ├── NSButton (+checkbox, +radio)
│       │   ├── NSTextField
│       │   └── NSSlider
│       ├── NSBox (containers)
│       ├── NSImageView
│       ├── NSStackView (layout)
│       └── NSSplitView (panes)
│   └── NSWindow (top-level)
└── NSApplication (app lifecycle)
```

### System Stack

```
┌─────────────────────────────────────────────┐
│              Your Application               │
│         (rust-kit-pro, Botanist)            │
├─────────────────────────────────────────────┤
│  rust-appkit (AppKit parity layer)          │
│  ├── NSWindow, NSView, NSButton            │
│  ├── Themes (Win11, XP, macOS)              │
│  └── RenderContext (2D software)            │
├─────────────────────────────────────────────┤
│  Platform Abstraction                       │
│  ├── winit (windowing)                      │
│  └── softbuffer (CPU rendering)             │
├─────────────────────────────────────────────┤
│  Windows OS / macOS / Linux                 │
└─────────────────────────────────────────────┘
```

## Design Principles

1. **AppKit Compatibility** - API matches macOS AppKit for easy porting
2. **Windows Native** - Uses winit for native windowing
3. **Theme Authenticity** - Each theme replicates its era's look
4. **Software Rendering** - No GPU required, works everywhere
5. **Modular** - Use only what you need

## Comparison

### vs. Native Win32
- **Pros:** Cleaner API, cross-platform potential, Rust-native
- **Cons:** Software rendering (for now), less mature

### vs. egui
- **Pros:** AppKit-style API, multiple themes, window-based
- **Cons:** Immediate mode vs retained mode (different paradigms)

### vs. Iced
- **Pros:** Lower-level control, familiar AppKit API
- **Cons:** Less batteries-included

## Development Status

**Current:** v0.1.0 - Core framework complete

**Working:**
- ✅ All core controls
- ✅ Theme system with 6 Windows versions
- ✅ Window management structure
- ✅ Event handling framework

**In Progress:**
- 🚧 Full winit integration
- 🚧 Software rendering implementation
- 🚧 Text rendering

## Examples

See `src/main.rs` for a comprehensive demo showing:
- All 6 Windows themes
- All UI controls
- Theme switching
- Color and font demonstration

Run with:
```bash
cargo run -p rust-appkit
```

## Testing

```bash
# Run tests
cargo test -p rust-appkit

# Check compilation
cargo check -p rust-appkit

# Run demo
cargo run -p rust-appkit
```

## Documentation

- `doc/flow.md` - Architecture and development flow
- `doc/task.md` - Roadmap and task list
- `src/main.rs` - Working examples

## License

Part of the Phi project. See parent repository for license.

## Contributing

This is an internal Phi project crate. For the main project, see the parent repository.

---

**Note:** This crate is currently in development. The API may change until v1.0.0.
