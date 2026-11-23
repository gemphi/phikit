# RenderContext — 2D Drawing Engine

## Overview

`RenderContext` provides 2D software rendering for views. Uses `softbuffer` for pixel output. Core rendering system for rust-appkit.

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 RenderContext Setup                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Drawing Commands                                │
│  ├─ clear(): Fill with background color                     │
│  ├─ fill_rect(): Solid rectangle                            │
│  ├─ stroke_rect(): Rectangle outline                        │
│  ├─ draw_line(): Single line segment                        │
│  ├─ draw_text(): Render text string                         │
│  └─ draw_image(): Blit image data                           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Present to Screen                          │
│  ├─ Flush pixel buffer to window surface                    │
│  └─ Double buffering prevents flicker                       │
└─────────────────────────────────────────────────────────────┘
```

## Coordinate System

```
Window/Buffer Coordinates
┌─────────────────────────────────────┐
│ 0,0                                │
│  ↓ +Y                               │
│                                     │
│  → +X                               │
│                                     │
│                max_x,max_y          │
└─────────────────────────────────────┘

Origin (0,0) at top-left, Y increases downward
```

## Connection to Other Modules

```
RenderContext
├── Used by: NSView.drawRect() (all views)
├── Used by: NSWindow (compositing)
├── Uses: softbuffer (backend)
├── Uses: NSColor (stroke/fill colors)
└── Uses: NSImage (texture source)
```

## Usage Examples

### Basic Drawing
```rust
use rust_appkit::prelude::*;
use rust_appkit::render::RenderContext;

struct MyViewDelegate;

impl NSViewDelegate for MyViewDelegate {
    fn draw_rect(&mut self, view: &mut NSView, context: &mut RenderContext) {
        // Clear background
        context.clear(NSColor::white());
        
        // Fill rectangle
        context.set_fill_color(NSColor::blue());
        context.fill_rect(NSRect::new(10.0, 10.0, 100.0, 50.0));
        
        // Stroke outline
        context.set_stroke_color(NSColor::black());
        context.set_line_width(2.0);
        context.stroke_rect(NSRect::new(10.0, 10.0, 100.0, 50.0));
    }
}
```

### Drawing Text
```rust
context.set_fill_color(NSColor::black());
context.draw_text("Hello", NSPoint::new(20.0, 30.0), 16.0);
```

### Custom Button Rendering
```rust
fn draw_button(context: &mut RenderContext, frame: NSRect, title: &str, pressed: bool) {
    // Background
    let bg_color = if pressed {
        NSColor::new(0.0, 0.5, 1.0, 1.0)  // Darker when pressed
    } else {
        NSColor::new(0.2, 0.6, 1.0, 1.0)
    };
    
    context.set_fill_color(bg_color);
    context.fill_rect(frame);
    
    // Border
    context.set_stroke_color(NSColor::new(0.0, 0.4, 0.8, 1.0));
    context.set_line_width(1.0);
    context.stroke_rect(frame);
    
    // Text (centered)
    context.set_fill_color(NSColor::white());
    let text_x = frame.x + frame.width / 2.0 - (title.len() as f32 * 4.0);
    let text_y = frame.y + frame.height / 2.0 + 6.0;
    context.draw_text(title, NSPoint::new(text_x, text_y), 14.0);
}
```

## Rendering Pipeline

```
View requests display
        │
        ▼
RenderContext created for view
        │
        ▼
View.drawRect() called
        │
        ▼
Commands: clear, fill, stroke, text
        │
        ▼
Buffer flushed to window surface
        │
        ▼
Window presents to screen
```

## Testing & Verification

```rust
let mut context = RenderContext::new(100, 100);

// Clear test
context.clear(NSColor::white());
// All pixels should be white

// Fill rect test
context.set_fill_color(NSColor::red());
context.fill_rect(NSRect::new(10.0, 10.0, 20.0, 20.0));
// Pixels in 10,10 to 30,30 should be red
```

## File Structure

```
src/render/
├── mod.rs          # RenderContext
├── path.rs         # Path drawing (lines, curves)
└── text.rs         # Text rasterization
```

## Missing from RenderContext

- **Anti-aliasing**: Smooth edges
- **Gradients**: Linear/radial fills
- **Shadows**: Drop shadows
- **Clipping**: Clip to path
- **Transformations**: Rotate, scale
- **Offscreen buffers**: Render to texture

These are stubbed for current use cases.

## Related

- **softbuffer**: The underlying pixel buffer backend
- **NSView.render()**: Entry point that creates RenderContext
- **Theme system**: Provides colors for rendering
