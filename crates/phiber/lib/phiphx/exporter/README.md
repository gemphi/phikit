---
id: exporters
title: phx Exporters
---

Writers that take phx geometry back out to disk. Like the importers, every
exporter is dependency-free — it emits the target format byte-for-byte without
Blender, OpenUSD, or a glTF SDK. The `.phx` writers are the centre of gravity:
they are the portable, fast-to-reload, collaboration-friendly format the rest
of the toolchain round-trips through.

## Overview

Where the importers narrow many formats down to phx types, the exporters fan
back out. A parsed scene can be persisted as a compact `.phx` (for fast
re-import and sharing), or transcoded to an interchange format (`.usda/.usdc`,
`.gltf/.glb`, `.blend`) for handoff to other tools.

Two `.phx` flavours exist:

* **`phx_native`** — a point-cloud chunk format (geometry only). Lightweight,
  used where just the silhouette/points matter.
* **`phx_scene`** — the richer scene format: a magic header + JSON metadata
  (camera, lights, materials, instances, collection hierarchy) followed by
  binary geometry. This is what a `.blend` import emits as its sidecar.

### Export fan-out

```mermaid
flowchart TD
    A[phx scene / mesh / point cloud] --> B{target format}
    B -->|".phx (scene)"| C[phx_scene<br/>JSON meta + binary geom]
    B -->|".phx (points)"| D[phx_native<br/>point-cloud chunk]
    B -->|".usda / .usdc"| E[usd_native]
    B -->|".gltf / .glb"| F[gltf_native]
    B -->|".blend"| G[blender_native]
    C --> H[fast re-import · collaboration]
    D --> H
    E --> I[interchange / handoff]
    F --> I
    G --> I
```

### `.phx` scene layout

```mermaid
graph LR
    M["PHX_SCENE\x01<br/>magic + version"] --> L["u32 json_length"]
    L --> J["JSON metadata<br/>camera · lights · materials<br/>instances · collections"]
    J --> B["binary geometry<br/>vertices · indices · UVs"]
```

### Round-trip with importers

```mermaid
flowchart LR
    BL[.blend] -->|blend_native import| SC[BlendScene]
    SC -->|phx_scene write| PHX[.phx sidecar]
    PHX -->|read_phx_scene| SC2[PhxScene]
    SC2 --> VP[engine viewport]
    SC2 -->|phx_scene_to_phx| PTS[point cloud]
```

## How is this module organized?

    exporters/
    ├── phx_scene.rs / phx_scene/    # rich scene format (meta + geometry) — the sidecar
    ├── phx_native.rs / phx_native/  # point-cloud chunk format
    ├── usd_native.rs / usd_native/  # USDA / USDC writer (mesh + point cloud)
    ├── gltf_native.rs / gltf_native/# glTF 2.0 writer (base64 buffers, mesh, points)
    ├── blender_native.rs / blender_native/ # .blend writer
    └── blender.rs                   # legacy Blender wrapper types

## Format summary

| Output | Module | Contents | Primary use |
|--------|--------|----------|-------------|
| `.phx` (scene) | `phx_scene` | camera, lights, materials, instances, hierarchy, geometry | fast re-import, collaboration sidecar |
| `.phx` (points) | `phx_native` | point-cloud chunk | lightweight geometry |
| `.usda` / `.usdc` | `usd_native` | mesh + point cloud | USD interchange |
| `.gltf` / `.glb` | `gltf_native` | mesh + point cloud, base64/embedded buffers | web/glTF interchange |
| `.blend` | `blender_native` | Blender scene | Blender round-trip |

## Key entry points

* `write_phx_scene` / `read_phx_scene` — round-trip the rich scene format.
* `export_phx_scene_to_bytes` / `import_phx_scene_from_bytes` — in-memory variants.
* `phx_scene_to_phx` — downconvert a scene to a plain point cloud.
* `export_mesh_usda` / `export_usdc` — USD writers.
* `export_mesh_gltf` / `export_point_cloud_glb` — glTF writers.

## Notes

* Each source file is kept ≤ 200 lines (project convention).
* `phx_scene` is the format the `.blend` importer writes as a freshness-gated
  sidecar (see `../importers/blend_native`), enabling instant re-import and
  on-the-fly collaboration.
