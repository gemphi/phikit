# ECS — Entity Component System

## Overview

`ecs` module provides data-oriented entity component system. Optimized for spatial computing and 3D scene management.

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 ECS Architecture                               │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Core Types                                        │
│  ├─ Entity: Unique ID (u64)                                   │
│  ├─ Component: Plain data (Transform, Mesh, etc)            │
│  └─ System: Logic that processes entities with components │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 World Storage                                  │
│  ├─ Sparse Set: Entity → Components                           │
│  ├─ Archetype: Entities with same component set               │
│  └─ Query: Filter entities by component requirements         │
└─────────────────────────────────────────────────────────────┘
```

## Core Concepts

| Concept | Description |
|---------|-------------|
| **Entity** | Unique identifier, just an ID |
| **Component** | Data container (no logic) |
| **System** | Logic that queries and updates |
| **World** | Container for all entities |
| **Query** | Filter entities by components |

## Connection to Other Modules

```
ECS
├── Used by: reality-foundation (scene management)
├── Used by: reality-kit (3D objects)
├── Used by: rust-kit-pro (editor scene)
└── Integrates with: timeline (animation)
```

## Usage Examples

### Define Components
```rust
use rust_appkit::ecs::*;

#[derive(Component, Debug, Clone)]
struct Transform {
    position: SCNVector3,
    rotation: SCNQuaternion,
    scale: SCNVector3,
}

#[derive(Component)]
struct MeshRenderer {
    mesh_id: MeshId,
    material_id: MaterialId,
}

#[derive(Component)]
struct PointLight {
    color: SCNVector3,
    intensity: f32,
    range: f32,
}
```

### Create World and Entities
```rust
let mut world = World::new();

// Spawn entity with components
let player = world.spawn()
    .insert(Transform {
        position: SCNVector3::zero(),
        rotation: SCNQuaternion::identity(),
        scale: SCNVector3::one(),
    })
    .insert(MeshRenderer {
        mesh_id: load_mesh("player.obj"),
        material_id: default_material(),
    })
    .id();

// Spawn light
let light = world.spawn()
    .insert(Transform {
        position: SCNVector3::new(0.0, 10.0, 0.0),
        rotation: SCNQuaternion::identity(),
        scale: SCNVector3::one(),
    })
    .insert(PointLight {
        color: SCNVector3::new(1.0, 0.9, 0.8),
        intensity: 100.0,
        range: 20.0,
    })
    .id();
```

### Query Systems
```rust
fn update_transforms(world: &mut World, dt: f32) {
    // Query all entities with Transform
    for (entity, mut transform) in world.query::<&mut Transform>() {
        // Apply some logic
        transform.position.y += 9.8 * dt * dt; // Gravity
    }
}

fn render_system(world: &World) {
    // Query entities with both Transform and MeshRenderer
    for (entity, (transform, mesh)) in world.query::<(&Transform, &MeshRenderer)>() {
        draw_mesh(&mesh.mesh_id, transform.to_matrix());
    }
}

fn light_system(world: &World) {
    // Query lights
    for (entity, (transform, light)) in world.query::<(&Transform, &PointLight)>() {
        set_light_position(entity, transform.position);
        set_light_color(entity, light.color, light.intensity);
    }
}
```

### Modify Components
```rust
// Get mutable reference
if let Some(mut transform) = world.get::<Transform>(player) {
    transform.position.x += 1.0;
}

// Remove component
world.entity_mut(player).remove::<PointLight>();

// Add component
world.entity_mut(player).insert(velocity_component);

// Despawn entity
world.despawn(player);
```

## ECS vs OOP

| OOP (Classes) | ECS (Components) |
|---------------|------------------|
| `class Player` inherits | `Player` = Entity + Components |
| Tight coupling | Loose composition |
| Hard to modify | Flexible at runtime |
| Cache misses | Data-oriented layout |

## Testing & Verification

```rust
let mut world = World::new();

// Create entity
let e = world.spawn()
    .insert(Transform::default())
    .insert(MeshRenderer { mesh_id: 1, material_id: 1 })
    .id();

// Query it back
let mut count = 0;
for (_, (transform, mesh)) in world.query::<(&Transform, &MeshRenderer)>() {
    count += 1;
}
assert_eq!(count, 1);

// Remove component
world.entity_mut(e).remove::<MeshRenderer>();

// Now only Transform
let mut count = 0;
for _ in world.query::<&Transform>() {
    count += 1;
}
assert_eq!(count, 1);

// MeshRenderer query should be empty
let mut count = 0;
for _ in world.query::<&MeshRenderer>() {
    count += 1;
}
assert_eq!(count, 0);
```

## File Structure

```
src/ecs/
├── mod.rs          # World, Entity, Component trait
├── query.rs        # Query, QueryFilter
├── system.rs       # System, SystemStage
└── storage.rs      # ComponentStorage, Archetype
```

## Missing from ECS

- **Reflection**: Runtime component inspection
- **Serialization**: Save/load entities
- **Networking**: Replication
- **Jobs**: Parallel system execution
- **Events**: Entity events/observers
- **Resources**: Global non-entity data

These are stubbed for current use cases.

## Related

- **reality-foundation**: Higher-level ECS integration
- **reality-kit**: 3D rendering with ECS
- **timeline**: Animates component values
