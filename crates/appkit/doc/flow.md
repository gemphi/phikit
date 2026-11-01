# Windows AppKit Development Flow

## Overview
This document describes the development flow for the Windows AppKit project.

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Windows AppKit Architecture                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                    │
│  │   winit      │───→│   NSWindow   │───→│   NSView     │                    │
│  │  (native)    │    │  (window)    │    │  hierarchy   │                    │
│  └──────────────┘    └──────────────┘    └──────────────┘                    │
│         │                   │                   │                             │
│         ↓                   ↓                   ↓                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                    │
│  │   Events     │───→│ Hit Testing  │───→│   Actions    │                    │
│  │ (mouse/key)  │    │              │    │ (callbacks)  │                    │
│  └──────────────┘    └──────────────┘    └──────────────┘                    │
│                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                          Themes Layer                                   ││
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        ││
│  │  │ Win11    │ │ Win10    │ │ Win7     │ │ WinXP    │ │ Win9x    │        ││
│  │  │ (Fluent) │ │ (Metro)  │ │ (Aero)   │ │ (Luna)   │ │ (Classic)│        ││
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘        ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Flow

### Window Creation Flow
1. Create `NSWindow` with frame and style
2. Set `content_view` to root `NSView` (usually `NSStackView`)
3. Add child views to content view
4. Call `make_key_and_order_front()` to display

### Event Handling Flow
1. winit receives native window event
2. `NSApplication::handle_event()` processes event
3. Event passed to `NSWindow::handle_mouse_event()`
4. Hit-testing finds target `NSView`
5. View processes event (e.g., `mouse_down`, `mouse_up`)
6. Action callback invoked (e.g., button click)

### Rendering Flow
1. `NSApplication::run()` starts event loop
2. `update_windows()` called each frame
3. `NSWindow::draw()` prepares render context
4. `NSView::draw()` recursively renders hierarchy
5. `RenderContext::fill_rect()` draws to pixel buffer
6. Buffer presented to winit surface

## Theme System Flow (Dynamic/Data-Driven)

### New Constant-Based Theme System

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Dynamic Theme System Architecture                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐              │
│  │  Constant    │    │  ThemeId     │    │  ThemeData   │              │
│  │  Keys        │───→│  {os, ver,   │───→│  {colors:   │              │
│  │  OS_WINDOWS  │    │   material,  │    │   HashMap}   │              │
│  │  WIN_11      │    │   variant}   │    │              │              │
│  │  MAT_ACRYLIC │    │              │    │              │              │
│  │  CLR_ACCENT  │    │              │    │              │              │
│  └──────────────┘    └──────────────┘    └──────────────┘              │
│         │                   │                   │                       │
│         ↓                   ↓                   ↓                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                    ThemeRegistry                                   │ │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │ │
│  │  │win11_lt │ │win11_dk │ │winxp_lt │ │mac_aqua │ │winamp   │    │ │
│  │  │windows  │ │windows  │ │windows  │ │macos    │ │custom   │    │ │
│  │  │11       │ │11       │ │xp       │ │aqua     │ │winamp   │    │ │
│  │  │acrylic  │ │acrylic  │ │luna     │ │glass    │ │classic │    │ │
│  │  │light    │ │dark     │ │light    │ │light    │ │light    │    │ │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘    │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Theme Application Flow (New System)
1. Use `ThemeRegistry::new()` to create registry
2. Access colors by key: `registry.color(keys::CLR_BG_WINDOW)`
3. Switch themes: `registry.set_current("win11_dark")`
4. Create custom themes with `ThemeId::new(os, version, material, variant)`

### Theme Key Constants
- **OS**: `OS_WINDOWS`, `OS_MACOS`, `OS_CUSTOM`
- **Versions**: `WIN_11`, `WIN_XP`, `WIN_7`, `MAC_AQUA`, `MAC_VENTURA`
- **Materials**: `MAT_ACRYLIC`, `MAT_LUNA`, `MAT_GLASS`, `MAT_WINAMP`
- **Variants**: `VAR_LIGHT`, `VAR_DARK`, `VAR_HIGH_CONTRAST`
- **Colors**: `CLR_BG_WINDOW`, `CLR_ACCENT`, `CLR_TEXT_LABEL`, etc.

### Example Usage
```rust
use rust_appkit::*;

let mut registry = ThemeRegistry::new();
registry.set_current("win11_dark");

// Access colors by key (not hardcoded fields)
let bg = registry.color(keys::CLR_BG_WINDOW);
let accent = registry.color(keys::CLR_ACCENT);

// Create custom theme
let custom = ThemeData {
    id: ThemeId::new(keys::OS_CUSTOM, "v1", keys::MAT_RETRO, keys::VAR_DARK),
    colors: standard_colors(), // Populate with keys
    fonts: ThemeFonts { ... },
    supports_transparency: true,
    supports_animations: true,
};
registry.register("my_theme", custom);
```

