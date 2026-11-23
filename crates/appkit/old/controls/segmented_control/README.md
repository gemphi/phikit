# NSSegmentedControl — Segmented Selection

## Apple Header: `NSSegmentedControl.h`

## Overview

`NSSegmentedControl` provides a horizontal row of segments where one or more can be selected. Mirrors Apple's [`NSSegmentedControl`](https://developer.apple.com/documentation/appkit/nssegmentedcontrol).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSSegmentedControl Structure               │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Segments                                          │
│  ├─ Each segment has: label or image, width, state          │
│  ├─ Selected segment: highlighted                           │
│  └─ Tracking mode: Select one, select any, or momentary     │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Interaction                                    │
│  ├─ Click segment → Selection changes                       │
│  ├─ Action fires with selected index                        │
│  └─ Visual state updates                                     │
└─────────────────────────────────────────────────────────────┘
```

## Tracking Modes

| Mode | Behavior |
|------|----------|
| **SelectOne** | Only one selected at a time (radio) |
| **SelectAny** | Multiple can be selected (checkboxes) |
| **Momentary** | Press and release, no selection state |

## Segment Styles

| Style | Appearance |
|-------|------------|
| `Automatic` | Platform default |
| `Rounded` | Pill-shaped segments |
| `TexturedRounded` | Dark textured look |
| `Separated` | Space between segments |
| `SmallSquare` | Compact square |

## Connection to Other Modules

```
NSSegmentedControl
├── Inherits: NSControl
├── Used by: Toolbars (view modes)
├── Used by: Filters (category selection)
└── Alternative to: NSTabView (when no content switch)
```

## Usage Examples

### Mode Selector
```rust
use rust_appkit::prelude::*;
use rust_appkit::segmented_control::{NSSegmentedControl, NSSegmentStyle, NSSegmentSwitchTracking};

let mut mode_selector = NSSegmentedControl::new(NSRect::new(0.0, 0.0, 180.0, 24.0));

mode_selector.set_segment_count(3);
mode_selector.set_label("Wireframe", 0);
mode_selector.set_label("Shaded", 1);
mode_selector.set_label("Rendered", 2);

mode_selector.set_segment_style(NSSegmentStyle::Rounded);
mode_selector.set_tracking_mode(NSSegmentSwitchTracking::SelectOne);

// Select first by default
mode_selector.set_selected_segment(0);

mode_selector.set_action(|selected_index| {
    match selected_index {
        0 => set_view_mode(ViewMode::Wireframe),
        1 => set_view_mode(ViewMode::Shaded),
        2 => set_view_mode(ViewMode::Rendered),
        _ => {}
    }
});
```

### View Controls
```rust
let mut view_controls = NSSegmentedControl::new(NSRect::new(0.0, 0.0, 100.0, 24.0));
view_controls.set_segment_count(2);

// Icons instead of labels
view_controls.set_image(zoom_out_icon(), 0);
view_controls.set_image(zoom_in_icon(), 1);

view_controls.set_action(|index| {
    if index == 0 { zoom_out(); } else { zoom_in(); }
});
```

### Toggle Filters
```rust
let mut filters = NSSegmentedControl::new(NSRect::new(0.0, 0.0, 200.0, 24.0));
filters.set_segment_count(4);
filters.set_tracking_mode(NSSegmentSwitchTracking::SelectAny);

filters.set_label("Meshes", 0);
filters.set_label("Lights", 1);
filters.set_label("Cameras", 2);
filters.set_label("Helpers", 3);

// All selected by default
for i in 0..4 {
    filters.set_selected(true, i);
}

filters.set_action(|_| {
    update_visibility(
        filters.is_selected(0), // meshes
        filters.is_selected(1), // lights
        filters.is_selected(2), // cameras
        filters.is_selected(3), // helpers
    );
});
```

## Testing & Verification

```rust
let mut control = NSSegmentedControl::new(NSRect::new(0.0, 0.0, 150.0, 24.0));
control.set_segment_count(3);
control.set_label("One", 0);
control.set_label("Two", 1);
control.set_label("Three", 2);

control.set_selected_segment(1);
assert_eq!(control.selected_segment(), 1);

control.set_tracking_mode(NSSegmentSwitchTracking::SelectAny);
control.set_selected(true, 0);
control.set_selected(true, 2);
assert!(control.is_selected(0));
assert!(!control.is_selected(1));
assert!(control.is_selected(2));
```

## File Structure

```
src/segmented_control/
└── mod.rs          # NSSegmentedControl, NSSegmentStyle, NSSegmentSwitchTracking
```

## Missing from Apple's NSSegmentedControl

- **Dividers**: Visual separation between segments
- **Spring width**: Auto-sizing segments
- **Help tags**: Per-segment tooltips
- **Menu**: Dropdown from segment
- **Images and text**: Both in same segment

These are stubbed for current use cases.

## Comparison

| NSSegmentedControl | NSTabView |
|--------------------|-----------|
| Compact row of buttons | Full tab interface |
| No content switching | Shows different views |
| Good for toolbars | Good for preferences |
| Mode selection | Content organization |
