# Step-by-Step: Animation Editor

> Complete guide to creating animations with keyframes and curves

---

## 🗺 Documentation Map

- [← Back to Reality Composer Pro Guide](./JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🎬 Animation System API](../../../../../phi/crates/rust-reality-kit/src/animation.rs)
- [🏠 RealityKit Home](../realitykit/-realitykit.md)

---

## 📊 Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| **Timeline UI** | ✅ Working | Multi-track timeline with scrubbing |
| **Keyframes** | ✅ Working | Add, move, delete keyframes |
| **Easing Curves** | ✅ Working | Linear, Quad, Cubic, Bezier |
| **Playback Controls** | ✅ Working | Play, pause, step, stop |
| **Animation Tracks** | ✅ Working | Position, Rotation, Scale |
| **Entity Binding** | ⚠️ Partial | Basic component integration |
| **Animation Export** | ⚠️ Partial | JSON serialization works |
| **Bone Animation** | ❌ Not Started | Planned for future |

**Last Updated:** 2024-01-20  
**App Status:** ✅ Running & Ready for testing  
**Crate:** `phi/crates/rust-kit-pro/src/animation_editor.rs`

---

## Overview

The Animation Editor in rust-kit-pro provides a professional timeline interface for creating, editing, and previewing animations with keyframes, easing curves, and real-time playback.

---

## Features

| Feature | Description |
|---------|-------------|
| **Timeline** | Multi-track timeline with scrubbing |
| **Keyframes** | Add, move, delete keyframes |
| **Curves** | Easing curve editor (Linear, Quad, Cubic) |
| **Playback** | Play, pause, step forward/back |
| **Tracks** | Position, Rotation, Scale animation tracks |

---

## Step 1: Opening the Animation Editor

```rust
use rust_kit_pro::animation_editor::AnimationEditor;

fn open_animation_editor() {
    let mut editor = AnimationEditor::new();
    // Editor is accessible via timeline panel (key 4)
}
```

**Keyboard Shortcut:** Press `4` in rust-kit-pro to open Animation panel.

---

## Step 2: Creating Animation Tracks

```rust
use reality_kit::animation::{AnimationTrack, Keyframe};
use glam::Vec3;

fn create_position_track() -> AnimationTrack {
    let mut track = AnimationTrack::position("Cube Position");
    
    // Add keyframes
    track.add_keyframe(Keyframe::new(0.0, Vec3::new(0.0, 0.0, 0.0)));
    track.add_keyframe(Keyframe::new(1.0, Vec3::new(5.0, 0.0, 0.0)));
    track.add_keyframe(Keyframe::new(2.0, Vec3::new(5.0, 3.0, 0.0)));
    
    track
}
```

---

## Step 3: Setting Keyframe Easing

```rust
use reality_kit::animation::EasingFunction;

fn apply_easing() {
    // Set easing for smooth animation
    let easing = EasingFunction::EaseInOutCubic;
    
    // Options:
    // - Linear
    // - EaseInQuad, EaseOutQuad, EaseInOutQuad
    // - EaseInCubic, EaseOutCubic, EaseInOutCubic
}
```

---

## Step 4: Playback Controls

| Button | Action | Shortcut |
|--------|--------|----------|
| ⏮ | Go to Start | Home |
| ▶ | Play / ⏸ Pause | Space |
| ⏹ | Stop | End |
| ⏭ | Go to End | Ctrl+End |
| ⏭ | Step Forward | Right Arrow |
| ⏮ | Step Back | Left Arrow |

---

## Step 5: Timeline Navigation

```rust
// Zoom controls
editor.zoom = 100.0; // pixels per second
editor.scroll_offset = 0.0; // pan left/right

// Current time
editor.current_time = 1.5; // jump to 1.5 seconds
```

**Mouse Controls:**
- **Scroll:** Zoom timeline
- **Middle-drag:** Pan timeline
- **Click:** Set playhead position
- **Drag keyframe:** Move keyframe

---

## Step 6: Curve Editor

Enable "Curves" checkbox to show the easing curve editor:

```rust
editor.show_curves = true;
```

**Curve Types:**
- Linear: Constant speed
- EaseIn: Accelerate from start
- EaseOut: Decelerate to end
- EaseInOut: Accelerate then decelerate

---

## Full Example

```rust
use rust_kit_pro::animation_editor::{AnimationEditor, KeyframeEditor};
use reality_kit::animation::{AnimationTimeline, EasingFunction};

fn setup_animation() {
    // Create timeline
    let mut timeline = AnimationTimeline::new(10.0); // 10 second duration
    
    // Add position track
    let mut pos_track = timeline.add_position_track();
    pos_track.add_keyframe(0.0, Vec3::ZERO);
    pos_track.add_keyframe(2.0, Vec3::new(5.0, 0.0, 0.0));
    pos_track.set_easing(EasingFunction::EaseInOutCubic);
    
    // Add rotation track
    let mut rot_track = timeline.add_rotation_track();
    rot_track.add_keyframe(0.0, Quat::IDENTITY);
    rot_track.add_keyframe(3.0, Quat::from_rotation_y(PI));
    
    // Playback
    timeline.play();
}
```

---

## Next Steps

- [Building Materials](building-materials/--jingle-building-materials.md)
- [Importing Assets](STEP_BY_STEP_ASSET_IMPORT_GUIDE.md)

---

*Generated for rust-kit-pro v0.1.0*
