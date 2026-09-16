# Phiphx Crate Directory & Module Hierarchy

This document outlines the directory structure and file organization for the point cloud extension (`phiphx`) module within the `apple` crate.

All files are structured to satisfy the under-200-lines constraint (excluding test lines) and logically isolate responsibilities.

## Directory Tree

```text
phiphx/
├── mod.rs                - Root loader & re-exports
├── error.rs              - PointCloudError & Result
├── vertex.rs             - Vertex structure & attributes
├── bounding_box.rs       - BoundingBox (AABB)
├── mesh.rs               - Mesh representation & area-weighted sampling
├── bvh.rs                - BVH spatial acceleration (BvhNode, SimpleRng)
├── config.rs             - Sampling configuration (MeshToCloudConfig, SamplingMethod)
├── selector.rs           - Primary subject selector (SubjectSelector, MeshSource)
│
├── point_cloud/          - Point Cloud module
│   ├── mod.rs            - Submodule re-exports
│   ├── types.rs          - PointCloud struct definitions
│   └── methods.rs        - Statistics & layout adapters
│
├── format/               - Point cloud format module
│   └── mod.rs            - PointCloudFormat enum & helpers
│
├── importer/             - Importer module
│   ├── mod.rs            - Root importer loader & traits
│   ├── asset.rs          - Unified AssetImporter router
│   ├── project.rs        - Reality Composer Pro ProjectImporter
│   ├── ply.rs            - PLY format importer (PlyImporter)
│   ├── phx.rs            - PHX native binary importer (PhxImporter)
│   ├── format.rs         - FileFormat enum & magic-byte detection
│   ├── obj.rs            - OBJ format importer (ObjImporter)
│   ├── blend/            - Blender SDNA parser module
│   │   ├── mod.rs        - BlendImporter entry point
│   │   ├── types.rs      - SDNA type definitions & MeshSource impl
│   │   ├── dna.rs        - Struct DNA loader
│   │   ├── dna_layout.rs - Struct alignment/size calculator
│   │   ├── parse.rs      - Block file parser
│   │   └── mesh.rs       - Mesh & scene extraction
│   ├── gltf/             - glTF format parser module
│   │   ├── mod.rs        - GltfImporter entry point
│   │   ├── importer.rs   - Node traversal & hierarchy
│   │   └── full.rs       - Mesh converter
│   └── usd/              - USD format parser module
│       ├── mod.rs        - UsdImporter entry point
│       ├── importer.rs   - Prim scanner & traversal
│       └── parse.rs      - USDA ASCII tokenizing & parsing
│
└── exporter/             - Exporter module
    ├── mod.rs            - PointCloudExporter entry point
    ├── gltf.rs           - glTF point cloud & mesh exporter
    ├── usd.rs            - USDA point cloud & mesh exporter
    ├── ply.rs            - PLY ASCII/Binary exporter
    ├── pcd.rs            - PCD ASCII/Binary exporter
    ├── las.rs            - LiDAR LAS exporter
    ├── xyz.rs            - XYZ comma-separated text exporter
    ├── json.rs           - JSON metadata/points exporter
    ├── binary.rs         - Optimized raw bytes exporter
    └── blend/            - Blender scene exporter
        ├── mod.rs        - BlenderSceneExporter entry point
        ├── chunks.rs     - PHX chunk block builders
        ├── config.rs     - PointCloudConfig & SceneExportConfig
        └── script_phx.rs - Sidecar .phx script writer
```

---

## 1. Core Module
* **mod.rs**: Root module loader, submodule declaration, and public API re-exports.
* **error.rs**: PointCloudError and Result types.
* **vertex.rs**: Vertex and VertexAttributes layout definitions.
* **bounding_box.rs**: Axis-Aligned Bounding Box (AABB) implementation.
* **mesh.rs**: Sibling of core structures. Triangle mesh representation, surface area calculations, and triangle sampling algorithms.
* **bvh.rs**: Spatial acceleration hierarchy (`BvhNode`, `BoundingBox`, and `SimpleRng`) built over mesh triangles to accelerate sampling.
* **config.rs**: Configuration types (`MeshToCloudConfig` and `SamplingMethod`) for mesh to point cloud conversion.
* **selector.rs**: Primary subject selector (`SubjectSelector` and `MeshSource` trait) used to choose the primary actor in a scene.

