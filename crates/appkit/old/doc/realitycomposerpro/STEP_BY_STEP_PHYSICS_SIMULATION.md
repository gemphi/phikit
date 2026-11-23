# Step-by-Step: Physics Simulation

> Guide to physics simulation and viewport visualization

---

## 🗺 Documentation Map

- [← Back to Reality Composer Pro Guide](./JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [⚙️ Physics API](../../../../../phi/crates/rust-reality-kit/src/physics.rs)
- [🔧 Viewport Integration](../../../../../phi/crates/rust-kit-pro/src/physics_viewport.rs)
- [🏠 RealityKit Home](../realitykit/-realitykit.md)

---

## 📊 Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| **Physics Viewport** | ✅ Working | Real-time debug visualization |
| **Colliders** | ✅ Working | Box, Sphere, Capsule, Cylinder |
| **Rigid Bodies** | ✅ Working | Dynamic + kinematic |
| **Joints** | ✅ Working | Fixed, Hinge, Ball-socket, Spring |
| **Debug Draw** | ✅ Working | Wireframe overlays |
| **Simulation Control** | ✅ Working | Play, pause, step |
| **Velocity Vectors** | ✅ Working | Visual force vectors |
| **Physics ECS** | ✅ Working | Component-based system |
| **Gravity Control** | ✅ Working | Configurable gravity |
| **Complex Shapes** | ⚠️ Partial | Mesh colliders planned |
| **Soft Body** | ❌ Not Started | Planned for future |

**Last Updated:** 2024-01-20  
**App Status:** ✅ Running & Ready  
**Crate:** `phi/crates/rust-kit-pro/src/physics_viewport.rs`

---

## Overview

rust-kit-pro includes a physics simulation system with real-time viewport visualization for colliders, joints, and velocity vectors.

---

## Features

| Feature | Description |
|---------|-------------|
| **Colliders** | Box, Sphere, Capsule, Cylinder shapes |
| **Rigid Bodies** | Dynamic and kinematic bodies |
| **Joints** | Fixed, Hinge, Ball-socket, Spring |
| **Debug Visualization** | Wireframe overlays in viewport |
| **Simulation Control** | Play, pause, step physics |

---

## Step 1: Add Physics Components

```rust
use reality_kit::physics::{RigidBodyComponent, ColliderComponent, ColliderShape};
use reality_foundation::World;
use glam::Vec3;

fn add_physics_object(world: &mut World, entity: Entity) {
    // Add rigid body
    let body = RigidBodyComponent::new(1.0); // 1kg mass
    world.insert(entity, body);
    
    // Add collider
    let collider = ColliderComponent::sphere(0.5); // radius 0.5m
    world.insert(entity, collider);
}
```

---

## Step 2: Collider Shapes

```rust
// Box collider
let box_collider = ColliderComponent::box(Vec3::new(1.0, 1.0, 1.0));

// Sphere collider
let sphere_collider = ColliderComponent::sphere(0.5);

// Capsule collider
let capsule_collider = ColliderComponent::capsule(0.3, 2.0); // radius, height

// Cylinder collider
let cylinder_collider = ColliderComponent::cylinder(0.5, 1.0); // radius, height
```

---

## Step 3: Create Joints

```rust
use reality_kit::physics::{JointComponent, JointType};

fn create_joint(world: &mut World, entity_a: Entity, entity_b: Entity) {
    let joint = JointComponent::new(
        entity_a,
        entity_b,
        JointType::Fixed, // or Hinge, BallSocket, Spring
    );
    
    let joint_entity = world.spawn();
    world.insert(joint_entity, joint);
}
```

**Joint Types:**
- `Fixed` - No movement between objects
- `Hinge { min_angle, max_angle }` - Rotation around axis
- `BallSocket` - Free rotation
- `Spring { stiffness, damping }` - Elastic connection

---

## Step 4: Enable Physics Viewport

```rust
use rust_kit_pro::physics_viewport::PhysicsViewport;

fn setup_viewport() {
    let mut viewport = PhysicsViewport::new();
    
    // Enable visualization
    viewport.show_colliders = true;
    viewport.show_joints = true;
    viewport.show_velocity = true;
}
```

---

## Step 5: Physics Controls

| Button | Action |
|--------|--------|
| ▶ Play | Start simulation |
| ⏸ Pause | Freeze simulation |
| ⏭ Step | Advance one frame |
| Enabled | Toggle simulation on/off |

```rust
// Programmatic control
viewport.simulation_enabled = true;
viewport.paused = false;
viewport.time_step = 1.0 / 60.0; // 60 FPS
viewport.gravity = Vec3::new(0.0, -9.81, 0.0);
```

---

## Step 6: Visual Debugging

```rust
// In viewport draw loop
viewport.draw_in_viewport(
    &painter,
    &world,
    view_projection_matrix,
    viewport_rect,
);
```

**Visual Elements:**
- 🟢 Green wireframes = Colliders
- 🔗 Magenta lines = Joints
- 🔵 Blue arrows = Velocity vectors
- 🟠 Yellow = Trigger colliders

---

## Full Example

```rust
use reality_kit::physics::*;
use reality_foundation::World;
use rust_kit_pro::physics_viewport::PhysicsViewport;

fn setup_physics_scene(world: &mut World) {
    // Create floor
    let floor = world.spawn();
    let floor_body = RigidBodyComponent::new_kinematic();
    let floor_collider = ColliderComponent::box(Vec3::new(10.0, 0.1, 10.0));
    world.insert(floor, (floor_body, floor_collider));
    
    // Create falling box
    let box_entity = world.spawn();
    let mut box_body = RigidBodyComponent::new(1.0);
    box_body.gravity_scale = 1.0;
    let box_collider = ColliderComponent::box(Vec3::new(1.0, 1.0, 1.0));
    world.insert(box_entity, (box_body, box_collider));
    
    // Set initial position
    if let Some(mut transform) = world.get::<Transform>(box_entity) {
        transform.translation = Vec3::new(0.0, 5.0, 0.0);
    }
}

fn update_physics(viewport: &mut PhysicsViewport, world: &mut World, dt: f32) {
    viewport.simulate(world, dt);
}
```

---

## Next Steps

- [Animation Editor](STEP_BY_STEP_ANIMATION_EDITOR.md)
- [AR Preview](STEP_BY_STEP_AR_PREVIEW.md)

---

*Generated for rust-kit-pro v0.1.0*
