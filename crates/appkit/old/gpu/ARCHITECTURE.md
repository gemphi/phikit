# GPU Architecture — Rust Design Decisions

## Overview

The `gpu/` module handles all wgpu-based rendering: camera, mesh upload, shader pipelines,
and the 3D viewport. This document explains the Rust patterns and memory safety choices.

---

## Pattern: Camera as `Arc<Mutex<ViewportCameraController>>`

```rust
/// Editor camera — arcball orbit model.
/// Shared between input handler (writes) and render loop (reads).
///
/// Why not just pass by reference?
/// The render loop and input handler run in different call stacks.
/// A reference would require a lifetime that spans both — impossible
/// without unsafe. Arc<Mutex<_>> gives safe shared ownership.
pub struct ViewportCameraController {
    pub position: SCNVector3,   // 12 bytes — camera world position
    pub target: SCNVector3,     // 12 bytes — look-at point
    pub up: SCNVector3,         // 12 bytes — up vector (usually Y)
    pub fov_y: f32,             // 4 bytes — vertical field of view (radians)
    pub aspect: f32,            // 4 bytes — width/height ratio
    pub near: f32,              // 4 bytes — near clip plane
    pub far: f32,               // 4 bytes — far clip plane
    pub distance: f32,          // 4 bytes — orbit distance from target
    pub yaw: f32,               // 4 bytes — horizontal rotation (radians)
    pub pitch: f32,             // 4 bytes — vertical rotation (radians)
}
// Total: 64 bytes. Fits in a single cache line.
```

**Memory diagram — camera in the render pipeline:**

```
┌─────────────────────────────────────────────────────────────────┐
│                         FRAME RENDER                             │
│                                                                 │
│  1. Lock camera (< 1μs)                                        │
│     ┌─────────────────────────────────────────────────────┐    │
│     │ let cam = viewport.camera.lock().unwrap();          │    │
│     │ let vp_matrix = cam.view_projection_matrix();       │    │
│     │ // cam drops here — lock released                   │    │
│     └─────────────────────────────────────────────────────┘    │
│                                                                 │
│  2. Upload to GPU (no lock held)                                │
│     ┌─────────────────────────────────────────────────────┐    │
│     │ CameraUniforms {                                    │    │
│     │   view_proj: [[f32; 4]; 4],  // 64 bytes           │    │
│     │   camera_position: [f32; 4], // 16 bytes           │    │
│     │ }                                                   │    │
│     │ // Written to wgpu::Buffer via queue.write_buffer() │    │
│     └─────────────────────────────────────────────────────┘    │
│                                                                 │
│  3. Draw call (GPU reads uniform buffer)                        │
│     ┌─────────────────────────────────────────────────────┐    │
│     │ render_pass.set_bind_group(0, &camera_bind_group)   │    │
│     │ render_pass.draw(0..vertex_count, 0..1)             │    │
│     └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘

Key insight: The Mutex lock is held ONLY while reading 64 bytes.
The GPU upload and draw happen AFTER the lock is released.
This means input events can update the camera between frames
without blocking the render.
```

---

## Pattern: `#[repr(C)]` + `Pod` + `Zeroable` for GPU Structs

```rust
/// Camera uniforms uploaded to the GPU every frame.
/// 
/// #[repr(C)] — C memory layout (no Rust reordering).
///   Required because the GPU shader expects fields in this exact order.
///
/// Pod — "Plain Old Data". Guarantees no padding bytes are uninitialized.
///   Required by wgpu for safe buffer writes (no UB from reading padding).
///
/// Zeroable — Can be safely initialized to all zeros.
///   Lets us create a default without unsafe.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniforms {
    pub view_proj: [[f32; 4]; 4],       // 64 bytes — combined VP matrix
    pub camera_position: [f32; 4],      // 16 bytes — world pos (w=1.0)
}
// Total: 80 bytes, 16-byte aligned (GPU requirement)
```

**Memory layout (byte-exact):**

```
Offset  Size  Field
0x00    64    view_proj[0..4][0..4]  (column-major 4x4 matrix)
0x40    16    camera_position[0..4]  (xyz + w padding)
────────────
0x50    80 bytes total

GPU sees this as a uniform buffer bound at group 0, binding 0.
The vertex shader reads: `uniforms.view_proj * vertex_position`
```

