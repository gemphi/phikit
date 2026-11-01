# NSBox — Container and Grouping View

## Apple Header: `NSBox.h`

## Overview

`NSBox` groups related views with a visual container. Mirrors Apple's [`NSBox`](https://developer.apple.com/documentation/appkit/nsbox).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    NSBox Creation                            │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Primary Box (default)                          │
│  ├─ Rounded rectangle with fill color                       │
│  ├─ Optional border                                        │
│  ├─ Optional title at top                                  │
│  └─ ContentView for subviews                               │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Separator (NSBoxSeparator)                      │
│  ├─ 1px line (horizontal or vertical)                      │
│  └─ No content, no title                                   │
└─────────────────────────────────────────────────────────────┘
```

## Box Types (NSBoxType)

| Type | Apple Constant | Purpose |
|------|----------------|---------|
| `Primary` | `NSBoxPrimary` | Standard group box with title |
| `Separator` | `NSBoxSeparator` | Divider line (horiz/vert) |
| `Custom` | `NSBoxCustom` | User-defined colors/borders |

## Title Positions (NSTitlePosition)

| Position | Description |
|----------|-------------|
| `NoTitle` | Box has no title |
| `AboveTop` | Title above box border |
| `AtTop` | Title centered on top border |
| `BelowTop` | Title inside, at top |

## Connection to Other Modules

```
NSBox
├── Contains: NSView (as contentView)
│   └── Subviews added here
├── Used by: NSStackView (nested grouping)
├── Used by: NSSplitView (pane containers)
└── Used by: Inspector panels (rust-kit-pro)
```

## Usage Examples

### Group Box with Title
```rust
use rust_appkit::prelude::*;
use rust_appkit::box_view::{NSBox, NSBoxType, NSTitlePosition};

let mut box_view = NSBox::new(NSRect::new(10.0, 10.0, 300.0, 200.0));
box_view.set_title("Transform Properties");
box_view.set_title_position(NSTitlePosition::AtTop);
box_view.set_fill_color(NSColor::new(0.12, 0.12, 0.14, 1.0));
box_view.set_border_color(NSColor::new(0.25, 0.25, 0.28, 1.0));

// Add content to box
let content = box_view.view_mut();
let label = NSTextField::label("Position X:");
content.add_subview(label.view_clone());
```

### Horizontal Separator
```rust
let separator = NSBox::separator(NSRect::new(0.0, 100.0, 400.0, 1.0));
// Automatically detects: width > height = horizontal
```

### Vertical Separator
```rust
let separator = NSBox::separator(NSRect::new(200.0, 0.0, 1.0, 300.0));
// Automatically detects: height > width = vertical
```

### Custom Styled Box
```rust
let mut custom_box = NSBox::new(NSRect::new(0.0, 0.0, 200.0, 150.0));
custom_box.set_box_type(NSBoxType::Custom);
custom_box.set_fill_color(NSColor::new(0.1, 0.1, 0.1, 0.8));
custom_box.set_border_color(NSColor::system_accent());
custom_box.set_border_width(2.0);
custom_box.set_corner_radius(8.0);
```

## Visual Rendering

```
NSBox Drawing Order
├── Fill (background color)
│   └── Rounded rectangle
├── Border (if border_width > 0)
│   └── Outset stroke
└── Title (if set)
    └── Text at title_position
```

## Inspector Panel Example (rust-kit-pro)

```rust
// Inspector sections use NSBox for grouping
fn create_inspector_section(title: &str, height: f32) -> NSBox {
    let mut box_view = NSBox::new(NSRect::new(0.0, 0.0, 320.0, height));
    box_view.set_title(title);
    box_view.set_title_position(NSTitlePosition::AtTop);
    box_view.set_fill_color(NSColor::new(0.14, 0.14, 0.16, 1.0));
    box_view.set_border_width(0.0); // No border for clean look
    box_view
}

let transform_section = create_inspector_section("Transform", 150.0);
let material_section = create_inspector_section("Material", 200.0);
```

## Testing & Verification

```rust
// Test separator detection
let h_sep = NSBox::separator(NSRect::new(0.0, 0.0, 100.0, 1.0));
// Renders as horizontal line

let v_sep = NSBox::separator(NSRect::new(0.0, 0.0, 1.0, 100.0));
// Renders as vertical line

// Test custom colors
let mut box_view = NSBox::new(NSRect::new(0.0, 0.0, 100.0, 100.0));
box_view.set_fill_color(NSColor::red());
box_view.set_border_color(NSColor::blue());
box_view.set_border_width(2.0);
```

## File Structure

```
src/box_view/
└── mod.rs          # NSBox, NSBoxType, NSTitlePosition
```

## Missing from Apple's NSBox.h

- **Secondary type**: Deprecated, merged into Primary
- **OldStyle type**: Deprecated Mac OS X 10.2 look
- **Title font**: Custom font for title text
- **ContentViewMargins**: Padding inside box
- **Transparent boxes**: Full transparency support

These are stubbed or simplified for current use cases.
