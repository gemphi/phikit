# NSScrollView — Scrollable Content Container

## Apple Header: `NSScrollView.h`

## Overview

`NSScrollView` provides scrolling for content larger than its visible area. Mirrors Apple's [`NSScrollView`](https://developer.apple.com/documentation/appkit/nsscrollview).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSScrollView Creation                        │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Components                                      │
│  ├─ DocumentView: The scrollable content (NSView)         │
│  ├─ ClipView: Visible portion mask                          │
│  ├─ Scroller (Horizontal): Bottom scrollbar                │
│  └─ Scroller (Vertical): Right scrollbar                   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Scrolling Logic                              │
│  ├─ Content size > Clip bounds? Show scrollers              │
│  ├─ Scroll wheel events → Update document origin            │
│  ├─ Clamp origin to valid range (0 to content - clip)     │
│  └─ Redraw document view at new position                    │
└─────────────────────────────────────────────────────────────┘
```

## Scroller Styles

| Style | Appearance | Use Case |
|-------|------------|----------|
| `Legacy` | Full scrollbar with arrows | Classic look |
| `Overlay` | Thin iOS-style overlay | Modern minimal |
| `Hidden` | No visible scroller | Touch/trackpad only |

## Connection to Other Modules

```
NSScrollView
├── Contains: DocumentView (NSView)
│   └── Your content goes here
├── Contains: NSScroller (2x)
├── Used by: NSTableView (lists)
├── Used by: NSOutlineView (trees)
├── Used by: NSTextView (documents)
└── Used by: Inspector panels (overflow content)
```

## Usage Examples

### Basic Scroll View
```rust
use rust_appkit::prelude::*;

let mut scroll_view = NSScrollView::new(NSRect::new(0.0, 0.0, 400.0, 300.0));

// Create content (larger than scroll view)
let mut content = NSView::new(NSRect::new(0.0, 0.0, 800.0, 600.0));
content.set_background_color(Some(NSColor::white()));

// Add some content to scroll
for i in 0..20 {
    let label = NSTextField::label(&format!("Item {}", i));
    label.view_mut().set_frame(NSRect::new(10.0, 580.0 - i as f32 * 30.0, 200.0, 22.0));
    content.add_subview(label.view_clone());
}

scroll_view.set_document_view(content.view_clone());
```

### Scroll to Position
```rust
// Programmatically scroll
scroll_view.scroll_to_point(NSPoint::new(100.0, 200.0));

// Scroll to make rect visible
scroll_view.scroll_rect_to_visible(NSRect::new(100.0, 200.0, 50.0, 50.0));
```

### With Content Insets
```rust
scroll_view.set_content_insets(NSEdgeInsets {
    top: 10.0,
    left: 10.0,
    bottom: 10.0,
    right: 10.0,
});
```

## Hierarchy Inspector Panel (rust-kit-pro)

```rust
// Hierarchy panel with many entities
let mut hierarchy_scroll = NSScrollView::new(NSRect::new(0.0, 0.0, 320.0, 400.0));

let hierarchy_content = create_entity_list(); // Tall view with all entities
hierarchy_scroll.set_document_view(hierarchy_content.view_clone());

// Auto-hide scrollers when content fits
hierarchy_scroll.set_autohides_scrollers(true);
```

## Scroll Event Handling

```
User scrolls (mouse wheel, trackpad, scrollbar)
        │
        ▼
NSScrollView captures scroll event
        │
        ▼
Calculate new document origin
        │
        ▼
Clamp to valid range (can't scroll past content)
        │
        ▼
Update scroller positions (thumb position)
        │
        ▼
Mark document view as needing display
        │
        ▼
Redraw with new visible rect
```

## Testing & Verification

```rust
let mut scroll = NSScrollView::new(NSRect::new(0.0, 0.0, 100.0, 100.0));
let content = NSView::new(NSRect::new(0.0, 0.0, 200.0, 200.0));

scroll.set_document_view(content.view_clone());

// Content is 2x scroll view size, so scrollers should show
assert!(scroll.has_vertical_scroller());
assert!(scroll.has_horizontal_scroller());

// Test scroll position
scroll.scroll_to_point(NSPoint::new(50.0, 50.0));
```

## File Structure

```
src/scroll_view/
└── mod.rs          # NSScrollView, NSScroller, NSClipView
```

## Missing from Apple's NSScrollView

- **Bounce/elasticity**: Overscroll rubber banding
- **Momentum scrolling**: Inertial scroll physics
- **Zooming**: Magnification support
- **Rulers**: Measurement rulers (NSRulerView)
- **Find bar**: Integrated search
- **Line scroll**: Arrow key scrolling amount

These are stubbed for current use cases.

## Common Patterns

| Pattern | Implementation |
|---------|----------------|
| **List** | NSTableView inside NSScrollView |
| **Document** | NSTextView as documentView |
| **Canvas** | Custom NSView with zoom/pan |
| **Inspector** | Stack of sections, auto-scroll |
