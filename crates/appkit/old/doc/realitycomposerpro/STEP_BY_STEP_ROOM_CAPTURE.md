# Step-by-Step: Room Plan Capture

> Scan indoor spaces and generate floor plans with ARKit

---

## 🗺 Documentation Map

- [← Back to Reality Composer Pro Guide](./JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🏠 Room Plan API](../../../../../phi/crates/rust-ios-ar/src/arkit/room_plan.rs)
- [🏠 RealityKit Home](../realitykit/-realitykit.md)

---

## Overview

Room Plan Capture uses ARKit to scan indoor spaces and generate:
- Floor plans with accurate dimensions
- 3D room geometry
- Door and window locations
- Detected furniture and objects
- Export to USDZ, USD, or JSON

**Note:** Requires iOS 16+ device with LiDAR (iPhone 12 Pro or later).

---

## 📊 Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| **ARRoomCaptureSession** | ✅ Working | Session management API |
| **Configuration** | ✅ Working | Capture settings |
| **Surface Detection** | ✅ Working | Floors, walls, ceilings |
| **Openings** | ✅ Working | Doors, windows, fireplaces |
| **Furniture Detection** | ✅ Working | Object categories |
| **Export USDZ** | ✅ Working | 3D model export |
| **Export JSON** | ✅ Working | Dimensions data |
| **Progress Tracking** | ✅ Working | Scan progress 0-100% |
| **Real Device Capture** | ❌ Not Started | Requires iOS 16+ device |
| **Floor Plan Export** | ⚠️ Partial | Planned for future |

**Last Updated:** 2024-01-20  
**App Status:** ✅ API Ready (simulation mode)  
**Crate:** `phi/crates/rust-ios-ar/src/arkit/room_plan.rs`

---

## Features

| Feature | Description |
|---------|-------------|
| **Room Scanning** | Walk around to capture space |
| **Surface Detection** | Floors, walls, ceilings |
| **Openings** | Doors, windows, fireplaces |
| **Furniture Detection** | Sofas, tables, beds, etc. |
| **Export** | USDZ, USD, JSON formats |

---

## Step 1: Create Capture Session

```rust
use rust_ios_ar::arkit::room_plan::ARRoomCaptureSession;

fn create_session() -> ARRoomCaptureSession {
    let mut session = ARRoomCaptureSession::new();
    
    // Configure
    session.configuration.capture_geometry = true;
    session.configuration.detect_elements = true;
    session.configuration.capture_furniture = true;
    session.configuration.export_format = ARRoomExportFormat::Usdz;
    
    session
}
```

---

## Step 2: Start Capturing

```rust
fn start_capture(session: &mut ARRoomCaptureSession) -> Result<(), ARRoomCaptureError> {
    session.start_capturing()?;
    
    println!("Room capture started");
    println!("Walk around the room slowly...");
    
    Ok(())
}
```

---

## Step 3: Track Progress

```rust
fn monitor_progress(session: &ARRoomCaptureSession) {
    // Progress 0.0 - 1.0
    println!("Progress: {:.0}%", session.progress * 100.0);
    
    // Captured rooms
    println!("Rooms captured: {}", session.room_count());
    
    // Total area
    println!("Total area: {:.1} m²", session.total_captured_area());
}
```

---

## Step 4: Handle Detected Surfaces

```rust
use rust_ios_ar::arkit::room_plan::ARRoomSurface;

impl ARRoomCaptureDelegate for MyDelegate {
    fn did_detect_surface(&mut self, surface: ARRoomSurface) {
        match surface.surface_type {
            ARSurfaceType::Floor => println!("Floor detected"),
            ARSurfaceType::Wall => println!("Wall detected"),
            ARSurfaceType::Ceiling => println!("Ceiling detected"),
        }
        
        println!("  Dimensions: {:?}", surface.dimensions);
        println!("  Position: {:?}", surface.transform);
    }
}
```

---

## Step 5: Finish and Export

```rust
fn finish_capture(session: &mut ARRoomCaptureSession) {
    // Stop capturing
    if let Some(room) = session.stop_capturing() {
        println!("Room captured: {}", room.name);
        println!("  Category: {:?}", room.category);
        println!("  Area: {:.1} m²", room.dimensions.area);
        println!("  Walls: {}", room.walls.len());
        println!("  Openings: {}", room.openings.len());
        println!("  Furniture: {}", room.furniture.len());
        
        // Export
        let export_path = "captured_room.usdz";
        if let Err(e) = session.export_room(&room, export_path) {
            eprintln!("Export failed: {:?}", e);
        } else {
            println!("Exported to {}", export_path);
        }
    }
}
```

---

## Room Categories

| Category | Description |
|----------|-------------|
| Bedroom | Sleeping area |
| LivingRoom | Main living space |
| Kitchen | Cooking area |
| Bathroom | Washing area |
| DiningRoom | Eating area |
| Office | Work space |
| Hallway | Connecting space |
| Garage | Vehicle storage |
| Storage | Storage area |

---

## Export Formats

| Format | Extension | Use Case |
|--------|-----------|----------|
| USDZ | `.usdz` | AR Quick Look, Reality Composer |
| USD | `.usd` | Editing in 3D software |
| Floor Plan | `.png` | 2D layout view |
| JSON | `.json` | Data analysis, dimensions |

---

## Best Practices

1. **Lighting** - Ensure room is well-lit
2. **Movement** - Walk slowly and cover all areas
3. **Walls** - Get close to walls for accurate detection
4. **Obstacles** - Remove or note large moving objects
5. **Time** - Allow 30-60 seconds for complete scan

---

## Furniture Detection

Detected furniture types:
- 🛋️ Sofa/Couch
- 🪑 Chair
- 🪜 Table
- 🛏️ Bed
- 🖥️ Desk
- 🗄️ Cabinet
- ⚡ Appliance
- 🪴 Plant
- 💡 Lighting
- 📺 TV/Monitor
- 📦 Storage

---

## Full Example

```rust
use rust_ios_ar::arkit::room_plan::*;

fn capture_room() -> Result<(), ARRoomCaptureError> {
    // Create session
    let mut session = ARRoomCaptureSession::new();
    session.configuration.export_format = ARRoomExportFormat::Usdz;
    
    // Start
    session.start_capturing()?;
    
    // Wait for capture (would be async in real app)
    std::thread::sleep(std::time::Duration::from_secs(60));
    
    // Finish
    if let Some(room) = session.stop_capturing() {
        println!("Captured: {:?}", room.category);
        
        // Export
        session.export_room(&room, "room.usdz")?;
        println!("Exported to room.usdz");
    }
    
    Ok(())
}
```

---

## Next Steps

- [AR Preview](STEP_BY_STEP_AR_PREVIEW.md)
- [Asset Import Guide](STEP_BY_STEP_ASSET_IMPORT_GUIDE.md)

---

*Generated for rust-ios-ar v0.1.0*
