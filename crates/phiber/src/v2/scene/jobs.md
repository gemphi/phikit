# Jobs — `phiber/v2/scene`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 2 | 3 |

## Todo

- [ ] Animation: drive object / bone matrices from the frame
- [ ] Pivot and lens editing from the view (not only Load UI)

## Done

- [x] Rendered shading = phiray GPU ray tracing (`view/draw/canvas/view3d/rendered.rs`), falls back to the mesh painter without an adapter
- [x] `RaySkin`: Armature deform of opened files on the GPU via phiray, installed in `app::run`; CPU fallback below 20k verts or without an adapter (`tests/gpu_skin.rs`)
- [x] Camera (`camera.rs`), overlay extras, shading modes, material and texture colour

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
