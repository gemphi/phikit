# Jobs — `phiber/v2/scene`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 1 | 9 |

## Todo

- [ ] Pivot and lens editing from the view (not only Load UI)

## Done

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
