# export

Point cloud exporter.

Implements export to various point cloud formats.

Callers enter through [`phx::Phx`](../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `binary.rs` | Custom PHX binary format exporter and helper base64 utilities. |
| `blend/` | Blender Native Exporter — Full Scene Export with Instancing This module provides comprehensive Blender scene export capabilities: - Mesh geometry… |
| `blend.rs` | Blender Native Exporter — Full Scene Export with Instancing |
| `gltf/` | Native glTF 2.0 Exporter Direct export to glTF format without external dependencies. Optimized for PHI PHX workflow. |
| `gltf.rs` | Native glTF 2.0 Exporter |
| `json.rs` | JSON format exporter and importer. |
| `las.rs` | LAS format exporter. |
| `pcd.rs` | PCD format exporter. |
| `phx/` | Native PHX Exporter Direct export to PHX binary format. Optimized for PHI workflow. |
| `phx.rs` | Native PHX Exporter |
| `ply.rs` | PLY (PHX) format exporter and importer. |
| `scene/` | PHX Scene Format — Extended PHX with Scene Metadata The PHX Scene format extends the standard PHX point cloud format to include: - Scene metadata… |
| `scene.rs` | PHX Scene Format — Extended PHX with Scene Metadata |
| `usd/` | Native USD Exporter Direct export to USDA/USDC format without external dependencies. Optimized for PHI PHX workflow. |
| `usd.rs` | Native USD Exporter |
| `xyz.rs` | XYZ ASCII format exporter and importer. |

Status and next steps: [`jobs.md`](jobs.md).
