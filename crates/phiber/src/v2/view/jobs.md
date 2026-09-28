# Jobs — `phiber/v2/view`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 5 | 4 |

## Todo

- [ ] Spatial look: one elevation scale (0 base · 1 zone · 2 popover · 3 modal) with soft depth shadows, translucent "glass" materials over the viewport, continuous rounded corners, and motion (modal scale-in, zone expand/collapse, hover lift) — all from theme tokens
- [ ] One menu look: every popover body uses `MenuItem` rows
- [ ] One zone look: all zone / area / host frames from the same surface treatment
- [ ] GPU viewport: renderer 2 (appkit wgpu, Material Preview) and renderer 3 (WebGPU path tracer) in the View3D area
- [ ] Viewport interaction on the real camera: orbit yaw + pitch, Shift pan of the pivot, dolly, numpad views, frame selected

## Done

- [x] Real orbit camera (`scene/camera.rs`), floor grid in 3D, view-relative studio light
- [x] Silhouettes match Blender renders (`tests/viewport_vs_blender.rs`)
- [x] Sortable headers: label left, arrow at the end, slot reserved
- [x] Window caption controls (`WindowControls`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
