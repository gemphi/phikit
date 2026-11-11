# blend
The Blend device: `.blend` files. `BlendDevice` (in `mod.rs`) is the `Device`
for them: `detect`, `peek`, `read`, `read_progressive`. `BlendImporter` serves
the asset pipeline. Parses Blender's SDNA container and extracts the scene
Blender would show, without Blender installed, then runs Blender's own
evaluation (modifiers, rigs, animation, physics, shading).
Callers enter through [`phx::Phx`](../../phx.rs); this folder is one of its parts.
## Files
| File | What it is |
|---|---|
| `mod.rs` | `BlendDevice`, `BlendImporter`, the progressive-open switch, the public re-exports. |
| `open.rs` | Open a `.blend`: the guarded read, settled into a `SceneModel` (whole, progressive, from bytes). |
| `file/` | The `.blend` container. Header, block headers, block table and pointer lookup (`scope`), DNA and struct layout, zstd, quick peek, names. |
| `read/` | What a file holds, one folder per kind: `asset/`, `curve/`, `mesh/`, `object/`, `scene/`, `text/`, and `idprop.rs`. |
| `write/` | Writing the file back: `to_bytes` / `write_to`, field patching by DNA, the view layer's blocks, the marker list, `BlendEditList`; `script/` writes Blender Python scripts. |
| `modifier/` | The viewport modifier stack (Blender `mesh_calc_modifiers`), limited to modifiers that change how an existing mesh looks. |
| `rig/` | Pose solving: constraints, IK, drivers, bone display. |
| `anim/` | Actions, F-curves, NLA, drivers, the motion of every object at a frame. |
| `physics/` | Rigid body, cloth, soft body, force fields: settings only. |
| `shade/` | Materials. Slots, viewport colour and Principled inputs, shader node tree, images (keys and packed bytes). |
| `test/` | Unit tests, one folder per folder tested; modules reach them by `#[path]`. |
## Ground truth
`tests/blender_truth.rs` compares per-mesh digests with `tests/truth/mesh_digest.txt` (made by Blender 5.0.1); `crates/phiber/tests/viewport_vs_blender.rs` compares whole saved views with Blender renders in `tests/truth/views/`.
Status and next steps: [`jobs.md`](jobs.md).
