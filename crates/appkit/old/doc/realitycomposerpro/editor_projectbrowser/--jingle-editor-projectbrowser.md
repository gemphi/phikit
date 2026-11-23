> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [EDITOR PROJECT BROWSER](./--jingle-editor-projectbrowser.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🕶 visionOS Home](../../../visionos/-visionos.md)

---

# Importing and organizing content in your project

Manage high-performance 3D assets in your Jingle project using the Project Browser interface.

## Overview

As you build spatial experiences with Jingle, the Project Browser serves as your central command for organizing, viewing, and managing all your 3D resources. Built into the PhiPro editor, it provides a seamless workflow for handling meshes, textures, materials, and audio files in your Rust application.

By default, the Project Browser is active when you launch a Jingle project, providing a structured view of your `assets/` directory.

![Jingle Project Browser](./../../assets/projectbrowser.jpg)

## Window Components

The Project Browser is divided into three functional areas that map directly to the Jingle `AssetManager`:

- **Navigator View (Hierarchy)**: Shows your project's folder structure and asset collections. This reflects the physical layout of your project's `assets/` directory.
- **File View (Asset Grid)**: Displays the individual asset files within the selected folder. You can see previews of 3D models and textures.
- **Inspector View (Details)**: Provides metadata and configuration options for the selected asset, such as mesh statistics, texture dimensions, or material parameters.

## Organizing Your Project structure

In Jingle, your project's structure is defined by your filesystem and mapped via the `AssetManifest`. You can organize content into logical groups that match your game's systems.

### Directory Mapping in Rust

```rust
use jingle::asset::{AssetManager, AssetType};
use std::path::PathBuf;

fn configure_project_browser(manager: &mut AssetManager) {
    // Navigator View equivalent: Setting the base directory
    manager.set_base_path(PathBuf::from("assets/botanist"));
    
    // File View equivalent: Listing assets
    let meshes = manager.get_by_type(AssetType::Mesh);
    println!("Project Browser (Meshes): {} items found", meshes.len());
}
```

## Managing Assets in the File View

The Jingle editor supports standard file operations within the Project Browser:
- **Import**: Add new files to the `assets/` directory.
- **Delete**: Remove an asset and update the manifest.
- **Preview**: Double-click any asset to open it in a specialized editor (like the Shader Graph for materials).

### Asset Selection and Referencing

When you select an asset in the File View, you can see its unique ID or reference path, which you'll use in your Rust code.

```rust
// Referencing an asset in code
let robot_mesh = manager.get_handle_by_name("robot_head_v02")
    .expect("Asset not found in Project Browser");
```

## Examining Information in the Inspector

The Inspector View shows detailed information about your assets, which is critical for optimization and debugging.

### Mesh Information
- Vertex and triangle counts.
- Bound material slots.
- Bounding box dimensions.

### Texture Information
- Resolution and pixel format.
- Mipmap levels.
- Filtering modes.

```rust
fn inspect_asset(manager: &AssetManager, handle: AssetHandle) {
    if let Some(asset) = manager.get(handle) {
        println!("Inspector:");
        println!("  Name: {}", asset.name);
        println!("  Size: {} bytes", asset.data.len());
        println!("  Type: {:?}", asset.asset_type);
    }
}
```

---

## Next Steps

- [Importing Assets Guide](../STEP_BY_STEP_ASSET_IMPORT_GUIDE.md)
- [Designing Materials with Shader Graph](../editor_shadergraph/--jingle-editor-shadergraph.md)
- [Jingle Project Setup](../JINGLE_PROJECT_SETUP.md)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
