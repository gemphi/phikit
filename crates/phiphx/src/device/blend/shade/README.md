# blend/shade

Materials. Slots, viewport colour and Principled inputs, shader node tree, images (keys and packed bytes).

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `driver.rs` | What a driver on a material writes — a `Material` property or a shader socket's `default_value` — and the clamp each write path takes (the three disagree). |
| `driver_read.rs` | Reading those drivers out of the file: `Material.adt` for the material's own properties, `Material.nodetree->adt` for the sockets, pathed `nodes["…"].inputs[i].default_value`. |
| `world_driver.rs` | Drivers on the world — `World.color` on the ID, and the Background node's `Strength` and `Color` on its tree. |
| `group.rs` | Node groups expanded inline before the fold: a `ShaderNodeGroup`'s tree spliced into its parent, links re-aimed across the Group Input / Output sockets, an unlinked group input carrying the instance's socket value. |
| `fold.rs` | The fold's value / vector / colour nodes: Map Range, Separate / Combine XYZ and Color, Vector Math, Vector Rotate, Normal, Fresnel and Layer Weight (as the hemispherical mean), Vertex Color (white) — and the same emitted to a bake. |
| `sepcomb.rs` | Separate / Combine Color in RGB, HSV and HSL (`svm_separate_color`, `rgb_to_hsl`). |
| `vector.rs` | `svm_vector_math` (every operation), `svm_node_vector_rotate` and Cycles' Euler transform. |
| `image.rs` | Image datablocks (#26): the key a material uses to name its texture and the bytes of images packed into the `.blend`. |
| `material.rs` | Per-object material extraction: resolves the object's first `Material` (`MA`) block and reads its diffuse colour + PBR roughness/metallic. |
| `mod.rs` | Materials. |
| `nodes.rs` | Material shader graph walk: `Material.*nodetree` → `bNode` list → Principled BSDF "Base Color" socket and the first Image Texture's… |
| `procedural.rs` | The Noise / Voronoi textures the surface reads per pixel — on the base colour, the Mix Shader's factor, a Bump on the normal — with their coordinates and colour ramps baked, for the tracer (`BlendProcedural`). |
| `procscalar.rs` | The Math and Map Range nodes around a Color Ramp — composed into the ramp's 256 stops before it, carried as the slot's tail after it — and the ramp's bake. |
| `texnode.rs` | A texture node read for the tracer: which node and output, its sockets folded and its storage's settings (noise type, Voronoi feature and metric, gradient / wave types, brick offsets, magic depth). |
| `surface.rs` | The surface shader the Material Output is linked to, folded to one `BlendBsdf`: Mix / Add Shader, Emission, Glass, Transparent, Diffuse, Glossy and the rest as the Principled parameters that draw most like them. |
| `slots.rs` | Material slot resolution: `Object/Mesh.**mat` + `totcol` → `MA` blocks, and the exact-name DNA field-offset lookup the material readers use. |

Status and next steps: [`jobs.md`](jobs.md).
