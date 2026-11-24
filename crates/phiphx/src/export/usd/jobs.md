# Jobs — `phiphx/export/usd`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 8 | 3 |

## Todo

- [ ] USDC (binary) writer — `export_usdc` is a stub; `.usdc` / `.usdz` are not write targets yet (a USD input going to the same encoding is copied)
- [ ] Scene: cameras and lights are left out (only meshes); textures (`base_color_texture`, normal / roughness maps) are not written, only flat colours and roughness / metallic / opacity
- [ ] Scene: faces go out as triangles — `SceneModel` holds triangles, not Blender's quads and n-gons (`SceneUvMap` has the faces; a polygon path would read them)
- [ ] Scene: UVs only per point (`SceneObject::uvs`); split UVs (per corner) need the face-varying UV map
- [ ] Scene: objects with no faces (PLY point clouds, curves without a bevel) are skipped; `def Points` would carry them
- [ ] Scene: instances are written as copies; a collection instance could be a USD reference / instanceable prim
- [ ] Check every written layer with Reality Composer Pro / `usdchecker` on a Mac (not available here)

## Done

- [x] USDA for meshes and point clouds (`UsdExporter`, `UsdaWriter`)
- [x] Shapes as USDA (`shape.rs`): Xform root, Mesh per part with faceVarying normals, displayColor, extent, `UsdPreviewSurface` material bound per part. Tests: `geom::shape::tests::shape_usda_reads_back`, `phiogy_shape_file_match_generator`
- [x] Whole scenes as USDA (`scene.rs`), whatever they were read from: shown objects as Xform + Mesh at their world transform, names cleaned and unique, materials as `UsdPreviewSurface`, per-face slots as `GeomSubset`s, posed rigs posed (`Phx::convert`). Replaces `BlenderScene::to_usda`'s first-mesh point cloud. Tests: `tests/usd_scene.rs`

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
