# NSView — Base View Class

## Apple Header: `NSView.h`

## Overview

`NSView` is the fundamental building block of the UI. All visual elements inherit from NSView. Mirrors Apple's [`NSView`](https://developer.apple.com/documentation/appkit/nsview).

Every button, label, image, and container inherits from `NSView`.

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    NSView Lifecycle                        │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              initWithFrame(frameRect)                      │
│  ├─ Allocate unique ID                                      │
│  ├─ Set frame (position/size in parent)                     │
│  ├─ Set bounds (internal coordinate space)                │
│  └─ Initialize delegate, autoresizing mask                  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              addSubview(childView)                          │
│  ├─ Child gets added to subviews Vec                       │
│  ├─ Child's bounds syncs to its frame size                 │
│  └─ Parent becomes responsible for layout                  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              drawRect(dirtyRect)                           │
│  ├─ Called by RenderContext during window redraw           │
│  ├─ Delegate's draw() method invoked                         │
│  └─ Subviews draw recursively (if not hidden)              │
└─────────────────────────────────────────────────────────────┘
```

## Key Properties (Apple Parity)

| Property | Apple Header | Purpose |
|----------|--------------|---------|
| `frame` | `NSRect frame` | Position and size in parent coordinates |
| `bounds` | `NSRect bounds` | Internal coordinate space |
| `isHidden` | `BOOL hidden` | Visibility toggle |
| `isFlipped` | `BOOL flipped` | Coordinate system orientation |
| `isOpaque` | `BOOL opaque` | Opacity hint for optimization |
| `alphaValue` | `CGFloat alphaValue` | Transparency (0.0 - 1.0) |
| `identifier` | `NSString *identifier` | Unique ID for finding views |
| `wantsLayer` | `BOOL wantsLayer` | GPU layer backing |
| `clipsToBounds` | `BOOL clipsToBounds` | Clip subviews to frame |
| `autoresizingMask` | `NSAutoresizingMask` | Auto-layout behavior |

## Connection to Other Modules

```
NSView
├── NSControl (interactive elements)
│   ├── NSButton
│   ├── NSTextField
│   └── NSSlider
├── NSBox (container with border)
├── NSImageView (image display)
├── NSStackView (auto-layout container)
├── NSSplitView (divided panes)
├── NSScrollView (scrollable content)
└── NSWindow (root container)
    └── contentView (NSView)
```

## Usage Examples

### Basic View Creation
```rust
use rust_appkit::prelude::*;

// Create a view with specific frame
let view = NSView::new(NSRect::new(10.0, 10.0, 200.0, 100.0));
```

### View Hierarchy
```rust
let parent = NSView::new(NSRect::new(0.0, 0.0, 400.0, 300.0));
let child = NSView::new(NSRect::new(10.0, 10.0, 100.0, 50.0));

parent.add_subview(child.view_clone());
```

### Custom Drawing
```rust
use rust_appkit::render::RenderContext;

struct MyViewDelegate;

impl NSViewDelegate for MyViewDelegate {
    fn draw(&self, view: &NSView, context: &mut RenderContext) {
        let bounds = view.bounds();
        context.set_color(NSColor::system_red());
        context.fill_rect(bounds);
    }
    
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

let mut view = NSView::new(NSRect::new(0.0, 0.0, 100.0, 100.0));
view.set_delegate(MyViewDelegate);
```

### Autoresizing
```rust
let mut view = NSView::new(NSRect::new(10.0, 10.0, 100.0, 50.0));
view.set_autoresizing_mask(
    NSAutoresizingMask::WIDTH_SIZABLE | 
    NSAutoresizingMask::MAX_Y_MARGIN
);
// View will resize width and stick to top when parent resizes
```

## Coordinate Systems

```
Parent View (frame: x=0, y=0, w=400, h=300)
├── Child View (frame: x=10, y=20, w=100, h=50)
│   └── Child's bounds: x=0, y=0, w=100, h=50
│   └── Child draws at (0,0) in its own space
│   └── But appears at (10,20) in parent
```

## Event Handling Flow

```
NSEvent generated (mouse click at 150, 200)
        │
        ▼
NSWindow routes to hitTest(150, 200)
        │
        ▼
NSView checks if point is inside its frame
        │
        ▼
Delegate's mouse_down() called if hit
        │
        ▼
View can call setNeedsDisplay() to redraw
```

## Testing & Verification

```rust
// Verify view properties
let view = NSView::new(NSRect::new(10.0, 20.0, 100.0, 50.0));
assert_eq!(view.frame().min_x(), 10.0);
assert_eq!(view.bounds().min_x(), 0.0); // Always 0 in own space

// Test subview hierarchy
let mut parent = NSView::new(NSRect::new(0.0, 0.0, 400.0, 300.0));
let child = NSView::new(NSRect::new(10.0, 10.0, 100.0, 100.0));
parent.add_subview(child.view_clone());
assert_eq!(parent.subviews().len(), 1);
```

## File Structure

```
src/view/
├── mod.rs          # NSView struct and core implementation
├── layout.rs       # Autoresizing and layout engine
├── constraints.rs  # NSLayoutConstraint for Auto Layout
├── events.rs       # Mouse/Keyboard event handling
└── drawing.rs      # RenderContext integration
```

## Missing from Apple's NSView.h

- **CALayer integration**: Full Core Animation layer backing
- **Constraint-based layout**: Full Auto Layout constraint solver
- **Printing support**: `writePDFInsideRect`, `print:`
- **Full-screen mode**: `enterFullScreenMode`
- **Drag-and-drop**: Full NSDraggingDestination protocol
- **Accessibility**: Full NSAccessibility protocol
- **Text input**: NSTextInputClient protocol

These are stubbed or partially implemented for the current use case.
