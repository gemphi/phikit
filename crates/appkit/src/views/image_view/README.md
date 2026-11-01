# NSImageView — Image Display

## Apple Header: `NSImage.h`

## Overview

`NSImageView` displays images in the UI. Mirrors Apple's [`NSImageView`](https://developer.apple.com/documentation/appkit/nsimageview).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    NSImageView Creation                      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│         Factory: imageViewWithImage(image)                   │
│  ├─ Creates NSView base                                    │
│  ├─ Stores NSImage reference                               │
│  ├─ Sets scaling mode (default: ProportionallyUpOrDown)  │
│  └─ Assigns NSImageViewDelegate for rendering              │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Rendering Cycle                         │
│  ├─ Get image size                                         │
│  ├─ Calculate target rect based on scaling mode            │
│  ├─ Draw image in target rect                              │
│  └─ Handle transparency (alpha channel)                    │
└─────────────────────────────────────────────────────────────┘
```

## Image Scaling Modes (NSImageScaling)

| Mode | Behavior | Use Case |
|------|----------|----------|
| `ProportionallyDown` | Scale down if too large, never up | Thumbnails |
| `AxesIndependently` | Stretch to fill (may distort) | Backgrounds |
| `None` | 1:1 pixel mapping, may clip | Pixel art |
| `ProportionallyUpOrDown` | Scale to fit, maintain aspect | Icons, previews |

## Connection to Other Modules

```
NSImageView
├── Contains: NSImage (via Arc)
├── Used by: NSButton (buttonWithImage)
├── Used by: NSCollectionView (item thumbnails)
├── Used by: NSToolbar (toolbar icons)
└── Used by: Asset Browser (rust-kit-pro)
```

## Usage Examples

### Basic Image View
```rust
use rust_appkit::prelude::*;
use rust_appkit::image_view::{NSImageView, NSImageScaling};

// Load image from file
let image = NSImage::from_file("icon.png").unwrap();

// Create image view with automatic sizing
let image_view = NSImageView::image_view_with_image(image);
// Frame automatically set to image size
```

### Custom Frame with Scaling
```rust
let image = svg_icons::get(IconId::Mesh).unwrap();

let mut image_view = NSImageView::new(NSRect::new(0.0, 0.0, 64.0, 64.0));
image_view.set_image(Some(image));
image_view.set_image_scaling(NSImageScaling::ProportionallyUpOrDown);
// Image centers and scales to fit 64x64, maintaining aspect ratio
```

### Fixed Size Thumbnail
```rust
let thumb = NSImageView::new(NSRect::new(0.0, 0.0, 128.0, 128.0));
thumb.set_image(Some(asset_preview));
thumb.set_image_scaling(NSImageScaling::ProportionallyDown);
// Never scales up, only down if image > 128x128
```

### Background Image (Stretch)
```rust
let bg = NSImageView::new(window_bounds);
bg.set_image(Some(wallpaper));
bg.set_image_scaling(NSImageScaling::AxesIndependently);
// Stretches to fill entire window
```

## Visual Scaling Examples

```
Image: 200x100 pixels
View: 100x100 pixels

ProportionallyDown:
┌─────────────────┐
│    ████████     │  ← Scaled to 100x50, centered
│    ████████     │
│                 │
└─────────────────┘

AxesIndependently:
┌─────────────────┐
│█████████████████│  ← Stretched to 100x100
│█████████████████│  ← Aspect ratio distorted
│█████████████████│
└─────────────────┘

None:
┌─────────────────┐
│████████        │  ← 1:1 pixels, clipped to 100x100
│████████        │
│                │
└─────────────────┘
```

## Integration with Asset Browser

```rust
// rust-kit-pro asset browser usage
fn create_asset_thumbnail(path: &str) -> NSView {
    let image = load_asset_preview(path);
    let mut thumb = NSImageView::new(NSRect::new(0.0, 0.0, 64.0, 64.0));
    thumb.set_image(Some(image));
    thumb.set_image_scaling(NSImageScaling::ProportionallyUpOrDown);
    thumb.view_clone()
}
```

## Testing & Verification

```rust
// Test factory method
let image = NSImage::new(100, 100);
let view = NSImageView::image_view_with_image(image);
// Frame should match image size

// Test scaling modes
let mut view = NSImageView::new(NSRect::new(0.0, 0.0, 50.0, 50.0));
view.set_image_scaling(NSImageScaling::None);
view.set_image_scaling(NSImageScaling::ProportionallyUpOrDown);
```

## File Structure

```
src/image_view/
└── mod.rs          # NSImageView, NSImageScaling
```

## Related Types

- **NSImage**: The actual image data (in `src/graphics/image.rs`)
- **NSBitmapImageRep**: Bitmap representation
- **NSCIImageRep**: Core Image representation

## Missing from Apple's NSImageView

- **Image alignment**: `imageAlignment` property
- **Editable**: `isEditable` for drag-and-drop
- **Animates**: Full GIF/WebP animation support
- **Layer contents placement**: CALayer integration

These are stubbed for current use cases.
