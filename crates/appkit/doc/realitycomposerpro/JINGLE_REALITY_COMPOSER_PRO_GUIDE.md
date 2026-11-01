# Reality Composer Pro for Jingle/RealityKit

> Complete step-by-step guide for building spatial experiences with Jingle, the Rust RealityKit implementation

---

## 🗺 Documentation Map

- [🏠 RealityKit API Reference](../realitykit/-realitykit.md)
- [🛠 Reality Composer Pro](JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) (This Guide)
- [🕶 visionOS Development](../visionos/-visionos.md)
- [📦 Jingle Kit Reference](../jingle/-jingle.md)

---

# Building 3D Content with Jingle Reality Composer

## Overview

Jingle provides a RealityKit-compatible API for building interactive 3D content in Rust. This guide shows you how to use the Reality Composer Pro workflow with Jingle's implementation.

With Jingle's Reality Composer tools, you can:
- Import and manage USDZ, OBJ, GLTF, and custom PHIRAY assets
- Design materials using the Shader Graph system
- Create animations and particle effects
- Build spatial experiences for visionOS and desktop
- Preview and optimize your content

![Reality Composer Pro Interface](./../assets/rcpro-heroshot.png)

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Project Structure](#project-structure)
3. [Importing Assets](#importing-assets)
4. [Working with the Shader Graph](#shader-graph)
5. [Building Materials](#building-materials)
6. [Entity Component System](#entity-component-system)
7. [Physics and Collisions](#physics-and-collisions)
8. [Animation System](#animation-system)
9. [API Comparison](#api-comparison)
10. [Best Practices](#best-practices)

---

## Getting Started

### Creating a New Jingle Project

```rust
// projects/botanist/app.rs
use jingle::{Scene, Entity, Transform, ARView, ARSession};
use jingle::material::{Material, ShaderGraphMaterial};

fn main() {
    // Create scene (equivalent to Reality Composer Pro project)
    let mut scene = Scene::new();
    
    // Add an entity
    let entity = scene.spawn(Transform::default());
    
    // Add material component
    scene.add_component(entity, Material::simple(Vec3::ONE));
}
```

### Reality Composer Pro vs Jingle Architecture

| Reality Composer Pro | Jingle Equivalent | Module |
|---------------------|-------------------|--------|
| Project (.rcproject) | `Scene` | `jingle::reality_kit::Scene` |
| Scene Graph | ECS World | `jingle::ecs::World` |
| Material Editor | `ShaderGraphMaterial` | `jingle::material::shader_graph` |
| Timeline | `AnimationSystem` | `jingle::animation` |
| Physics | `PhysicsWorld` | `jingle::physics` |

---

## Project Structure

### File Organization (Reality Composer Pro Style)

```
my_project/
├── assets/
│   ├── meshes/          # .obj, .gltf, .usdz files
│   ├── textures/        # .png, .jpg, .hdr files
│   ├── materials/       # .mat shader graphs
│   └── audio/           # .wav, .mp3 files
├── src/
│   ├── main.rs          # App entry point
│   ├── scene.rs         # Scene composition
│   └── materials.rs     # Material definitions
└── Cargo.toml
```

### Cargo.toml Dependencies

```toml
[dependencies]
jingle = { path = "../../phi/ext/jingle" }
glam = "0.25"
```

---

## Importing Assets

### Supported Formats

| Format | Type | Jingle Support |
|--------|------|----------------|
| USDZ | Scene/Model | ✅ Full |
| OBJ | Mesh | ✅ Full |
| GLTF | Scene/Model | ✅ Full |
| FBX | Scene/Animation | ⚠️ Partial |
| PHX | Point Cloud | ✅ Full |
| STL | Mesh | ✅ Full |
| DAE | Collada | ✅ Full |
| PNG | Texture | ✅ Full |
| JPEG | Texture | ✅ Full |
| HDR | Environment | ✅ Full |
| EXR | HDR Texture | ✅ Full |
| WAV | Audio | ✅ Full |
| MP3 | Audio | ✅ Full |

### Step-by-Step: Importing a USDZ Asset

```rust
use jingle::reality_kit::{Entity, Transform, ModelComponent};
use jingle::asset::{AssetManager, AssetType};

// 1. Create asset manager
let mut assets = AssetManager::new();

// 2. Load USDZ file
let handle = assets.load_file(
    "robot_head",
    AssetType::Mesh,
    "meshes/robot_head.usdz"
);

// 3. Create entity with model component
let entity = scene.spawn(
    Transform::from_position(glam::Vec3::new(0.0, 1.0, 0.0))
);

// 4. Add model component (RealityKit equivalent)
let model = ModelComponent::with_model(
    handle.0,  // mesh resource ID
    material.0 // material ID
);
scene.add_component(entity, model);
```

### Project Browser Equivalent (Rust)

```rust
// List all assets in project
fn list_project_assets(assets: &AssetManager) {
    println!("Project Browser:");
    println!("==================");
    
    for asset in assets.get_by_type(AssetType::Mesh) {
        println!("📦 {}", asset.name);
    }
    
    for asset in assets.get_by_type(AssetType::Texture) {
        println!("🖼️  {}", asset.name);
    }
    
    for asset in assets.get_by_type(AssetType::Material) {
        println!("🎨 {}", asset.name);
    }
}
```

---

## Shader Graph

### Overview

Jingle's Shader Graph provides node-based material editing, equivalent to Reality Composer Pro's Shader Graph editor.

### Built-in Shader Graph Nodes

| Node | Description | Reality Composer Pro Equivalent |
|------|-------------|--------------------------------|
| `Position` | Vertex position | Geometry > Position |
| `Normal` | Surface normal | Geometry > Normal |
| `UV` | Texture coordinates | Geometry > UV |
| `Time` | Animation time | Time |
| `Texture` | Texture sampling | Texture > Sample |
| `Add` | Vector addition | Math > Add |
| `Multiply` | Vector multiplication | Math > Multiply |
| `DotProduct` | Dot product | Math > Dot Product |
| `Fresnel` | Fresnel effect | Fresnel |
| `Noise` | Procedural noise | Noise |

### Creating a Material with Shader Graph

```rust
use jingle::material::shader_graph::{
    ShaderGraphMaterial, ShaderNode, MaterialBuilder
};

// Step 1: Create shader graph
let mut graph = ShaderGraphMaterial::new();

// Step 2: Add nodes
let position = graph.add_node(ShaderNode::Position);
let normal = graph.add_node(ShaderNode::Normal);
let time = graph.add_node(ShaderNode::Time);

// Step 3: Create output
let color = graph.add_node(ShaderNode::Color {
    color: Vec4::new(1.0, 0.5, 0.2, 1.0)
});

// Step 4: Connect to material output
graph.connect(color, graph.output_node);

// Step 5: Build material
let material = MaterialBuilder::from_graph(&graph);
```

### Fresnel Outline Effect (Reality Composer Pro Pattern)

```rust
use jingle::material::shader_graph::preset::fresnel_outline;

// Equivalent to: Creating Fresnel Outline in Shader Graph
let outline_material = fresnel_outline(
    Vec4::new(1.0, 1.0, 1.0, 1.0), // Color
    0.5,                            // Width
);
```

### Vertex Displacement Material

```rust
use jingle::material::shader_graph::preset::vertex_displacement;

// Equivalent to: Creating Vertex Displacement in Shader Graph
let displacement_material = vertex_displacement(
    Vec4::new(0.2, 0.5, 1.0, 1.0), // Base color
    0.1,                            // Displacement strength
    2.0,                            // Noise scale
);
```

---

## Building Materials

### PBR Material Configuration

| Property | Reality Composer Pro | Jingle API |
|----------|---------------------|------------|
| Base Color | Base Color | `material.base_color` |
| Metallic | Metallic | `material.metallic` |
| Roughness | Roughness | `material.roughness` |
| Normal | Normal Map | `material.normal_texture` |
| Emissive | Emissive Color | `material.emissive` |

### Code Example: Creating a PBR Material

```rust
use jingle::reality_kit::Material;
use jingle::asset::TextureResource;

// Load textures
let albedo = TextureResource::load("textures/metal_albedo.png");
let normal = TextureResource::load("textures/metal_normal.png");

// Create PBR material (Reality Composer Pro equivalent)
let material = Material::pbr()
    .with_base_color_texture(albedo)
    .with_normal_texture(normal)
    .with_metallic(0.8)
    .with_roughness(0.2);
```

### Simple Materials

```rust
// Diffuse material (matte)
let diffuse = Material::diffuse(Vec3::new(0.8, 0.3, 0.2));

// Emissive material (glow)
let emissive = Material::emissive(
    Vec3::new(1.0, 0.5, 0.0),
    2.0 // intensity
);
```

---

## Entity Component System

### ECS Architecture (RealityKit Pattern)

Reality Composer Pro's scene graph translates to Jingle's ECS:

| Reality Composer Pro | Jingle ECS |
|---------------------|------------|
| Entity | `Entity(u64)` |
| Transform Component | `Transform` |
| Model Component | `ModelComponent` |
| Physics Body | `RigidBody` + `CollisionShape` |
| Anchor | `AnchorComponent` |

### Working with Entities

```rust
use jingle::ecs::{Entity, World};
use jingle::reality_kit::{Transform, ModelComponent, PhysicsBodyComponent};

// Create world (equivalent to scene)
let mut world = World::new();

// Spawn entity
let robot = world.spawn((
    Transform::from_position(Vec3::new(0.0, 1.0, 0.0)),
    ModelComponent::with_model(mesh_id, material_id),
));

// Add physics
world.add_component(robot, PhysicsBodyComponent::dynamic());

// Query entities
for (entity, transform, model) in world.query::<(Transform, ModelComponent)>() {
    println!("Entity {:?} at position {:?}", entity, transform.position);
}
```

---

## Physics and Collisions

### Physics World Setup

```rust
use jingle::physics::{PhysicsWorld, RigidBody, Collider};

// Create physics world
let mut physics = PhysicsWorld::new();

// Create rigid body
let body = RigidBody::dynamic()
    .with_mass(1.0)
    .with_position(Vec3::new(0.0, 5.0, 0.0));

// Create collider
let collider = Collider::cuboid(0.5, 0.5, 0.5);

// Add to world
let body_handle = physics.add_rigid_body(body);
physics.add_collider(collider, body_handle);
```

### Collision Detection

```rust
use jingle::physics::CollisionEvent;

// Listen for collisions
for event in physics.collision_events() {
    match event {
        CollisionEvent::Started(entity_a, entity_b) => {
            println!("Collision between {:?} and {:?}", entity_a, entity_b);
        }
        CollisionEvent::Stopped(entity_a, entity_b) => {
            println!("Collision ended");
        }
    }
}
```

---

## Animation System

### Animation Timeline (Reality Composer Pro Equivalent)

```rust
use jingle::animation::{AnimationSystem, Keyframe, Interpolation};

// Create animation system
let mut animations = AnimationSystem::new();

// Define keyframes (equivalent to timeline)
let keyframes = vec![
    Keyframe::new(0.0, 0.0, Interpolation::Bezier),
    Keyframe::new(1.0, 10.0, Interpolation::Bezier),
    Keyframe::new(2.0, 0.0, Interpolation::Bezier),
];

// Create animation clip
let bounce = animations.create_clip("bounce", keyframes);

// Play on entity
animations.play(entity, bounce, 1.0); // 1x speed
```

### Procedural Animation

```rust
// Update animation in game loop
fn update_animations(
    time: f32,
    animations: &mut AnimationSystem,
    world: &mut World
) {
    // Animate all entities
    for (entity, transform) in world.query_mut::<Transform>() {
        // Bobbing animation
        transform.position.y = (time * 2.0).sin() * 0.1;
    }
}
```

---

## API Comparison: Reality Composer Pro vs Jingle

### Scene Management

```swift
// Reality Composer Pro / RealityKit (Swift)
let scene = try! Entity.load(named: "Scene")
let robot = scene.findEntity(named: "Robot")
robot.position = SIMD3<Float>(0, 1, 0)
```

```rust
// Jingle (Rust)
let mut scene = Scene::new();
let robot = scene.spawn(Transform::from_position(Vec3::new(0.0, 1.0, 0.0)));
scene.add_component(robot, ModelComponent::new());
```

### Materials

```swift
// RealityKit
var material = SimpleMaterial()
material.baseColor = .white
material.roughness = 0.5
entity.components[ModelComponent.self]?.materials = [material]
```

```rust
// Jingle
let material = Material::pbr()
    .with_base_color(Vec3::ONE)
    .with_roughness(0.5);
scene.add_component(entity, ModelComponent::with_material(material));
```

### Physics

```swift
// RealityKit
let body = PhysicsBodyComponent(
    massProperties: .init(mass: 1.0),
    material: .default,
    mode: .dynamic
)
entity.components.set(body)
```

```rust
// Jingle
let body = RigidBody::dynamic().with_mass(1.0);
let collider = Collider::cuboid(0.5, 0.5, 0.5);
let body_handle = physics.add_rigid_body(body);
physics.add_collider(collider, body_handle);
```

---

## Best Practices

### 1. Asset Management

```rust
// Use AssetManager for organized loading
let mut assets = AssetManager::new();

// Load manifest (Reality Composer Pro project equivalent)
let manifest = AssetManifest::default_manifest();
let handles = manifest.load_all(&mut assets);

// Check loading progress
while assets.loading_progress() < 1.0 {
    println!("Loading: {:.0}%", assets.loading_progress() * 100.0);
}
```

### 2. Performance Optimization

```rust
// Batch entity creation
let entities: Vec<Entity> = (0..100)
    .map(|i| scene.spawn(Transform::from_position(positions[i])))
    .collect();

// Use BVH8 for ray queries
let bvh = BVH8::from_scene(&scene);
let hit = bvh.intersect_ray(origin, direction);
```

### 3. Scene Organization

```rust
// Use hierarchy (Reality Composer Pro group equivalent)
let parent = scene.spawn(Transform::default());
let child = scene.spawn(Transform::default());
scene.set_parent(child, parent);

// Query by component type
for (entity, (transform, model)) in scene.query::<(Transform, ModelComponent)>() {
    // Process renderable entities
}
```

---

## Next Steps

- [Importing and Organizing Content](editor_projectbrowser/--editor_projectbrowser.md)
- [Designing Materials with Shader Graph](editor_shadergraph/--editor_shadergraph.md)
- [Building Materials](building-materials/--building-materials.md)
- [Creating Fresnel Effects](creating-a-fresnel-outline-effect-with-shader-graph/--creating-a-fresnel-outline-effect-with-shader-graph.md)
- [Vertex Displacement](creating-a-vertex-displacement-material-with-shader-graph/--creating-a-vertex-displacement-material-with-shader-graph.md)

---

## Resources

- [RealityKit Documentation](../realitykit/-realitykit.md)
- [Jingle Kit Reference](../../jingle/-jingle.md)
- [API Examples](../../../examples/)

*Generated for Jingle RealityKit Implementation v0.1.0*
