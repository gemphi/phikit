---
id: importers
title: phx Importers
---

Readers that bring external 3D scenes into phx. Each importer is
dependency-free (no Blender, no OpenUSD, no glTF SDK) and converts a source
file into phx geometry — point clouds, `Mesh`es, or the richer `BlendScene`.
The goal is a single, uniform ingestion layer so that anything dropped into the
editor's Project Browser — regardless of where it came from — surfaces as the
same in-engine types, ready for the hierarchy, the viewport, and the inspector.

## Overview

A 3D asset arrives in one of several wire formats, each with its own quirks: a
`.blend` is a raw memory dump of Blender's runtime structs; a `.usd*` is a
scene graph that may be ASCII, binary, or zipped; a `.gltf`/`.glb` is a
JSON-plus-binary transmission format. Rather than leak those quirks upward,
every importer narrows its format down to the same handful of phx types so the
rest of the engine never has to care about provenance.

All importers share the small trait surface in `importers.rs`:

* `Importer` — `can_import` / `import_bytes` / `import_file`. Detection is by
  content signature first (magic bytes), extension second, so a mislabelled
  file still routes correctly.
* `MeshSource` — uniform read access to a parsed object's name, vertices,
  indices, and world matrix. This is the seam the GPU asset loaders read from.
* `Mesh` / `MeshToCloudConfig` — the common mesh type and the point-sampling
  config used when a caller wants a `PointCloud` preview instead of full
  topology.

### Ingestion pipeline

```mermaid
flowchart TD
    A[Source file / bytes] --> B{Detect signature}
    B -->|".blend / BLENDER"| C[blend_native]
    B -->|"#usda / PXR-USDC / PK zip"| D[usd]
    B -->|"glTF JSON / glb"| E[gltf]
    C --> F[BlendScene<br/>collections + objects]
    D --> G[PointCloud<br/>or Vec&lt;Mesh&gt;]
    E --> H[Vec&lt;Mesh&gt;]
    F --> I[MeshSource trait]
    G --> I
    H --> I
    I --> J[GPU asset loaders]
    I --> K[.phx exporter<br/>portable sidecar]
    J --> L[Viewport · Hierarchy · Inspector]
    K --> M[Fast re-import · Collaboration]
```

### Where each importer sits

```mermaid
graph LR
    subgraph traits["importers.rs · shared surface"]
        T1[Importer]
        T2[MeshSource]
        T3[Mesh / MeshToCloudConfig]
    end
    blend_native --> traits
    usd --> traits
    gltf --> traits
    blender[blender · scene builder] --> blend_native
    traits --> exporters[../exporters · phx_scene]
```

## How is this module organized?

    importers.rs            # shared traits (Importer, MeshSource), Mesh, MeshToCloudConfig
    importers/
    ├── blend_native/       # native Blender .blend parser → BlendScene (+ .phx sidecar)   [see its README]
    ├── usd/                # USDA / USDC / USDZ importer                                   [see its README]
    ├── blender/            # higher-level Blender scene builder (cleanup, convert, process)
    ├── gltf/               # glTF 2.0 importer (mesh, transform, full-scene)
    ├── blender.rs          # legacy Blender wrapper types (PHXMesh)
    └── gltf.rs             # glTF module root

## Output formats

| Input            | Importer            | Primary output           | Topology | Materials |
|------------------|---------------------|--------------------------|----------|-----------|
| `.blend`         | `blend_native`      | `BlendScene` + `.phx`    | full     | diffuse + Principled base color |
| `.usda/.usdc/.usdz` | `usd`            | `PointCloud` / `Vec<Mesh>`| full (usda) / approx (usdc) | — |
| `.gltf/.glb`     | `gltf`              | `Vec<Mesh>`              | full     | per-primitive |

## Design notes

* **Dependency-free by policy.** Each format is parsed from first principles so
  the crate builds anywhere without native toolchains. The trade-off is that
  some readers (notably USDC) are heuristic rather than exhaustive.
* **Signature-first detection.** `can_import` inspects magic bytes before
  trusting the extension, so renamed files still route to the right reader.
* **One result, many sources.** Whatever the input, callers consume the parsed
  object through `MeshSource`, which keeps the GPU side format-agnostic.
* **Portable sidecars.** A `.blend` import also emits a `.phx` (see
  `../exporters/phx_scene.rs`) so the next load is instant and the scene can be
  shared for on-the-fly collaboration.

See `../exporters/` for the matching writers (notably `phx_scene` — the
portable `.phx` format used for fast re-import and collaboration).
