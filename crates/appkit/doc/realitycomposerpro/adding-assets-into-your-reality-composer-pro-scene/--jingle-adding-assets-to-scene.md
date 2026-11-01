> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [ADDING ASSETS TO YOUR JINGLE SCENE](./--jingle-adding-assets-to-scene.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🕶 visionOS Home](../../../visionos/-visionos.md)

---

# Adding assets to your Jingle Reality Composer scene

Import assets to design high-performance Jingle scenes for your Rust applications.

## Overview

Jingle uses the `Scene` structure to organize and store your spatial content. Each project typically starts with a primary scene that contains a collection of entities. For example, a robot exploration scene might contain the robot 3D model, botanical assets, and spatial audio sources.

With Jingle's implementation, you can compose these assets in Rust code or load them from pre-defined manifests, allowing your app to manage complex 3D environments efficiently.

![Jingle Scene Interface](./../../assets/rcpro-biplanewindow.png)

## Designing Scenes in Rust

In Jingle, you create and manage scenes using the `Scene` API. You can have multiple scenes in your project, each driving a different `RealityView` or application state.

```rust
use jingle::reality_kit::Scene;

fn create_main_scene() -> Scene {
    // Initialize a new scene (Reality Composer Pro equivalent)
    let mut scene = Scene::new();
    
    // Add default entities or load from manifest
    println!("New scene created with Root entity: {:?}", scene.root());
    
    scene
}
```

## Adding Assets to Your Project

In Jingle, designing scenes involves importing assets into your `AssetManager`. You can add assets in several ways:

1. **Embedded Assets**: Compile assets directly into your binary for fast loading.
2. **Filesystem Assets**: Load assets from the `assets/` directory at runtime.
3. **Network Assets**: Stream assets from a remote endpoint.

### Using the Asset Manager

The `AssetManager` treats all imported assets as managed resources. Changes made to assets in a scene (like material overrides) are stored on the entity components, not in the source asset files.

```rust
use jingle::asset::{AssetManager, AssetType};

fn load_assets(manager: &mut AssetManager) {
    // Load a 3D model (Reality Composer Pro "Import" equivalent)
    manager.load_file(
        "biplane",
        AssetType::Mesh,
        "meshes/biplane.obj"
    );
    
    // Load a texture for a material
    manager.load_file(
        "engine_noise",
        AssetType::Texture,
        "textures/noise.png"
    );
}
```

## Composing Scenes from Assets

To add an asset to the current scene, you spawn an entity and attach the appropriate components.

```rust
use jingle::reality_kit::{Entity, Transform, ModelComponent};

fn spawn_biplane(scene: &mut Scene, mesh_id: u64, material_id: u64) -> Entity {
    // Create an entity (Reality Composer Pro "Drag to Viewport" equivalent)
    let biplane = scene.spawn(Transform::default());
    
    // Add visual representation
    let model = ModelComponent::new(mesh_id, material_id);
    scene.add_component(biplane, model);
    
    biplane
}
```

### Content Library Integration

Jingle projects can access a library of standard assets defined in the `AssetManifest`.

![Jingle Content Library](./../../assets/rcpro-contentlibrary.png)

```rust
use crate::projects::botanist::assets::AssetManifest;

fn initialize_from_library(manager: &mut AssetManager) {
    let manifest = AssetManifest::default_manifest();
    manifest.load_all(manager);
}
```

## Manipulating Scene Entities

All entities in a Jingle scene exist at a specific position, orientation, and scale. When you select an entity in the PhiPro editor, you can use the manipulator to move, rotate, and scale it.

### Entity Transforms in Rust

```rust
use glam::{Vec3, Quat};
use jingle::reality_kit::Transform;

fn move_entity(scene: &mut Scene, entity: Entity) {
    if let Some(mut transform) = scene.get_component_mut::<Transform>(entity) {
        // Move 2 meters up
        transform.position += Vec3::new(0.0, 2.0, 0.0);
        
        // Rotate 45 degrees around Y axis
        transform.rotation *= Quat::from_rotation_y(45.0f32.to_radians());
        
        // Scale up by 1.5x
        transform.scale *= 1.5;
    }
}
```

## Activating and Deactivating Entities

To simplify complex scenes, you can deactivate entities to hide them from the viewport and exclude them from rendering without removing them from the project.

```rust
fn toggle_entity_state(scene: &mut Scene, entity: Entity, active: bool) {
    // Equivalent to "Deactivate" in Reality Composer Pro
    scene.set_entity_active(entity, active);
}
```

![Entity Deactivation](./../../assets/rcpro-entitydeactivate.png)

## Entity Component System (ECS)

Jingle follows the Entity Component System design pattern. You store data on entities using components (like `Transform`, `ModelComponent`, `RigidBody`) and implement behavior by writing systems.

```rust
// Custom component for a spinning rotor
struct Rotor {
    speed: f32,
}

// System to update all rotors
fn rotor_system(scene: &mut Scene, dt: f32) {
    for (entity, rotor) in scene.query_mut::<Rotor>() {
        if let Some(mut transform) = scene.get_component_mut::<Transform>(entity) {
            transform.rotation *= Quat::from_rotation_y(rotor.speed * dt);
        }
    }
}
```

## Creating Entity Hierarchies

Entities can have parent-child relationships. This is useful for grouping related objects, like an airplane model with multiple child meshes.

```rust
fn create_airplane_hierarchy(scene: &mut Scene) -> Entity {
    let plane_root = scene.spawn(Transform::default());
    
    let fuselage = scene.spawn(Transform::default());
    let wing_l = scene.spawn(Transform::from_position(Vec3::new(-5.0, 0.0, 0.0)));
    let wing_r = scene.spawn(Transform::from_position(Vec3::new(5.0, 0.0, 0.0)));
    
    // Set parent-child relationships
    scene.set_parent(fuselage, plane_root);
    scene.set_parent(wing_l, plane_root);
    scene.set_parent(wing_r, plane_root);
    
    plane_root
}
```

---

## Next Steps

- [Applying Materials on Imported Assets](../applying-realitykit-materials-on-imported-assets/--applying-realitykit-materials-on-imported-assets.md)
- [Designing Materials with Shader Graph](../editor_shadergraph/--editor_shadergraph.md)
- [Building and Running Scenes](../build-and-run-reality-composer-pro-scenes-in-your-app/--build-and-run-reality-composer-pro-scenes-in-your-app.md)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
