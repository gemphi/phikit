> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [CREATING A JINGLE PROJECT PACKAGE](./--jingle-creating-project-package.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🕶 visionOS Home](../../../visionos/-visionos.md)

---

# Creating a Jingle project package in your app

Discover how to add a new or existing Jingle project as a managed package to your Rust application.

## Overview

If you're building a spatial application with Jingle, you'll need an efficient way to organize and design your 3D content. Jingle uses a "Project Package" structure, which combines an `AssetManifest` with a directory of raw assets (USDZ, OBJ, textures, etc.). This structure is equivalent to a Reality Composer Pro Swift package but optimized for the Rust ecosystem.

Jingle project packages include:
- **`AssetManifest`**: A definition of all resources and their paths.
- **`assets/` directory**: The physical storage for models, textures, and audio.
- **`src/` logic**: Optional Rust code for procedural materials or scene logic.

## 1. Create a New Jingle Project

When you use the PhiPro launcher, you can initialize a new project with a standard structure.

### Using the PhiPro CLI

```bash
# Create a new Jingle project
phi --new MySpatialProject
```

This command creates a new directory with the following structure:

```
MySpatialProject/
├── assets/              # RealityKitContent bundle equivalent
│   ├── meshes/
│   ├── textures/
│   └── materials/
├── project.toml         # Package metadata
└── src/
    └── scene.rs         # Default scene logic
```

![Jingle New Project](./../../assets/rcpro-newproject.png)

## 2. Launch Jingle Reality Composer

You can launch the Jingle editor directly for your project to begin designing in 3D.

```bash
# Open the project in Jingle editor
phi --project MySpatialProject
```

This launches the Jingle Reality Composer window, where you can arrange entities, design materials, and preview your spatial experience.

![Jingle Welcome Window](./../../assets/rcpro-welcomewindow.png)

## 3. Integrating into an Existing Rust App

To use a Jingle project package in your existing Rust application, you need to add the Jingle dependency and point your `AssetManager` to the project directory.

### Cargo.toml Setup

Add Jingle to your application's dependencies:

```toml
[dependencies]
jingle = { path = "path/to/phi/ext/jingle" }
```

### Initializing the Project in Code

In your application's startup logic, configure the `AssetManager` to load the project package:

```rust
use jingle::asset::{AssetManager, AssetManifest};
use std::path::PathBuf;

fn initialize_project() -> AssetManager {
    let mut manager = AssetManager::new();
    
    // Set base path to your project folder
    manager.set_base_path(PathBuf::from("path/to/MySpatialProject/assets"));
    
    // Load the manifest (equivalent to RealityKitContentBundle)
    let manifest = AssetManifest::default_manifest();
    manifest.load_all(&mut manager);
    
    println!("✅ Jingle project package integrated");
    
    manager
}
```

## 4. Loading Scenes into RealityView

By default, Jingle projects provide a high-level API to load their primary scenes into your application's view hierarchy.

```rust
use jingle::reality_kit::Scene;

async fn load_scene(manager: &mut AssetManager) -> Scene {
    let mut scene = Scene::new();
    
    // Load the scene named "Main" from the project package
    // Equivalent to Swift's Entity.load(named: "Scene", in: realityKitContentBundle)
    scene.load_from_manifest("Main", manager).await;
    
    scene
}
```

For more details on loading and running scenes, see [Building and running Jingle scenes in your app](../build-and-run-reality-composer-pro-scenes-in-your-app/--jingle-build-and-run.md).

---

## Next Steps

- [Importing and Organizing Content](../editor_projectbrowser/--jingle-editor-projectbrowser.md)
- [Designing Materials with Shader Graph](../editor_shadergraph/--jingle-editor-shadergraph.md)
- [Project Setup Guide](../JINGLE_PROJECT_SETUP.md)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
