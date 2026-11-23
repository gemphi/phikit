# NSTimelineView — Animation Timeline

## Overview

`NSTimelineView` provides an animation timeline editor for scrubbing, keyframes, and playback control. Custom component for rust-kit-pro's animation system.

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 Timeline Structure                           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Timeline Components                             │
│  ├─ Time ruler: Shows frames/seconds                        │
│  ├─ Track lanes: One per animated property                  │
│  ├─ Keyframes: Dots on tracks at specific times             │
│  ├─ Playhead: Current time indicator                        │
│  └─ Controls: Play, pause, stop, loop                       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Interaction                                  │
│  ├─ Scrub: Drag playhead to preview                         │
│  ├─ Add keyframe: Click on track at time                    │
│  ├─ Move keyframe: Drag left/right                          │
│  ├─ Delete keyframe: Right-click, delete                  │
│  └─ Play: Start playback animation                          │
└─────────────────────────────────────────────────────────────┘
```

## Timeline Components

| Component | Description |
|-----------|-------------|
| **Ruler** | Time scale (frames or seconds) |
| **Tracks** | One per property (position.x, rotation.y, etc) |
| **Keyframes** | Red diamonds at specific times |
| **Playhead** | Vertical line showing current time |
| **Controls** | Play/pause, frame counter, loop toggle |

## Connection to Other Modules

```
NSTimelineView
├── Uses: NSStackView (track layout)
├── Uses: NSSlider (scrubbing)
├── Uses: NSButton (playback controls)
├── Used by: rust-kit-pro animation panel
└── Integrates with: ECS animation system
```

## Usage Examples

### Basic Timeline
```rust
use rust_appkit::prelude::*;
use rust_appkit::timeline::NSTimelineView;

let mut timeline = NSTimelineView::new(NSRect::new(0.0, 0.0, 800.0, 200.0));

// Set duration (60 fps, 5 seconds)
timeline.set_duration_frames(300);

// Add tracks
timeline.add_track("Position X", TrackType::Float);
timeline.add_track("Position Y", TrackType::Float);
timeline.add_track("Position Z", TrackType::Float);
timeline.add_track("Rotation", TrackType::Quaternion);

// Add keyframes
timeline.add_keyframe("Position X", 0, 0.0);
timeline.add_keyframe("Position X", 150, 10.0); // 2.5 sec
```

### Playback Control
```rust
timeline.set_playback_handler(|state| {
    match state {
        PlaybackState::Play => start_animation_loop(),
        PlaybackState::Pause => pause_animation(),
        PlaybackState::Stop => {
            stop_animation();
            timeline.set_current_frame(0);
        }
    }
});

// Scrub callback
timeline.set_scrub_handler(|frame| {
    preview_frame(frame);
});
```

### Entity Animation Binding
```rust
fn bind_entity_to_timeline(entity: Entity, timeline: &mut NSTimelineView) {
    // Get transform component
    let transform = world.get::<Transform>(entity).unwrap();
    
    // Create tracks matching component fields
    timeline.add_track("Transform.Position.X", TrackType::Float);
    timeline.add_track("Transform.Position.Y", TrackType::Float);
    timeline.add_track("Transform.Position.Z", TrackType::Float);
    
    // Bind current values as keyframe at frame 0
    timeline.add_keyframe("Transform.Position.X", 0, transform.position.x);
    timeline.add_keyframe("Transform.Position.Y", 0, transform.position.y);
    timeline.add_keyframe("Transform.Position.Z", 0, transform.position.z);
}
```

## Timeline Layout

```
┌────────────────────────────────────────────────────────────────────┐
│ ▶  ⏸  ⏹   Frame: 150 / 300   [Loop ▣]  Time: 2.50s              │  ← Controls
├────────┬─────────────────────────────────────────────────────────┤
│        │  0    30    60    90   120   150   180   210   240   300 │  ← Ruler
│ Pos.X  │──●─────────────────────────────────────●──────────────────│  ← Track
│ Pos.Y  │──●─────────────────────────────────────●──────────────────│
│ Pos.Z  │──●─────────────────────────────────────●──────────────────│
│ Rot.X  │  │                                                 │    │
│ Rot.Y  │  │                                                 │    │
│ Rot.Z  │  │                                                 │    │
│        │  │                                                 │    │
└────────┴───────────────────────────────────────────────────────────┘
           ↑
         Playhead (at frame 150)
```

## Testing & Verification

```rust
let mut timeline = NSTimelineView::new(NSRect::new(0.0, 0.0, 400.0, 150.0));
timeline.set_duration_frames(100);

// Add track and keyframes
timeline.add_track("Test", TrackType::Float);
timeline.add_keyframe("Test", 0, 0.0);
timeline.add_keyframe("Test", 50, 5.0);
timeline.add_keyframe("Test", 100, 0.0);

assert_eq!(timeline.keyframe_count("Test"), 3);

// Test scrub
timeline.set_current_frame(25);
let interpolated = timeline.value_at_frame("Test", 25);
assert_eq!(interpolated, 2.5); // Linear interpolation
```

## File Structure

```
src/timeline/
├── mod.rs          # NSTimelineView, TrackType, PlaybackState
├── track.rs        # TimelineTrack, Keyframe
└── ruler.rs        # TimeRuler, frame/second display
```

## Missing from NSTimelineView

- **Bezier curves**: Smooth keyframe interpolation
- **Audio waveform**: Sync with audio tracks
- **Layers/groups**: Nested track organization
- **Markers**: Named time positions
- **Export**: Video/GIF render
- **Import**: Animation file formats

These are stubbed for current use cases.

## Related

- **Animation panel** in rust-kit-pro uses this view
- Integrates with **reality-foundation** animation system
- Supports **ECS** component animation
