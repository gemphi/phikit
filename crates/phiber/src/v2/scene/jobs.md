# Jobs — `phiber/v2/scene`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 1 | 11 |

## Todo

- [ ] Pivot and lens editing from the view (not only Load UI)

## Done

- [x] A render is never handed nothing (`render_cast.rs`): F12 on pose 26 reached the tracer with an empty scene (995 objects in the file, 967 in the viewport, 0 in the render) and the window said "No render". `BRenderCast` counts what the render rules keep — objects, then objects with geometry — and when nothing would be drawn it names the step that emptied it (the collections with the camera off or excluded, by id) in the Info editor and renders what the viewport shows instead; a deliberate divergence from Blender's empty frame, logged every time (`render_cast::tests`)
- [x] `BRenderPhase::Failed` (`render_phase.rs`): a job that ends with no picture is a failure, not an empty slot — the info bar prints `Render failed: <why>` from `render_job::failure()` (set by `still_stepwise`'s refusal) where it printed "No render" (`render_phase_tests`)
- [x] Animation drives the scene from the frame (`scene::motion`, `scene::pose`, phiphx `SceneMotion`): objects, rig bones, rig-deformed meshes, lights and cameras all pose at the shown frame (`BPosed::exact`, `solved_at`), so playback at the file's fps actually moves the scene — `moving_frame`/`posed` re-solve only an animated scene, a still one never re-poses. The frame is per render pass like the loaded model. (phiphx motion/rig tests + `scene::motion`)
- [x] Snap menu: Cursor to Grid, Selection to Grid and Selection to Active (`scene::cursor`, `view3d.snap_*`): the cursor rounds to the nearest world unit, and each selected object's origin snaps to the nearest grid point or onto the active object — the same transform machinery as Selection to Cursor, one per-object target. Replaces three `op()` placeholders. (`cursor_tests`)
- [x] A render keeps its picture between readbacks (`render_job::start`): most samples report only a count (`BStillAt::Counting`), and the publish closure sent those with EMPTY bytes, wiping the peek published a moment before — the Render Result flashed on at each readback (3/s) and off at the very next sample, which reads as a blank canvas and makes an animation flicker. The closure now keeps the last image's planes and republishes them with the new count; only a readback replaces them. (`render_job` tests)
- [x] `view3d.view_camera` (Numpad 0, View ▸ Camera): into the scene camera's view and back out to the orbit for the viewport under the pointer (`view_op` `view_camera`, `BViewSlot::current`), an unset area reading its workspace's default (the Animation left viewport opens through the camera); no camera in the scene says so and flips nothing, as Blender's poll does. Replaces the menu's placeholder. (`view_op` tests)
- [x] Rendered shading = phiray GPU ray tracing (`view/draw/canvas/view3d/rendered.rs`), falls back to the mesh painter without an adapter
- [x] `RaySkin`: Armature deform of opened files on the GPU via phiray, installed in `app::run`; CPU fallback below 20k verts or without an adapter (`tests/gpu_skin.rs`)
- [x] Camera (`camera.rs`), overlay extras, shading modes, material and texture colour
- [x] The Overlays and Gizmos popovers drive the viewport (`overlay.rs`): floor and axes, text, statistics, extras, bones, relationship lines (`extra/relation.rs`), outlines, origins, the 3D cursor, the navigation gizmo — each file opening with its own view's switches (`view/draw/canvas/view3d/guide/tests.rs`)
- [x] The 3D cursor moves and is used (`cursor.rs`): Shift + right-click onto the surface, Object ▸ Snap, Shift C, new objects at it, saved with the file (`cursor_tests.rs`, phiphx `write/cursor.rs`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
