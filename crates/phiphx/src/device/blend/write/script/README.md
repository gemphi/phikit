# export/blend

Blender Native Exporter — Full Scene Export with Instancing

This module provides comprehensive Blender scene export capabilities:
- Mesh geometry with materials
- Light sources (point, sun, spot, area)
- Camera settings
- Object instancing (linked duplicates, collection instances)
- Scene hierarchy (parent-child relationships)
- Collection export
- Point cloud generation via area-weighted sampling

# Supported Export Formats
- PHX Scene (.phx.ply) - Extended PHX with scene metadata
- glTF 2.0 (.gltf/.glb) - With PHX point cloud extensions
- USDA (.usda) - USD ASCII with PointInstancer for instances

# Instancing Support
Blender's instancing is exported via:
- **USD**: `PointInstancer` with prototypes for particle/dupli systems
- **glTF**: `EXT_mesh_gpu_instancing` extension
- **PHX Scene**: Instance transforms with prototype references

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `chunks.rs` | Native Blender Exporters for PHX Format |
| `config.rs` | Export configuration types for Blender native scene export. |
| `helpers.rs` | Blender type-name parsing (`From<&str>`). |
| `script_gltf.rs` | glTF 2.0 scene writer (delegates to `gltf_native`). |
| `script_phx.rs` | PHX scene format writer (binary: magic + JSON). |
| `script_usd.rs` | USDA scene writer (delegates to `usd_native`). |
| `tests.rs` | Unit tests for Blender native scene export. |
| `types.rs` | Core data types for Blender native scene export. |

Status and next steps: [`jobs.md`](jobs.md).
