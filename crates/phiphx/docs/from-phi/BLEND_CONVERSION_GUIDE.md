# Blender .blend to PHX Scene Conversion - Complete Guide

## Overview

This guide documents the complete solution for converting Blender .blend files to PHX scene format using native Rust code (no Python/Blender dependency required).

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Input: .blend file                                              │
│  (FLORIS_BOLDRIOD_RIG.blend, etc.)                              │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│  Native Rust Parser (blend_native.rs)                           │
│  ├── BlendHeader (version, pointer size, endian)                 │
│  ├── BlendBlock[] (data blocks: OB, ME, SC, DNA, etc.)         │
│  └── DnaType (structure definitions)                           │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│  BlendScene (native representation)                            │
│  ├── collections: BlendCollection[]                            │
│  └── objects: BlendSceneObject[]                               │
│      ├── name, type, location, rotation, scale                   │
│      ├── vertices: Vec<[f32; 3]>                               │
│      └── faces: Vec<Vec<u32>>                                 │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│  PHX Scene Format (phx_scene.rs)                                 │
│  ├── PhxScene { name, collections, meshes, materials }          │
│  └── Binary format: PHX_SCENE\x01 + JSON + binary geometry     │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────┐
│  Output: .phx file                                               │
└─────────────────────────────────────────────────────────────────┘
```

## Step-by-Step Implementation

### Step 1: Parse .blend Binary Header

The .blend file starts with a 12-byte header:

```rust
// Header format: "BLENDER-v304" or "BLENDER-V304" or "BLENDER-_304" etc.
//   7 bytes: "BLENDER"
//   1 byte:  '_' (4-byte ptr) or '-' (8-byte ptr)
//   1 byte:  'v' (little endian) or 'V' (big endian)
//   3 bytes: version number (e.g., "304")

pub struct BlendHeader {
    pub version: String,        // e.g., "304"
    pub pointer_size: u8,     // 4 or 8
    pub little_endian: bool,  // true for 'v'
    pub version_number: u32,  // 304
}
```

### Step 2: Read Data Blocks

After the header, .blend files contain sequential data blocks:

```rust
pub struct BlendBlock {
    pub code: [u8; 4],        // Block type: "OB"=Object, "ME"=Mesh, "DNA1"=DNA, "ENDB"=end
    pub size: u32,            // Data size in bytes
    pub old_address: u64,     // Memory address when saved (for pointer resolution)
    pub sdna_index: u32,        // Index into DNA type table
    pub count: u32,           // Number of structures in this block
    pub data: Vec<u8>,        // Raw binary data
}
```

Block types:
- **OB** (Object): Scene objects with transforms
- **ME** (Mesh): Vertex/face data
- **SC** (Scene): Scene settings
- **GR** (Group/Collection): Collection hierarchy
- **DNA1**: Structure definitions (critical for parsing)
- **ENDB**: End marker

### Step 3: Parse DNA Structure

The DNA block contains structure definitions needed to decode other blocks:

```rust
pub struct DnaType {
    pub names: Vec<String>,      // Field names
    pub types: Vec<String>,      // Type names (int, float, char[64], etc.)
    pub type_sizes: Vec<u16>,    // Size of each type in bytes
    pub structs: Vec<DnaStruct>, // Structure definitions
}

pub struct DnaStruct {
    pub type_index: u16,         // Index into types[]
    pub fields: Vec<DnaField>,   // Fields in this struct
}
```

### Step 4: Decode Objects Using DNA

Using the DNA structure, decode OB blocks to extract:

```rust
pub struct BlendSceneObject {
    pub name: String,
    pub object_type: String,     // "MESH", "CAMERA", "LIGHT", etc.
    pub location: [f32; 3],
    pub rotation: [f32; 3],      // Euler angles
    pub scale: [f32; 3],
    pub matrix_world: [[f32; 4]; 4],
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<Vec<u32>>,
}
```

### Step 5: Convert to PHX Scene Format

Transform native BlendScene to PHX format:

```rust
pub struct PhxScene {
    pub name: String,
    pub frame: i32,
    pub resolution: [i32; 2],
    pub format: String,          // "phx-scene-v1"
    pub camera: Option<PhxCamera>,
    pub lights: Vec<PhxLight>,
    pub meshes: Vec<PhxMesh>,
    pub instances: Vec<PhxInstance>,
    pub materials: Vec<PhxMaterial>,
    pub collections: Vec<PhxCollection>,
}
```

### Step 6: Export to Binary

PHX format layout:

```
PHX_SCENE\x01           (10 bytes magic + version)
<4 bytes>              JSON length (u32 little-endian)
<JSON data>            Scene metadata (UTF-8)
<binary geometry>      Vertex/index buffers (optional, for large scenes)
```

## Files Created

| File | Purpose |
|------|---------|
| `src/importers/blend_native.rs` | Native .blend parser |
| `src/importers/blender_export_script.rs` | **DEPRECATED** - Python script for Blender |
| `src/importers/blender.rs` | **DEPRECATED** - Blender subprocess wrapper |
| `src/exporters/phx_scene.rs` | PHX scene format writer |
| `examples/convert_blend_to_phx.rs` | Conversion CLI tool |
| `BLEND_CONVERSION_GUIDE.md` | This documentation |

## Usage

### Command Line

```bash
# Convert a .blend file to PHX format
cargo run -p phx --example convert_blend_to_phx -- \
  "assets/FLORIS_BOLDRIOD_RIG.blend"

# With custom output path
cargo run -p phx --example convert_blend_to_phx -- \
  "input.blend" \
  "output.phx"
```

### Library API

```rust
use phx::importers::import_blend_native;
use phx::exporters::phx_scene::{export_phx_scene_to_bytes, PhxScene};

// Import .blend file (no Blender required!)
let blend_scene = import_blend_native(Path::new("scene.blend"))?;

// Convert to PHX format
let phx_scene = convert_to_phx_scene(&blend_scene);

// Export to bytes
let bytes = export_phx_scene_to_bytes(&phx_scene)?;
std::fs::write("scene.phx", bytes)?;
```

## Testing

Run the test suite:

```bash
# Unit tests for blend parser
cargo test -p phx blend_native

# Integration test with sample files
cargo test -p phx --test blend_integration

# Run the conversion example
cargo run -p phx --example convert_blend_to_phx -- \
  "assets/FLORIS_BOLDRIOD_RIG.blend"
```

## Known Limitations

1. **Object Extraction**: Currently parses file structure but needs DNA-based decoding for full object/mesh data extraction
2. **Materials**: PBR material extraction from .blend not yet implemented
3. **Modifiers**: Applied modifiers are baked into mesh data (no modifier stack preserved)
4. **Animation**: Static frame only (no animation data)

## Future Improvements

- [ ] Complete DNA-based object data decoding
- [ ] Material/texture extraction
- [ ] Collection hierarchy reconstruction
- [ ] Instance/dupli support
- [ ] Multi-threaded parsing for large files
- [ ] Streaming parser for memory efficiency
