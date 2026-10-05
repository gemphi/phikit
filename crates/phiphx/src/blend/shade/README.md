# blend/shade

Materials. Slots, viewport colour and Principled inputs, shader node tree, images (keys and packed bytes).

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `image.rs` | Image datablocks (#26): the key a material uses to name its texture and the bytes of images packed into the `.blend`. |
| `material.rs` | Per-object material extraction: resolves the object's first `Material` (`MA`) block and reads its diffuse colour + PBR roughness/metallic. |
| `mod.rs` | Materials. |
| `nodes.rs` | Material shader graph walk: `Material.*nodetree` → `bNode` list → Principled BSDF "Base Color" socket and the first Image Texture's… |
| `procedural.rs` | The Noise / Voronoi textures the surface reads per pixel — on the base colour, the Mix Shader's factor, a Bump on the normal — with their coordinates and colour ramps baked, for the tracer (`BlendProcedural`). |
| `texnode.rs` | A texture node read for the tracer: which node and output, its sockets folded and its storage's settings (noise type, Voronoi feature and metric, gradient / wave types, brick offsets, magic depth). |
| `surface.rs` | The surface shader the Material Output is linked to, folded to one `BlendBsdf`: Mix / Add Shader, Emission, Glass, Transparent, Diffuse, Glossy and the rest as the Principled parameters that draw most like them. |
| `slots.rs` | Material slot resolution: `Object/Mesh.**mat` + `totcol` → `MA` blocks, and the exact-name DNA field-offset lookup the material readers use. |

Status and next steps: [`jobs.md`](jobs.md).
