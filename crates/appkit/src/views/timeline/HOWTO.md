# Timeline — How to Use

## Files
- `mod.rs` — `NSTimelineView`, `TimelineState`, global active timelines registry
- `delegate.rs` — `TimelineDelegate` handles drawing + scrubbing
- `track.rs` — `TimelineTrackBase` trait for animation tracks
- `builder.rs` — builder helpers for timeline construction

## Quick Start
```rust
use rust_appkit::views::timeline::NSTimelineView;

// Create a timeline view
let mut timeline = NSTimelineView::new(frame);

// Add animation tracks
timeline.add_track(my_position_track);
timeline.add_track(my_rotation_track);

// Playback control
timeline.play();
timeline.pause();
timeline.stop(); // resets current_time to 0

// Access state directly
if let Ok(state) = timeline.state.lock() {
    println!("Time: {}/{}", state.current_time, state.duration);
}
```

## Key APIs
- `NSTimelineView::new(frame)` — create + register in global active list
- `timeline.add_track(track)` — add a `TimelineTrackBase` implementor
- `timeline.play()` — set `is_playing = true`
- `timeline.pause()` — set `is_playing = false`
- `timeline.stop()` — pause + reset time to 0
- `timeline.view()` / `timeline.view_mut()` — underlying NSView
- `get_active_timelines()` — global registry for tick/drive from other crates

## Rules
- Tracks implement `TimelineTrackBase + Send + Sync`
- State is `Arc<Mutex<TimelineState>>` — safe to read from render thread
- Active timelines are globally registered for external tick drivers
- Delegate handles playhead rendering and scrub interaction
- `duration` defaults to 10.0 seconds
