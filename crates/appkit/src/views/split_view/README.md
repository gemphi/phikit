# NSSplitView — Divided Panes

## Apple Header: `NSSplitView.h`

## Overview

`NSSplitView` divides a window into resizable panes with draggable dividers. Mirrors Apple's [`NSSplitView`](https://developer.apple.com/documentation/appkit/nssplitview).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSSplitView Creation                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Add Subviews (Panes)                            │
│  ├─ Left/Top pane (sidebar)                                │
│  ├─ Center pane (main content)                             │
│  └─ Right/Bottom pane (inspector)                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Configure Divider Style                         │
│  ├─ Thick (traditional macOS)                               │
│  ├─ Thin (modern look)                                      │
│  └─ PaneSplitter (no visible divider)                       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 User Interaction                             │
│  ├─ Mouse down on divider starts drag                       │
│  ├─ Mouse dragged updates pane sizes                        │
│  ├─ Mouse up commits new sizes                              │
│  └─ Panes resize based on autoresizingMask                 │
└─────────────────────────────────────────────────────────────┘
```

## Divider Styles (NSSplitViewDividerStyle)

| Style | Appearance | Use Case |
|-------|------------|----------|
| `Thick` | 9px wide, grippy | Legacy Mac look |
| `Thin` | 1px line | Modern, minimal |
| `PaneSplitter` | Invisible | Seamless panels |

## Connection to Other Modules

```
NSSplitView
├── Contains: NSView[] (panes)
├── Uses: NSSplitViewController (optional)
│   └── Manages multiple split views
├── Used by: NSWindow (main window layout)
└── Used by: rust-kit-pro (4-panel layout)
```

## rust-kit-pro 4-Panel Layout

```
┌─────────────────────────────────────────────────────────────┐
│                    Main NSSplitView                          │
│  ┌────────────────┬──────────────────────────────────────┐  │
│  │                │                                      │  │
│  │   Hierarchy    │         Center NSSplitView          │  │
│  │   (Sidebar)    │  ┌──────────────────────────────┐  │  │
│  │                │  │                              │  │  │
│  │   320px        │  │       3D Viewport            │  │  │
│  │                │  │                              │  │  │
│  │                │  └──────────────────────────────┘  │  │
│  │                │  ┌──────────────────────────────┐  │  │
│  │                │  │                              │  │  │
│  │                │  │       Timeline Panel           │  │  │
│  │                │  │       (Footer)                 │  │  │
│  │                │  │                              │  │  │
│  │                │  └──────────────────────────────┘  │  │
│  └────────────────┴──────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        Bottom Tabs / Status Bar                        │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Usage Examples

### Basic Horizontal Split
```rust
use rust_appkit::prelude::*;

let mut split = NSSplitView::new(NSRect::new(0.0, 0.0, 800.0, 600.0));
split.set_divider_style(NSSplitViewDividerStyle::Thin);

let sidebar = create_sidebar();
let main_content = create_viewport();

split.add_subview(sidebar);
split.add_subview(main_content.view_clone());
// Sidebar gets ~1/3, main gets ~2/3
```

### Vertical Split (Top/Bottom)
```rust
let mut split = NSSplitView::new(NSRect::new(0.0, 0.0, 800.0, 600.0));
split.set_is_vertical(true); // Stack top-to-bottom

let top_pane = create_viewport();
let bottom_pane = create_timeline();

split.add_subview(top_pane);
split.add_subview(bottom_pane.view_clone());
```

### Nested Splits (Reality Composer Pro)
```rust
// Outer: Left sidebar | Center content
let mut outer_split = NSSplitView::new(frame);
outer_split.set_divider_style(NSSplitViewDividerStyle::Thin);

let sidebar = create_hierarchy_panel();
let center_area = create_center_area();

outer_split.add_subview(sidebar);
outer_split.add_subview(center_area.view_clone());

// Center: Viewport (top) | Timeline (bottom)
let mut center_split = NSSplitView::new(center_frame);
center_split.set_is_vertical(true);

let viewport = create_3d_viewport();
let timeline = create_timeline_panel();

center_split.add_subview(viewport);
center_split.add_subview(timeline.view_clone());
```

## Divider Drag Flow

```
User clicks divider
        │
        ▼
NSSplitViewDelegate.mouse_down() - NO
(wants_mouse_down() returns true)
        │
        ▼
Store divider index and start position
        │
        ▼
Mouse dragged
        │
        ▼
Calculate delta from start position
        │
        ▼
Resize left pane by delta
Resize right pane by -delta
        │
        ▼
Call layout_subviews() on affected panes
```

## Testing & Verification

```rust
let mut split = NSSplitView::new(NSRect::new(0.0, 0.0, 300.0, 200.0));

let left = NSView::new(NSRect::new(0.0, 0.0, 100.0, 200.0));
let right = NSView::new(NSRect::new(0.0, 0.0, 200.0, 200.0));

split.add_subview(left);
split.add_subview(right.view_clone());

// After layout, left.frame.width + divider + right.frame.width = 300
```

## File Structure

```
src/split_view/
├── mod.rs          # NSSplitView, NSSplitViewDividerStyle
├── controller.rs   # NSSplitViewController (multi-split management)
└── item.rs         # NSSplitViewItem (pane configuration)
```

## Missing from Apple's NSSplitView

- **Animations**: Smooth resize animations
- **Autosave**: Remember divider positions
- **Min/max sizes**: Pane size constraints
- **Collapsible**: Hide/show panes
- **Priority**: Which pane shrinks first

These are stubbed for current use cases.
