# PHX - PHI Point Cloud eXtension

PHX converts USD and glTF files to various point cloud formats.

## Features

- **Import Formats**: USD (USDA/USDC/USDZ), glTF/glb
- **Export Formats**: PHX, PCD, LAS, XYZ, JSON, Binary
- **Mesh Sampling**: Convert triangle meshes to point clouds via area-weighted or uniform sampling
- **Point Cloud Operations**: Transform, filter, downsample, merge, calculate statistics

## Quick Start

```rust
use phx::importers::gltf::GltfImporter;
use phx::importers::{Importer, MeshToCloudConfig};
use phx::PointCloudFormat;

// Import glTF and convert to point cloud
let config = MeshToCloudConfig::default()
    .with_density(100.0)  // points per unit area
    .with_normals(true)
    .with_colors(true);

let point_cloud = GltfImporter::import_file_with_config("model.glb", &config)?;

// Export to PHX format
let format = PointCloudFormat::ply_default();
let ply_data = point_cloud.export(&format)?;
std::fs::write("output.ply", ply_data)?;
```

## Test-Driven Development

This crate was developed using TDD. Tests are in `tests/point_cloud_tests.rs`.

Run tests:
```bash
cargo test -p phx
```

Run examples:
```bash
cargo run --example convert_gltf_to_phx -- model.glb output.ply
cargo run --example convert_usd_to_pcd -- mesh.usda output.pcd
```

## Architecture

```
phx/
├── src/
│   ├── lib.rs          # Core types: PointCloud, Vertex, BoundingBox
│   ├── formats.rs      # Export format definitions
│   ├── exporters.rs    # Export to PHX, PCD, etc.
│   └── importers/
│       ├── mod.rs      # Importer trait, Mesh struct
│       ├── gltf.rs     # glTF/glb importer
│       └── usd.rs      # USD/USDZ importer
├── tests/              # Tests (written first per TDD)
├── examples/           # Usage examples
└── tests/data/         # Test data files
```

## Point Cloud Formats

| Format | Import | Export | Binary | ASCII | Notes |
|--------|--------|--------|--------|-------|-------|
| PHX    | ✓      | ✓      | ✓      | ✓     | Stanford Triangle Format |
| PCD    | ✗      | ✓      | ✓      | ✓     | Point Cloud Library format |
| LAS    | ✗      | ✓      | ✓      | ✗     | ASPRS LiDAR standard |
| XYZ    | ✓      | ✓      | ✗      | ✓     | Simple text format |
| JSON   | ✓      | ✓      | ✗      | ✓     | Custom PHI format |
| Binary | ✗      | ✓      | ✓      | ✗     | Custom binary format |

## Mesh to Point Cloud Conversion

The core algorithm samples points from triangle meshes:

1. Calculate triangle areas
2. Build cumulative distribution for area-weighted sampling
3. Randomly select triangles based on area
4. Sample random barycentric coordinates within each triangle
5. Interpolate position, normal, and color

Configuration options:
- `density`: Points per unit area
- `min_points`/`max_points`: Point count limits
- `uniform_sampling`: Use uniform instead of area-weighted sampling
- `include_normals`/`include_colors`: Output attributes

## License

MIT
