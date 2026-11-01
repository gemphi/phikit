# NSAnimation — Animated Property Changes

## Apple Header: `NSAnimation.h`

## Overview

`NSAnimation` provides smooth interpolation of values over time. Mirrors Apple's [`NSAnimation`](https://developer.apple.com/documentation/appkit/nsanimation).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSAnimation Setup                            │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Animation Configuration                       │
│  ├─ Duration: Seconds to complete                            │
│  ├─ Curve: Timing function (ease, linear, etc)             │
│  ├─ Frame rate: Updates per second                         │
│  └─ Value range: Start and end values                      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Animation Loop                               │
│  ├─ Start animation                                          │
│  ├─ Timer fires at frame rate                              │
│  ├─ Calculate interpolated value                             │
│  ├─ Callback updates property                                │
│  └─ Complete when progress = 1.0                           │
└─────────────────────────────────────────────────────────────┘
```

## Animation Curves

| Curve | Description | Use Case |
|-------|-------------|----------|
| `Linear` | Constant speed | Mechanical motion |
| `EaseIn` | Slow start, fast end | Entering view |
| `EaseOut` | Fast start, slow end | Exiting view |
| `EaseInOut` | Slow both ends | Smooth transitions |

## Connection to Other Modules

```
NSAnimation
├── Animates: NSView properties (frame, alpha)
├── Animates: NSColor transitions
├── Uses: NSTimer (frame updates)
└── Used by: View transitions
```

## Usage Examples

### Fade Animation
```rust
use rust_appkit::prelude::*;
use rust_appkit::animation::{NSAnimation, NSAnimationCurve};

fn fade_out(view: &mut NSView, duration: f64) {
    let start_alpha = view.alpha_value();
    
    let mut anim = NSAnimation::new(duration);
    anim.set_animation_curve(NSAnimationCurve::EaseInOut);
    
    anim.set_progress_handler(|progress| {
        let alpha = start_alpha * (1.0 - progress);
        view.set_alpha_value(alpha);
    });
    
    anim.start();
}
```

### Frame Animation
```rust
fn animate_frame(view: &mut NSView, target: NSRect, duration: f64) {
    let start = view.frame();
    
    let mut anim = NSAnimation::new(duration);
    
    anim.set_progress_handler(move |progress| {
        let x = lerp(start.x, target.x, progress);
        let y = lerp(start.y, target.y, progress);
        let w = lerp(start.width, target.width, progress);
        let h = lerp(start.height, target.height, progress);
        
        view.set_frame(NSRect::new(x, y, w, h));
    });
    
    anim.start();
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
```

### View Transition
```rust
// Crossfade between two views
let mut fade_out = NSAnimation::new(0.3);
fade_out.set_progress_handler(|p| {
    old_view.set_alpha_value(1.0 - p);
});

let mut fade_in = NSAnimation::new(0.3);
fade_in.set_progress_handler(|p| {
    new_view.set_alpha_value(p);
});

fade_out.start();
// Chain: fade_in starts after fade_out completes
```

## Animation Groups

```rust
let mut group = NSAnimationGroup::new();

let fade = NSAnimation::new(0.5);
let move = NSAnimation::new(0.5);
let scale = NSAnimation::new(0.5);

group.add_animation(fade);
group.add_animation(move);
group.add_animation(scale);

group.start();
// All animations run concurrently
```

## Testing & Verification

```rust
let mut anim = NSAnimation::new(1.0);
anim.set_animation_curve(NSAnimationCurve::Linear);

let progress_values: Vec<f64> = vec![];
anim.set_progress_handler(|p| {
    progress_values.push(p);
});

anim.start();
// After 1 second, progress_values should go 0.0 → 1.0
```

## File Structure

```
src/animation/
└── mod.rs          # NSAnimation, NSAnimationCurve, NSAnimationGroup
```

## Missing from Apple's NSAnimation

- **Spring animations**: Physics-based bouncing
- **Keyframe animations**: Multiple value points
- **Core Animation**: CALayer integration
- **Transaction grouping**: Batch property changes
- **Implicit animations**: Automatic on property set
- **Completion blocks**: Callback on finish
- **Pause/resume**: Animation control

These are stubbed for current use cases.
