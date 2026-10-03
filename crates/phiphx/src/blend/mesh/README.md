# blend/mesh

Mesh geometry. Positions, faces, UVs, normals, face attributes, CustomData (≤ 4.x) and attribute storage (5.x), vertex groups.

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `attrs.rs` | Blender 5.x `Mesh.attribute_storage` (#46). |
| `block.rs` | Vertex/face extraction from a single ME (Mesh) data block. |
| `custom.rs` | `CustomData` float3 layers (`position`, legacy vdata) and layer helpers. |
| `faces.rs` | Face-domain readers: `CustomData` int layers (`material_index`, `.corner_vert`) and `face_offset_indices`. |
| `groups.rs` | Vertex-group weights (`CD_MDEFORMVERT` in the vertex `CustomData`, or the pre-2.9 `Mesh.dvert` array): `MDeformVert { MDeformWeight *dw; int… |
| `legacy.rs` | Pre-4.x mesh arrays: `mvert`, `mpoly` / `mloop`, `poly_offsets`. |
| `mod.rs` | Mesh geometry. |
| `normals.rs` | Per-corner custom / split normals. |
| `offsets.rs` | `Mesh` field offsets (modern 4.x/5.x attribute layout + legacy fields), computed from the file's DNA, plus the count / pointer readers. |
| `positions.rs` | 4.x/5.x `position` and `.corner_vert` attributes (attribute storage or CustomData). |
| `read.rs` | Mesh data resolution, heuristic vertex scanning, and triangulation. |
| `shading.rs` | Face-domain attributes looked up **by exact name**: `sharp_face` (`CD_PROP_BOOL`, Blender 3.5+; absent ⇒ every face smooth) and `material_index`… |
| `uv.rs` | Per-corner UVs: legacy `MLoopUV` and 3.5+ float2 `CustomData` layers. |

Status and next steps: [`jobs.md`](jobs.md).
