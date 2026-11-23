# NSSlider — Value Selection Control

## Apple Header: `NSSlider.h`

## Overview

`NSSlider` provides horizontal and vertical sliders for selecting numeric values. Mirrors Apple's [`NSSlider`](https://developer.apple.com/documentation/appkit/nsslider).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSSlider Creation                            │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Configure Range                                 │
│  ├─ minValue: Starting value (e.g., 0.0)                │
│  ├─ maxValue: Ending value (e.g., 100.0)                 │
│  └─ initial: Starting position                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 User Interaction                             │
│  ├─ Click track: Jump to position                          │
│  ├─ Drag knob: Smooth value changes                        │
│  └─ Action callback: Fired continuously or on release       │
└─────────────────────────────────────────────────────────────┘
```

## Slider Types

| Type | Orientation | Typical Use |
|------|-------------|-------------|
| `Horizontal` | Left-to-right track | Most common (volume, opacity) |
| `Vertical` | Bottom-to-top track | Audio faders, scroll indicators |

## Connection to Other Modules

```
NSSlider
├── Inherits: NSControl
│   └── Inherits: NSView
├── Used by: Inspector panels (property sliders)
├── Used by: Audio mixers (volume faders)
└── Used by: Toolbar accessory views
```

## Usage Examples

### Horizontal Slider (Actual API)
```rust
use rust_appkit::prelude::*;

// Factory: min, max, value (from mod.rs:54-60)
let slider = NSSlider::horizontal(0.0, 100.0, 50.0);

// Or manual construction
let mut slider = NSSlider::new(NSRect::new(0.0, 0.0, 200.0, 21.0));
slider.set_min_value(0.0);   // mod.rs:73
slider.set_max_value(100.0); // mod.rs:84
slider.set_value(50.0);      // mod.rs:95
```

### Vertical Slider (Audio Fader)
```rust
let fader = NSSlider::vertical(0.0, 1.0, 0.75);
// Tall thin control
```

### With Action Callback (Actual API)
```rust
let mut opacity_slider = NSSlider::horizontal(0.0, 1.0, 1.0);

// on_action from mod.rs:235-238
opacity_slider.on_action(|| {
    let value = opacity_slider.value();
    viewport.set_opacity(value);
});

// Or use control action directly
opacity_slider.control_mut().set_action(|| {
    println!("Slider changed");
});
```

### Complete API (from mod.rs)
```rust
// Getters
let min = slider.min_value();      // mod.rs:79
let max = slider.max_value();      // mod.rs:90
let val = slider.value();          // mod.rs:103
let tick_count = slider.number_of_tick_marks();  // mod.rs:123
let is_vert = slider.is_vertical(); // mod.rs:138
let is_cont = slider.is_continuous(); // mod.rs:148

// Setters
slider.set_min_value(0.0);         // mod.rs:73
slider.set_max_value(100.0);       // mod.rs:84
slider.set_value(50.0);            // mod.rs:95
slider.set_alt_increment_value(5.0); // mod.rs:113
slider.set_number_of_tick_marks(10); // mod.rs:118
slider.set_allows_tick_mark_values_only(true); // mod.rs:128
slider.set_vertical(true);         // mod.rs:133
slider.set_continuous(true);       // mod.rs:143
slider.set_track_fill_color(Some(NSColor::blue())); // mod.rs:153

// View access
let view = slider.view();          // mod.rs:220
let view = slider.view_clone();    // mod.rs:225
let view = slider.as_view();       // mod.rs:215
let control = slider.control();    // mod.rs:201
let control = slider.control_mut(); // mod.rs:206
let view = slider.view_mut();      // mod.rs:210

// AppKit alias
slider.set_double_value(75.0);     // mod.rs:230
```

## Visual Rendering

```
Horizontal Slider
┌──────────────────────────────────────────────────────────┐
│ ◀──────●─────────────────────────────────────────────▶   │
│ min   current                                      max   │
└──────────────────────────────────────────────────────────┘

Vertical Slider
┌──────────┐
│    ▲     │
│    │     │
│    │     │
│    ●     │ ← current
│    │     │
│    │     │
│    ▼     │
└──────────┘
```

## Testing & Verification

```rust
let mut slider = NSSlider::horizontal(0.0, 100.0, 50.0);
assert_eq!(slider.min_value(), 0.0);
assert_eq!(slider.max_value(), 100.0);
assert_eq!(slider.value(), 50.0);

// Set new value
slider.set_value(75.0);
assert_eq!(slider.value(), 75.0);
```

## File Structure

```
src/slider/
└── mod.rs          # NSSlider, NSSliderType
```

## Missing from Apple's NSSlider

- **Tick marks**: Marked positions on track
- **Snapping**: Discrete value steps
- **Accessory views**: Icons at ends
- **Number formatter**: Display formatting
- **Alt increment**: Small/large step sizes
- **Vertical flip**: Bottom-to-top vs top-to-bottom

These are stubbed for current use cases.
