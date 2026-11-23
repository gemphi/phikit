> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [BUILD AND RUN JINGLE SCENES](./--jingle-build-and-run.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🕶 visionOS Home](../../../visionos/-visionos.md)

---

# Building and running Jingle Reality Composer scenes in your app

Preview scenes on your spatial device and learn how to load them in your Rust application.

## Overview

As your Jingle Reality Composer projects grow in complexity, you'll need efficient ways to preview and test your content. Jingle provides tools to preview scenes directly on a visionOS device or simulator, and a straightforward API for loading these scenes into your production application.

## Previewing Scenes

You can preview your Jingle scenes directly from the PhiPro editor without performing a full application rebuild.

### Preview on visionOS

If you have a visionOS device or simulator active, you can launch a live preview of your Jingle scene. This allows for rapid iteration of layout, materials, and animations.

```bash
# Preview the current project in visionOS mode
phi --project MyProject --vision
```

In the editor UI, look for the **Spatial Preview** button (represented by a headset icon) to stream the current viewport state to your connected device.

## Loading Scenes in Your App

Loading a Jingle scene in Rust is similar to loading standard RealityKit assets. You use the `Scene` and `AssetManager` APIs to fetch your content and integrate it into your application's `RealityView` equivalent.

### Project Manifests

Each Jingle project defines an `AssetManifest` that acts as the "bundle" for your content. The system uses this manifest to resolve paths and handle asset dependencies.

### Code Example: Loading a Scene

The following code shows how to load a scene into your Jingle application:

```rust
use jingle::reality_kit::{Scene, Entity, Transform};
use jingle::asset::{AssetManager, AssetManifest};

async fn setup_app_content(manager: &mut AssetManager, scene: &mut Scene) {
    println!("🔄 Loading Jingle content...");
    
    // 1. Create or load the manifest
    let manifest = AssetManifest::default_manifest();
    
    // 2. Load all assets (RealityKit await Entity.load equivalent)
    let handles = manifest.load_all(manager);
    
    // 3. Initialize entities from the loaded assets
    if let Some(mesh_handle) = manager.get_by_name("Biplane") {
        let biplane = scene.spawn(Transform::default());
        scene.add_component(biplane, ModelComponent::new(mesh_handle.id, 0));
        
        println!("✅ Scene loaded successfully");
    }
}
```

When the `AssetManager` finishes loading, the entities are integrated into your ECS `World`, which is then processed by the render system for display.

## Running the Application

To build and run your Jingle project, use the standard Cargo workflow. You can test performance and visual fidelity across different platforms.

### Target: Desktop (MacOS/Windows/Linux)

```bash
cargo run --package phipro -- --project MyProject
```

### Target: visionOS

```bash
cargo run --package phipro --features visionos -- --project MyProject --vision
```

## Performance Monitoring

Jingle includes built-in statistics tracking, equivalent to the "Statistics" tab in Reality Composer Pro. You can view frame times, draw calls, and entity counts directly in the PhiPro console.

```rust
fn check_performance(scene: &Scene) {
    let stats = scene.get_render_stats();
    println!("Frame Time: {:.2}ms", stats.frame_time);
    println!("Draw Calls: {}", stats.draw_calls);
    println!("Active Entities: {}", scene.entity_count());
}
```

---

## Next Steps

- [Project Setup](../JINGLE_PROJECT_SETUP.md)
- [Importing Assets](../adding-assets-into-your-reality-composer-pro-scene/--jingle-adding-assets-to-scene.md)
- [API Comparison](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md#api-comparison)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
