# Step-by-Step: AR Preview

> Preview ARKit camera feed and tracked anchors in rust-kit-pro

---

## 🗺 Documentation Map

- [← Back to Reality Composer Pro Guide](./JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [📱 ARKit API](../../../../../phi/crates/rust-ios-ar/src/)
- [🔍 AR Preview Module](../../../../../phi/crates/rust-kit-pro/src/ar_preview.rs)
- [🏠 RealityKit Home](../realitykit/-realitykit.md)

---

## 📊 Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| **Camera Feed** | ✅ Working | Live ARKit camera view |
| **Plane Detection** | ✅ Working | Floor, wall, ceiling visualization |
| **Image Anchors** | ✅ Working | 2D image tracking |
| **World Origin** | ✅ Working | Coordinate axes display |
| **Tracking State** | ✅ Working | Quality indicator overlay |
| **Simulation Mode** | ✅ Working | Fake AR data for testing on macOS |
| **Anchor Visualization** | ⚠️ Partial | Basic wireframe only |
| **Real Device Streaming** | ❌ Not Started | Requires iOS device connection |

**Last Updated:** 2024-01-20  
**App Status:** ✅ Running & Ready (simulation mode)  
**Crate:** `phi/crates/rust-kit-pro/src/ar_preview.rs`

---

## Overview

AR Preview in rust-kit-pro displays the ARKit camera feed and overlays detected planes, anchors, and tracking information directly in the editor viewport.

**Note:** Requires iOS device with ARKit support.

---

## Features

| Feature | Description |
|---------|-------------|
| **Camera Feed** | Live ARKit camera view |
| **Plane Detection** | Visualize floor, walls, ceiling |
| **Anchor Tracking** | Track objects in 3D space |
| **World Origin** | Visual coordinate system |
| **Tracking State** | Quality indicator |

---

## Step 1: Start AR Session

```rust
use rust_kit_pro::ar_preview::ARPreview;

fn start_ar() -> Result<(), String> {
    let mut preview = ARPreview::new();
    preview.init()?;
    preview.show();
    
    println!("AR session started");
    Ok(())
}
```

---

## Step 2: Configure AR Preview

```rust
fn configure_preview(preview: &mut ARPreview) {
    // Show/hide elements
    preview.show_camera_feed = true;
    preview.show_planes = true;
    preview.show_images = true;
    preview.show_feature_points = false;
    preview.show_world_origin = true;
    
    // Set preview quality
    preview.preview_quality = 0.5; // 0.0 - 1.0
}
```

---

## Step 3: Control Panel

| Setting | Description |
|---------|-------------|
| ▶ Start AR | Begin ARKit session |
| ⏸ Pause | Pause tracking |
| ⏹ Stop | End session |
| Camera Feed | Show/hide camera |
| Planes | Show detected planes |
| Images | Show image anchors |
| Feature Points | Show tracking points |
| World Origin | Show coordinate axes |

---

## Step 4: Viewport Overlay

```rust
fn draw_ar_overlay(
    preview: &ARPreview,
    painter: &Painter,
    world: &World,
    view_proj: Mat4,
    viewport_rect: Rect,
) {
    // Draw AR visualization in viewport
    preview.draw_in_viewport(
        painter,
        world,
        view_proj,
        viewport_rect,
    );
}
```

---

## Step 5: Access Tracked Data

```rust
fn get_tracked_data(preview: &ARPreview) {
    // Get planes
    for plane in &preview.planes {
        println!("Plane at {:?} - {:?}", 
            plane.center(), 
            plane.classification()
        );
    }
    
    // Get tracking state
    if let Some(frame) = &preview.last_frame {
        if let Some(camera) = frame.camera() {
            println!("Tracking: {:?}", camera.tracking_state());
        }
    }
}
```

---

## Plane Classifications

| Color | Type |
|-------|------|
| 🔴 Red | Floor |
| 🟢 Green | Wall |
| 🔵 Blue | Ceiling |
| 🟡 Yellow | Table/Seat |

---

## Tracking States

| State | Icon | Description |
|-------|------|-------------|
| Normal | 🟢 | Good tracking |
| Limited | 🟡 | Some drift expected |
| Not Available | ⚪ | Tracking lost |

---

## Requirements

- iOS device with A12 chip or later
- iOS 13.0+
- Camera permissions enabled

---

## Simulation Mode

On non-iOS platforms, AR Preview simulates AR data:

```rust
// Fake planes for testing
preview.simulate_ar_data();
```

---

## Full Example

```rust
use rust_kit_pro::ar_preview::ARPreview;

fn setup_ar_preview() -> Result<(), String> {
    let mut ar = ARPreview::new();
    
    // Initialize
    ar.init()?;
    
    // Configure
    ar.show_camera_feed = true;
    ar.show_planes = true;
    ar.show_world_origin = true;
    
    // Show
    ar.show();
    
    Ok(())
}

fn update(ar: &mut ARPreview) {
    // Update AR data
    ar.update();
    
    // Check status
    match ar.session_state {
        ARSessionState::Running => println!("AR active"),
        _ => println!("AR not running"),
    }
}
```

---

## Next Steps

- [Room Plan Capture](STEP_BY_STEP_ROOM_CAPTURE.md)
- [Physics Simulation](STEP_BY_STEP_PHYSICS_SIMULATION.md)

---

*Generated for rust-kit-pro v0.1.0*