**Why `Pod` matters:**
```rust
// WITHOUT Pod — unsafe, could have uninitialized padding bytes
let bytes: &[u8] = unsafe { std::slice::from_raw_parts(&uniforms as *const _ as *const u8, 80) };

// WITH Pod — safe, bytemuck guarantees all bytes are initialized
let bytes: &[u8] = bytemuck::bytes_of(&uniforms);  // no unsafe!
```

**Alternative: Manual byte serialization**
```rust
// Alternative — rejected (error-prone, verbose)
let mut buf = [0u8; 80];
buf[0..64].copy_from_slice(bytemuck::cast_slice(&view_proj));
buf[64..80].copy_from_slice(bytemuck::cast_slice(&position));
```

---

## Pattern: Exponential Zoom (No Negative Distance)

```rust
/// Zoom handler — exponential scaling prevents negative distance.
///
/// Linear zoom: distance += delta * speed
///   Problem: distance can go negative (camera flips through target)
///
/// Exponential zoom: distance *= 1.1^(delta * speed)
///   Always positive. Smooth at all scales. Feels natural.
pub fn handle_zoom(&mut self, delta: f32, sensitivity: f32) {
    // 1.1^x gives smooth exponential curve
    // Positive delta = zoom in (distance decreases)
    // Negative delta = zoom out (distance increases)
    let factor = 1.1f32.powf(delta * sensitivity);

    // Clamp to prevent:
    //   - Too close (< 0.1): geometry clips through near plane
    //   - Too far (> 1000): precision loss in depth buffer
    self.distance = (self.distance * factor).clamp(0.1, 1000.0);

    // Recompute camera position from yaw/pitch/distance
    self.update_from_arcball();
}
```

**Alternative: Linear zoom**
```rust
// Alternative — rejected (can go negative)
self.distance += delta * sensitivity;
// If user scrolls fast, distance becomes negative.
// Camera teleports behind the target. Disorienting.
```

---

## Pattern: Arcball Orbit (Spherical Coordinates)

```rust
/// Recompute camera position from spherical coordinates.
/// The camera orbits around `target` at `distance`, controlled by yaw/pitch.
///
/// Math:
///   x = target.x + distance * cos(pitch) * sin(yaw)
///   y = target.y + distance * sin(pitch)
///   z = target.z + distance * cos(pitch) * cos(yaw)
///
/// This is a point on a sphere of radius `distance` centered at `target`.
pub fn update_from_arcball(&mut self) {
    // Clamp pitch to prevent gimbal lock at poles
    // ±89° instead of ±90° avoids the singularity
    self.pitch = self.pitch.clamp(
        -89.0_f32.to_radians(),
        89.0_f32.to_radians(),
    );

    let cos_pitch = self.pitch.cos();
    self.position.x = self.target.x + self.distance * cos_pitch * self.yaw.sin();
    self.position.y = self.target.y + self.distance * self.pitch.sin();
    self.position.z = self.target.z + self.distance * cos_pitch * self.yaw.cos();
}
```

**Alternative: Quaternion orbit**
```rust
// Alternative — viable, more complex
// Store orientation as a quaternion, apply incremental rotations.
// Pro: No gimbal lock at all. Con: More math, harder to debug,
// harder to snap to axis-aligned views (front/top/right).
// We chose spherical because axis snapping is trivial (set yaw/pitch to known values).
```

---

## GPU Resource Ownership

| Resource | Rust Type | Lifetime | Why |
|----------|-----------|----------|-----|
| wgpu::Device | Owned by `GpuRenderer` | App lifetime | Single GPU context |
| wgpu::Queue | Owned by `GpuRenderer` | App lifetime | Command submission |
| Vertex buffers | `Vec<wgpu::Buffer>` in MeshCache | Per-mesh | Freed when mesh unloaded |
| Camera uniform buffer | `wgpu::Buffer` in RenderPass | App lifetime | Rewritten every frame |
| Depth texture | `wgpu::Texture` in RenderPass | Recreated on resize | Size-dependent |
| Pipelines | `wgpu::RenderPipeline` | App lifetime | Compiled once |

All GPU resources are owned by Rust structs. When the struct drops, the resource is freed.
No manual `destroy()` calls. No resource leaks. RAII handles everything.
