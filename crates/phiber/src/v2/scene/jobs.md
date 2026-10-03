# Jobs — `phiber/v2/scene`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 2 | 5 |

## Todo

- [ ] Animation: drive object / bone matrices from the frame
- [ ] Pivot and lens editing from the view (not only Load UI)

## Done

- [x] Rendered shading = phiray GPU ray tracing (`view/draw/canvas/view3d/rendered.rs`), falls back to the mesh painter without an adapter
- [x] `RaySkin`: Armature deform of opened files on the GPU via phiray, installed in `app::run`; CPU fallback below 20k verts or without an adapter (`tests/gpu_skin.rs`)
- [x] Camera (`camera.rs`), overlay extras, shading modes, material and texture colour
- [x] The Overlays and Gizmos popovers drive the viewport (`overlay.rs`): floor and axes, text, statistics, extras, bones, relationship lines (`extra/relation.rs`), outlines, origins, the 3D cursor, the navigation gizmo — each file opening with its own view's switches (`view/draw/canvas/view3d/guide/tests.rs`)
- [x] The 3D cursor moves and is used (`cursor.rs`): Shift + right-click onto the surface, Object ▸ Snap, Shift C, new objects at it, saved with the file (`cursor_tests.rs`, phiphx `write/cursor.rs`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
