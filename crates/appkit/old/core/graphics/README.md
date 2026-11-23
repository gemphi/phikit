# Graphics — Colors, Images, and Geometry

## Overview

`graphics` module provides fundamental types for colors, images, rectangles, and points. Core types used throughout rust-appkit. Mirrors Apple's `NSColor`, `NSImage`, `NSRect`, `NSPoint` from `AppKit.framework/Headers/`.

## Types

| Type | Description | Apple Equivalent |
|------|-------------|------------------|
| `NSColor` | RGBA color | `NSColor` |
| `NSImage` | Bitmap image | `NSImage` |
| `NSRect` | Rectangle (x, y, w, h) | `NSRect` |
| `NSPoint` | 2D point (x, y) | `NSPoint` |
| `NSSize` | 2D size (w, h) | `NSSize` |

## Colors

### System Colors
```rust
use rust_appkit::graphics::NSColor;

let white = NSColor::white();
let black = NSColor::black();
let clear = NSColor::clear();
let red = NSColor::red();
let green = NSColor::green();
let blue = NSColor::blue();
```

### Dynamic Colors (adapt to appearance)
```rust
let label = NSColor::labelColor();           // Text color
let control = NSColor::controlColor();         // Control background
let accent = NSColor::controlAccentColor();    // Accent/highlight
let selected = NSColor::selectedContentBackgroundColor();
```

### Custom Colors
```rust
let custom = NSColor::new(0.2, 0.5, 0.8, 1.0); // r, g, b, a
let with_alpha = NSColor::new(1.0, 1.0, 1.0, 0.5); // 50% white
```

## Geometry

### Rectangles
```rust
use rust_appkit::graphics::{NSRect, NSPoint, NSSize};

// Create rect
let rect = NSRect::new(10.0, 20.0, 100.0, 50.0);
// x=10, y=20, width=100, height=50

// Access properties
let x = rect.x;
let y = rect.y;
let width = rect.width;
let height = rect.height;

// Check containment
let point = NSPoint::new(50.0, 40.0);
let contains = rect.contains(point); // true

// Inset
let inset = rect.inset(5.0, 5.0); // Shrink by 5 on each side

// Intersection
let other = NSRect::new(80.0, 30.0, 50.0, 50.0);
let intersection = rect.intersection(other);
```

### Points and Sizes
```rust
let point = NSPoint::new(100.0, 200.0);
let size = NSSize::new(640.0, 480.0);

// Distance between points
let p1 = NSPoint::new(0.0, 0.0);
let p2 = NSPoint::new(3.0, 4.0);
let distance = p1.distance_to(p2); // 5.0
```

## Images

### Creating Images
```rust
use rust_appkit::graphics::NSImage;

// From file
let image = NSImage::from_file("icon.png")?;

// From bytes
let image = NSImage::from_bytes(&png_bytes)?;

// Create blank
let image = NSImage::new(100, 100);
```

### Image Properties
```rust
let size = image.size(); // NSSize
let width = size.width;
let height = size.height;
```

## Connection to Other Modules

```
Graphics Types
├── Used by: All view modules (frame, bounds, background)
├── Used by: RenderContext (drawing)
├── Used by: Theme system (colors)
└── Foundation for: Geometry calculations
```

## File Structure

```
src/graphics/
├── mod.rs          # NSColor, NSRect, NSPoint, NSSize
└── image.rs        # NSImage
```

## Testing & Verification

```rust
let color = NSColor::new(1.0, 0.5, 0.0, 1.0);
assert_eq!(color.red(), 1.0);
assert_eq!(color.green(), 0.5);
assert_eq!(color.blue(), 0.0);
assert_eq!(color.alpha(), 1.0);

let rect = NSRect::new(0.0, 0.0, 100.0, 100.0);
let point = NSPoint::new(50.0, 50.0);
assert!(rect.contains(point));

let far = NSPoint::new(150.0, 150.0);
assert!(!rect.contains(far));
```

## Related

- **NSImageView**: Displays `NSImage`
- **NSColorWell**: Color picker (stubbed)
- **Theme system**: Uses `NSColor` for definitions
