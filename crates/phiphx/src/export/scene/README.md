# export/scene

PHX Scene Format — Extended PHX with Scene Metadata

The PHX Scene format extends the standard PHX point cloud format to include:
- Scene metadata (name, frame, resolution)
- Camera with FOV, clip planes, and transform
- Light sources (point, sun, spot, area) with PBR properties
- Mesh geometry with materials and UVs
- Object instancing (linked duplicates, collection instances)
- Scene hierarchy (collections, parent-child relationships)

# File Format
```text
PHX_SCENE\x01                    # Magic + version
<u32: json_length>              # Length of JSON metadata
<json: scene_metadata>           # Scene description
<binary: geometry_data>          # Vertices, indices, etc.
```

# Example Scene JSON
```json
{
"name": "MyScene",
"frame": 1,
"resolution": [1920, 1080],
"format": "phx-scene-v1",
"camera": {
"name": "Camera",
"type": "perspective",
"fov": 0.785,
"matrix_world": [...]
},
"lights": [
{ "name": "Sun", "type": "SUN", "energy": 10.0, ... }
],
"meshes": [
{ "name": "Cube", "vertices": [...], "indices": [...], ... }
],
"instances": [
{ "name": "Tree_Instancer", "prototype": "Tree", "transforms": [...] }
],
"materials": [
{ "name": "Material", "base_color": [1, 0, 0, 1], ... }
]
}
```

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `read.rs` | PHX scene binary deserialization. |
| `scene.rs` | `PhxScene` construction and conversion methods. |
| `tests.rs` | tests |
| `types.rs` | PHX Scene data types and format constants. |
| `write.rs` | PHX scene binary serialization and conversion helpers. |

Status and next steps: [`jobs.md`](jobs.md).
