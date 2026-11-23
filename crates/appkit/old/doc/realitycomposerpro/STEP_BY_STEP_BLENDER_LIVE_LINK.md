# Step-by-Step: Blender Live Link

> Real-time synchronization between rust-kit-pro and Blender

---

## 🗺 Documentation Map

- [← Back to Reality Composer Pro Guide](./JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🔗 Live Link API](../../../../../phi/crates/rust-kit-pro/src/blender_live_link.rs)
- [🏠 RealityKit Home](../realitykit/-realitykit.md)

---

## 📊 Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| **TCP Protocol** | ✅ Working | JSON message protocol over TCP |
| **Server Mode** | ✅ Working | rust-kit-pro waits for Blender |
| **Client Mode** | ✅ Working | rust-kit-pro connects to Blender |
| **Transform Sync** | ✅ Working | Position, rotation, scale |
| **Mesh Sync** | ✅ Working | Send/receive mesh geometry |
| **Scene Sync** | ✅ Working | Full world synchronization |
| **Materials** | ⚠️ Partial | Basic color only |
| **Camera Sync** | ⚠️ Partial | Viewport matching planned |
| **UI Panel** | ✅ Working | Live Link panel in editor |
| **Blender Addon** | ❌ Not Started | Python addon for Blender side |

**Last Updated:** 2024-01-20  
**App Status:** ✅ Running & Ready (rust-kit-pro side)  
**Crate:** `phi/crates/rust-kit-pro/src/blender_live_link.rs`

---

## Overview

Blender Live Link enables real-time bidirectional sync between rust-kit-pro and Blender 3.x. Changes in either application are instantly reflected in the other.

---

## Features

| Feature | Description |
|---------|-------------|
| **Mesh Sync** | Send/receive mesh geometry |
| **Transform** | Position, rotation, scale sync |
| **Materials** | Material properties sync |
| **Camera** | Viewport camera matching |
| **Hierarchy** | Scene structure sync |

---

## Step 1: Start Server Mode

**In rust-kit-pro:**
1. Open Blender Live Link panel
2. Select "Server" mode
3. Set port (default: 12000)
4. Click "▶ Start Server"

```rust
use rust_kit_pro::blender_live_link::{BlenderLiveLink, LiveLinkMode};

fn start_server() {
    let mut live_link = BlenderLiveLink::new(LiveLinkMode::Server);
    live_link.start_server(12000).unwrap();
    println!("Server listening on port 12000");
}
```

---

## Step 2: Connect from Blender

**In Blender:**
1. Install rust-kit-pro Live Link addon
2. Go to View3D > Sidebar > Live Link
3. Set host: `127.0.0.1`, port: `12000`
4. Click "Connect"

---

## Step 3: Enable Auto-Sync

```rust
// Enable automatic sync (100ms interval)
live_link.set_auto_sync(true);
live_link.set_sync_interval(100);
```

**Or use UI:**
- Check "Auto-sync" checkbox
- Set interval (milliseconds)

---

## Step 4: Manual Sync

```rust
use reality_foundation::World;

fn manual_sync(live_link: &mut BlenderLiveLink, world: &World) {
    // Sync all objects
    live_link.sync_world(world);
    
    // Flush pending updates
    live_link.flush();
}
```

**UI Buttons:**
- `🔄 Sync All` - Sync entire scene
- `📤 Flush` - Send pending updates

---

## Step 5: Sync Individual Object

```rust
fn sync_object(
    live_link: &mut BlenderLiveLink,
    entity: Entity,
    transform: &Transform,
    mesh: &MeshComponent,
    name: &str
) {
    // Sync transform
    live_link.sync_entity_transform(entity, transform, name);
    
    // Sync mesh
    live_link.sync_mesh(entity, mesh, name);
    
    // Flush immediately
    live_link.flush();
}
```

---

## Connection Modes

| Mode | Use Case |
|------|----------|
| **Server** | rust-kit-pro waits for Blender to connect |
| **Client** | rust-kit-pro connects to Blender addon |

---

## Protocol Messages

```rust
use rust_kit_pro::blender_live_link::LiveLinkMessage;

// Transform update
let msg = LiveLinkMessage::TransformUpdate {
    object_name: "Cube".to_string(),
    position: [0.0, 0.0, 0.0],
    rotation: [0.0, 0.0, 0.0, 1.0], // quaternion
    scale: [1.0, 1.0, 1.0],
    sequence: 1,
};
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Connection refused | Check firewall, ensure port is open |
| Sync lag | Reduce sync interval |
| Objects not appearing | Verify object names match |
| Blender crashes | Update to Blender 3.4+ |

---

## Next Steps

- [Animation Editor](STEP_BY_STEP_ANIMATION_EDITOR.md)
- [Shader Graph Guide](STEP_BY_STEP_SHADER_GRAPH_GUIDE.md)

---

*Generated for rust-kit-pro v0.1.0*
