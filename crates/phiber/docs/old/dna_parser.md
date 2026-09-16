# Blender DNA File Parser & Data Model

## 1. `.blend` File Format Architecture

A `.blend` file is a binary memory dump of Blender's internal data structures organized as follows:

```
┌────────────────────────────────────────────────────────┐
│ HEADER (12 Bytes)                                      │
│ "BLENDER" + pointer_size ('_'=32bit, '-'=64bit) +       │
│ endianness ('v'=little, 'V'=big) + version ("402")     │
├────────────────────────────────────────────────────────┤
│ FILE BLOCKS                                            │
│ Code (4B) | Length (4B) | Address (4/8B) |             │
│ SDNA Index (4B) | Count (4B) | Payload Data ...        │
├────────────────────────────────────────────────────────┤
│ DNA1 BLOCK (Structure Definitions)                     │
│ SDNA table describing C structs, fields, and types     │
└────────────────────────────────────────────────────────┘
```

## 2. `blenda` Module in BUIApp

In `buiapp/blenda/`, `BUIApp` implements native parsing of `.blend` data structures into high-level Rust models:

- `BlenderScene`: Root scene data block holding objects, materials, and collections.
- `BlenderObject`: Scene graph object with transform, type, mesh reference, and material assignment.
- `BlenderMesh`: Geometric mesh containing vertices, faces, edges, UV layers, and vertex colors.
- `Material`: Principled BSDF shader properties and node graph tree (`NodeTree`).

## 3. Ingestion Pipeline

```
.blend file / memory buffer
  └─> blend_parser::parse_header()
  └─> blend_parser::parse_blocks()
  └─> blend_parser::read_dna()
  └─> build_scene() -> BlenderScene
        └─> Conversion to BModel / WGPU Mesh Buffers
```
