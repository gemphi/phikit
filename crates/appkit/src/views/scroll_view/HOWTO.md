# ScrollView — How to Use

## Files
- `mod.rs` — `NSScrollView`, `NSClipView`, `NSScroller`, `NSBorderType`, scroll delegate

## Quick Start
```rust
use rust_appkit::views::scroll_view::{NSScrollView, NSBorderType};

// Create a scroll view
let mut scroll = NSScrollView::new(frame);
scroll.set_border_type(NSBorderType::NoBorder);

// Set the document (content) view — can be larger than the scroll frame
let mut content = NSView::new(NSRect::sized(800.0, 2000.0));
// ... populate content ...
scroll.set_document_view(content);

// Programmatic scrolling
scroll.scroll_to_point(NSPoint::new(0.0, 500.0));

// Flash scrollers for visual feedback
scroll.flash_scrollers();
```

## Key APIs
- `NSScrollView::new(frame)` — create with BezelBorder default
- `scroll.set_document_view(view)` — set scrollable content
- `scroll.document_view()` — get current document view
- `scroll.scroll_to_point(point)` — jump to position
- `scroll.scroll_to_visible(rect)` — ensure rect is visible
- `scroll.set_border_type(type)` — NoBorder, LineBorder, BezelBorder, GrooveBorder
- `scroll.view()` / `scroll.view_mut()` — underlying NSView

## Rules
- Document view is added as a subview automatically
- Scroll wheel events are handled by the internal delegate
- Scroll offset is clamped to `[0, content_size - visible_size]`
- `clips_to_bounds` is set to `true` automatically
- Both pixel and line deltas are supported (pre-converted at 20px/line)