## Data Flow

### State Changes
```
User Input → NSEvent → NSView → Action → State Update → Redraw
```

### Rendering Pipeline
```
NSWindow → RenderContext → Pixel Buffer → winit Surface → Screen
```

## Development Workflow

### 1. Adding New Component
1. Create module file (e.g., `new_component.rs`)
2. Define struct mirroring macOS AppKit API
3. Implement methods following Windows AppKit patterns
4. Add to `lib.rs` module list and re-exports
5. Add theme-aware styling
6. Write tests in `main.rs`
7. Update documentation

### 2. Adding New Theme
1. Add variant to `WindowsTheme` enum
2. Create `theme_x()` method in `theme.rs`
3. Define `ThemeColors`, `ThemeMetrics`, `ThemeFonts`
4. Update `ThemeManager::available_themes()`
5. Test with `cargo run`

### 3. Integration Steps
1. Export public API from `lib.rs`
2. Ensure winit compatibility
3. Test on target Windows versions
4. Update parent project integration

## File Organization Flow

```
rust-appkit/
├── src/
│   ├── lib.rs           → Public API exports
│   ├── main.rs          → Demo & tests
│   ├── graphics.rs      → NSColor, NSRect, NSPoint
│   ├── view.rs          → NSView base class
│   ├── control.rs       → NSControl base
│   ├── button.rs        → NSButton
│   ├── text_field.rs    → NSTextField
│   ├── slider.rs        → NSSlider
│   ├── stack_view.rs    → NSStackView
│   ├── scroll_view.rs   → NSScrollView
│   ├── split_view.rs    → NSSplitView
│   ├── window.rs        → NSWindow
│   ├── application.rs   → NSApplication
│   ├── event.rs         → NSEvent
│   ├── render.rs        → RenderContext
│   └── theme.rs         → Windows themes
├── doc/
│   ├── flow.md          → This file
│   ├── task.md          → Tasks & roadmap
│   └── readme.md        → User documentation
└── Cargo.toml
```

## Testing Flow

### Unit Tests (New Data-Driven System)
Located in `main.rs` under `#[cfg(test)]`:

**Theme Registry Tests:**
- `test_theme_registry()` - Registry creation and theme access by key
- `test_theme_keys()` - All constant keys are correct strings
- `test_custom_theme_creation()` - Creating themes with `ThemeId::new()`

**Legacy Compatibility Tests:**
- `test_legacy_theme_manager()` - Backward compatible `ThemeManager` still works
- `test_all_themes()` - All theme definitions valid

**Component Tests:**
- `test_colors()` - Color creation and HashMap access
- `test_window_creation()` - Window instantiation
- `test_button_creation()` - Button creation
- `test_stack_view()` - Layout container

### Running Tests
```bash
cd phi/crates/rust-appkit
cargo test
```

### Manual Testing
```bash
cargo run
```
- Verifies all themes load via registry
- Creates sample UI components
- Tests color access by key (not hardcoded fields)
- Demonstrates theme switching

### Integration Testing
- Build with `cargo check`
- Test theme registry: `registry.color(keys::CLR_ACCENT)`
- Link with parent project
- Verify winit integration

## Integration with Parent Project

The Windows AppKit is designed to be used by the main Phi project:

### Using the New Theme Registry (Recommended)
```rust
// In main Phi application
use rust_appkit::*;

fn create_ui() {
    // New data-driven approach with constant keys
    let mut registry = ThemeRegistry::new();
    registry.set_current("win11_light");
    
    // Access colors by key (not hardcoded fields)
    let bg_color = registry.color(keys::CLR_BG_WINDOW);
    let accent = registry.color(keys::CLR_ACCENT);
    let text = registry.color(keys::CLR_TEXT_LABEL);
    
    let window = NSWindow::standard("Phi Editor", 1280.0, 800.0);
    // ... build UI with theme-aware colors
}
```

### Legacy ThemeManager (Backward Compatible)
```rust
use rust_appkit::*;

fn legacy_ui() {
    // Old API still works
    let theme = ThemeManager::new();
    let def = theme.current_definition();
    let window = NSWindow::standard("Phi Editor", 1280.0, 800.0);
}
```

## Future Development Flow

### Phase 1: Core (Complete)
- ✓ Basic views and controls
- ✓ Window management
- ✓ Event handling
- ✓ Theme system

### Phase 2: Rendering (In Progress)
- Software rendering with softbuffer
- Text rendering with fontdue
- Image support

### Phase 3: Advanced Controls
- Menu system
- Table views
- Collection views
- Split view controller

### Phase 4: Polish
- Animation support
- Accessibility
- High DPI support
- Multiple monitor support
