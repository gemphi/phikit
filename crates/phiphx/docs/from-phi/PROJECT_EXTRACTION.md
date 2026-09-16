# PHX Project Extraction Architecture

## Overview

The `phx` module provides a unified pipeline for importing 3D asset files (like `.blend` or `.usd`) and extracting them into structured project folder hierarchies. This ensures that artists and developers can drop an asset file into the system and instantly receive a well-organized folder structure containing serialized geometry ready for engine use.

## Unified Import API

The core functionality is provided by `phx::project::import_to_project`. This function acts as a single entry point for all supported asset types:

```rust
pub fn import_to_project(input_file: &Path, output_base_dir: &Path) -> Result<PathBuf>
```

When you call this function with an asset (e.g., `Mannequins-Female--Pose-05.blend` or `scene.usd`), it will automatically:
1. Detect the file type via extension.
2. Create a project root directory with the `.phx` suffix (e.g., `Mannequins-Female--Pose-05.phx/`).
3. Parse the asset file.
4. Extract the hierarchy and serialize objects into the project structure.

## Extraction Flow by Format

### Blender (`.blend`) Extraction

1. **Parsing**: The `.blend` file is opened and parsed natively by `BlendFile::parse`. This skips the need for a running Blender instance.
2. **Scene Extraction**: The scene hierarchy and all objects (meshes, curves, etc.) are extracted into memory.
3. **Folder Creation**: 
   - A root project folder is created: `[asset_name].phx/`.
   - For every collection in the Blender scene (ignoring `WGT` collections), a subfolder is created. This happens recursively, mirroring the exact collection tree from Blender.
4. **File Serialization**:
   - For each object within a collection, a standalone `.usdz` file is natively generated inside the collection's folder (e.g., `[asset_name].phx/CollectionA/ObjectB.usdz`).
   - The `.usdz` file is an uncompressed zip containing the `.usda` format (which holds the serialized mesh data: vertices, normals, indices, and matrix transforms).

### USD (`.usd`, `.usda`, `.usdc`, `.usdz`) Extraction

1. **Parsing**: The USD file is read. For `.usdz`, the zip archive is decompressed in memory to find the primary `.usda`/`.usd`/`.usdc` payload. For `.usda`, the ASCII text is parsed to extract `def Mesh` nodes and geometry.
2. **Scene Extraction**: The importer currently aggregates all mesh geometry into a single `PointCloud` or mesh definition. *(Note: Future iterations will parse the full USD scenegraph to mirror the recursive collection extraction seen in Blender).*
3. **Folder Creation**:
   - A root project folder is created: `[asset_name].phx/`.
4. **File Serialization**:
   - The aggregated point cloud or mesh data is serialized as a `.usdz` file within the project folder.

## Folder Structure Example

If you import `Character.blend` with the following Blender hierarchy:

```
Scene Collection
├── Body
│   ├── Torso
│   └── Head
└── Clothing
    ├── Shirt
    └── Pants
```

The resulting file system structure will be:

```
exports/Character.phx/
├── Body/
│   ├── Torso.usdz
│   └── Head.usdz
└── Clothing/
    ├── Shirt.usdz
    └── Pants.usdz
```

## How to Run

You can test the unified extraction pipeline using the provided example:

```bash
cargo run --example extract_project
```

This will scan `phi/ext/phx/imports/` for any `.blend` or `.usd` files and extract them to `phi/ext/phx/exports/`.
