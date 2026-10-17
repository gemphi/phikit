# Jobs — `phiphx/blend`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 3 | 5 |

## Todo

- [ ] Animation reader: legacy `bAction.curves` and layered actions (4.4+). **Bézier evaluation is done** — `anim/fcurve.rs:179`, `IPO_BEZ => bezier(a, b, frame)`, with its own test; it was on this line as open and was not (found in the batch-191 sweep)
- [ ] Linked libraries (`LI` blocks, `ID.lib`)
- [ ] Writer: new `OB` / `ME` blocks for added and duplicated objects, dropping deleted ones (`write/` patches only what the file already has)

## Done

- [x] Reads Blender 2.8 → 5.2; 1,276 meshes match Blender 5.0.1 exactly (`tests/blender_truth.rs`)
- [x] Split into `file/`, `object/`, `mesh/`, `shade/`, `scene/`, `modifier/`, `asset/`
- [x] Constraints on bones and on objects — every matrix kind, IK, drivers — objects that follow a rig's bones (bone parents, bone targets), and objects on a curve's path (curve parents, Follow Path, Clamp To), solved with the motion (`rig/tests.rs`, `rig/constraint/tests.rs`, `anim/follow_tests.rs`, `curve/tests.rs`)
- [x] What the viewport draws of a rig and over the scene: each armature's shown bones and custom shapes (`rig/display_tests.rs`), each 3D view's overlay and gizmo switches, the 3D cursor, and the relationship lines — parent display origins and constraint targets (`scene/overlay_tests.rs`)
- [x] Writer (`write/`): the read file back byte for byte in every BHead layout, zstd/gzip kept, with the session's edits patched by DNA — frame range, names, visibility and base flags, active object, collection flags, markers (`tests/save_truth.rs`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
