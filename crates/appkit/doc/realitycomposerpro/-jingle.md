> [JINGLE](../-jingle.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](./-jingle.md)
- [🕶 visionOS Home](../visionos/-visionos.md)

---

# Jingle Reality Composer

Build, create, and design 3D content for your spatial Rust applications.

## Overview

**Jingle** provides a collection of tools and editors you can use to build interactive 3D content for your RealityKit-compatible apps. With Jingle, you can create complex scenes composed of multiple assets — including audio files, 3D models, particle effects, lighting, and animations — all within the PhiPro development environment.

![A screenshot of a Jingle Reality Composer project window in PhiPro. The window has a single tab at the top called Scene. The window is divided horizontally into two sections. The top section has three views: a hierarchy/outliner view of content in the scene on the left, a 3D viewport in the middle showing a rendered model with additional assets using the Phiray engine, and a properties inspector on the right. The bottom section has four tabs, labeled Project Browser, Shader Graph, Timelines Audio Mixer, and Statistics. Project Browser is selected, and it's also divided into three sections with a folder hierarchy on the left, a grid of asset icons in the middle, and an inspector on the right showing the entity hierarchy for the asset selected in the middle.](./../assets/rcpro-heroshot.png)

Jingle also includes multiple editors to help you import and manage assets in your project, design complex materials with Shader Graph, create animations, and optimize the performance of your content.

## Topics

### Essentials

- [Creating a Jingle package in your app](./creating-a-reality-composer-pro-package-in-your-app/--jingle-creating-project-package.md)
- [Configuring the Jingle project window](./configure-the-reality-composer-pro-project-window/--configure-the-jingle-project-window.md)
- [Adding assets to your Jingle scene](./adding-assets-into-your-reality-composer-pro-scene/--jingle-adding-assets-to-scene.md)
- [Building and running Jingle scenes in your app](./build-and-run-reality-composer-pro-scenes-in-your-app/--jingle-build-and-run.md)

### Editors

- [Importing and organizing content in your project](./editor_projectbrowser/--jingle-editor-projectbrowser.md)
- [Designing materials with Shader Graph](./editor_shadergraph/--jingle-editor-shadergraph.md)

### Materials

- [Building Jingle materials](./building-materials/--jingle-building-materials.md)

---

## Jingle vs Reality Composer Pro

| Feature | Reality Composer Pro (Apple) | Jingle (PhiPro) |
|---------|------------------------------|-----------------|
| **Platform** | macOS, visionOS | Windows, macOS, Linux |
| **Language** | Swift | Rust |
| **3D Engine** | RealityKit | Phiray + Jingle |
| **UI Framework** | SwiftUI | egui + PhiWin |
| **Viewport** | Metal | wgpu (WebGPU) |
| **Asset Format** | USDZ | USDZ, OBJ, GLTF, PHIRAY |
| **Shader Graph** | Node-based | Node-based (Rust) |
| **Physics** | RealityKit Physics | Jingle Physics |
| **Multiplayer** | RealityKit | Phx ( Phi networking) |

---

## PhiPro Integration

Jingle is fully integrated into **PhiPro**, the multi-app spatial development launcher:

```bash
# Launch Jingle Editor
phi

# Launch with specific project
phi --project MyProject

# Launch in visionOS mode
phi --vision

# Launch Botanist game (built on Jingle)
phi --botanist
```

---

## Code Example

```rust
use jingle::reality_kit::{Scene, Entity, Transform, ModelComponent};
use jingle::asset::AssetManager;

fn main() {
    // Create scene (equivalent to Reality Composer Pro project)
    let mut scene = Scene::new();
    let mut assets = AssetManager::new();
    
    // Load USDZ asset
    let robot_handle = assets.load_file("robot", AssetType::Mesh, "robot.usdz");
    
    // Create entity
    let robot = scene.spawn(Transform::default());
    scene.add_component(robot, ModelComponent::new(robot_handle.id, 0));
    
    // Run application
    phipro::run(scene, assets);
}
```

---

*Generated for Jingle RealityKit Implementation v0.1.0*