## 2. Point Cloud Module
* **point_cloud/mod.rs**: Submodule loading and re-exports.
* **point_cloud/types.rs**: Struct definitions for PointCloud, PointCloudMetadata, and PointCloudStats.
* **point_cloud/methods.rs**: Point cloud data manipulation, statistics calculation, and memory-layout adapters.

## 3. Importer Module
Consolidates all asset loading, parsing, and project scaffolding. Format-independent geometry calculations are kept at the root of `phiphx/` to isolate importing/parsing from internal types.
* **importer/mod.rs**: Shared parser interfaces and entry types.
* **importer/asset.rs**: Unified Asset Importer. `AssetImporter::import` dispatches raw files by format and routes them to the correct format-specific importer.
* **importer/project.rs**: Reality Composer Pro Project Importer. Scaffolds RCP workspace directories, writes project.json manifests, and generates Scenes/*.scene.json entity structures.
* **importer/ply.rs**: Point cloud parser for Stanford Triangle .ply files.
* **importer/phx.rs**: Parser for native .phx binary scene files.
* **importer/format.rs**: FileFormat enum and magic-byte format detection helpers.
* **importer/obj.rs**: OBJ format parser.
* **importer/blend/**: Blender .blend file format parser:
  * **importer/blend/mod.rs**: Orchestrator dispatching parsing tasks.
  * **importer/blend/types.rs**: Core SDNA structs and `MeshSource` implementation.
  * **importer/blend/dna.rs / dna_layout.rs**: Struct DNA parsing and layout offsets.
  * **importer/blend/parse.rs**: Block file parse methods.
  * **importer/blend/mesh.rs / scene.rs**: Mesh extraction and collections.
* **importer/gltf/**: glTF .gltf and .glb parser:
  * **importer/gltf/mod.rs**: glTF orchestrator entry point.
  * **importer/gltf/importer.rs**: Node and scene hierarchy parsing.
  * **importer/gltf/full.rs**: Full mesh/scene conversion.
* **importer/usd/**: Pixar USD .usda, .usdc, and .usdz parser:
  * **importer/usd/mod.rs**: USD orchestrator entry point.
  * **importer/usd/importer.rs**: Prim scanning and USD schema traversal.
  * **importer/usd/parse.rs**: Low-level tokenizing and USDA parser.

## 4. Exporter Module
Consolidates all serialization, conversion, and format saving.
* **exporter/mod.rs**: Root loader and public interface PointCloudExporter.
* **exporter/blend/**: Full Blender scene exporter with parental hierarchies, lights, and PointInstancers.
* **exporter/gltf.rs**: glTF ASCII / Binary exporter logic.
* **exporter/usd.rs**: Pixar USD ASCII (.usda) point-cloud and mesh exporter logic.
* **Format-Specific Exporters**:
  * **exporter/ply.rs**: Stanford PLY ASCII and Binary exporter.
  * **exporter/pcd.rs**: Point Cloud Library PCD ASCII and Binary exporter.
  * **exporter/las.rs**: ASPRS LiDAR LAS binary format writer.
  * **exporter/xyz.rs**: Plain text XYZ comma-separated file exporter.
  * **exporter/json.rs**: Formatted JSON exporter.
  * **exporter/binary.rs**: Optimized raw bytes point layout exporter.

## 5. Point Cloud Format Module
* **format/mod.rs**: Defines PointCloudFormat enum (Ply, Pcd, Las, Xyz, Json, Binary) and serialization configurations.
