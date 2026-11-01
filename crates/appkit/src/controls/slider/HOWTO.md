# Slider — How to Use

## Files
- `mod.rs` — `NSSlider` struct, constructors, value/drag logic, tick marks

## Quick Start
```rust
use rust_appkit::controls::slider::NSSlider;

// Horizontal slider (min, max, initial)
let mut slider = NSSlider::horizontal(0.0, 100.0, 50.0);

// Action callback on value change
slider.on_action(|| {
    println!("Slider moved");
});

// Set value programmatically
slider.set_value(75.0);

// Tick marks
slider.set_number_of_tick_marks(5);
slider.set_allows_tick_mark_values_only(true);

// Vertical slider
let mut v_slider = NSSlider::vertical(0.0, 1.0, 0.5);
```

## Key APIs
- `NSSlider::horizontal(min, max, value)` — horizontal slider
- `NSSlider::vertical(min, max, value)` — vertical slider
- `slider.set_value(f64)` — set current value (clamped to range)
- `slider.value()` — get current value
- `slider.set_min_value(f64)` / `slider.set_max_value(f64)` — range
- `slider.on_action(closure)` — callback on change
- `slider.set_continuous(bool)` — fire during drag vs. only on release
- `slider.set_number_of_tick_marks(n)` — snap points
- `slider.set_allows_tick_mark_values_only(bool)` — snap-only mode
- `slider.drag_to(position)` — simulate drag (0.0–1.0 normalized)
- `slider.closest_tick_mark_value_to_value(f64)` — snap helper

## Rules
- Values are always clamped to `[min_value, max_value]`
- `continuous = true` (default) fires action during drag
- `continuous = false` fires action only on mouse_up
- Tick mark snapping uses nearest-tick rounding
- Call `mouse_down()` before `drag_to()` to start a drag sequence
