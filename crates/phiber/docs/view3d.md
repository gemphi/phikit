# The 3D Viewport — what is Blender's, what is not yet, and the tracking

> Companion to `rendering.md` (what the viewport's Material and Rendered
> modes paint and how they compare to Cycles) and `timeline.md` (the
> animation editors). This one is the viewport itself — `space_view3d`,
> the overlay engine, the navigation gizmo — against Blender 5.2, with a
> tracking table kept current as work lands. Written 2026-09-26.

## 0. Tracking

Status: **done** (ported and guarded), **partial** (drawn, but not the whole
of Blender's behaviour), **todo** (not started). The Blender column names the
source each row is measured against.

| # | Part | Blender source | Status | Notes |
| --- | --- | --- | --- | --- |
| 1 | Shading modes Wire / Solid / Material / Rendered | `View3DShading::type`, Workbench, EEVEE / Cycles | **done** | Solid is a Workbench-style dispatch over the tracer's BVH; Material and Rendered are the path tracer (`rendering.md` §1, §18) |
| 2 | Floor grid and X / Y axis lines | `overlay_grid.cc` | **done** | two decades at once, fading as cells shrink; perspective-clipped |
| 3 | Camera frustums, light gizmos, empties | `overlay_extra.cc`, `overlay_shape.cc`, `BKE_camera_view_frame_ex` | **done** | `scene/extra`; unit shapes from Blender 5.2 |
| 4 | Text info overlay (User Perspective, collection \| object, frame, statistics) | `view3d_draw.cc::draw_viewport_name` / `draw_selected_name` / `draw_stats` | **partial** | lines are there; statistics counts objects / verts / faces only (no Tris, no memory) |
| 5 | Navigation gizmo (compass, orbit, zoom, pan, perspective, camera) | `view3d_gizmo_navigate.cc` | **partial** | compass caps write viewpoints; zoom / recenter / frame-all / perspective-orthographic buttons act; the camera button (`VIEW3D_OT_view_camera`) toggles this area's camera view (`BCameraSlot::toggle`); the gizmo corner is choosable. Not yet: the compass's drag orbit |
| 6 | Orbit / zoom / pan / named viewpoints / perspective toggle | `view3d_navigate*.cc`, `view3d.view_axis` | **done** | drag orbits, scroll zooms, pitch clamped ±90 |
| 7 | Selection by click, active object highlight | `view3d_select.cc`, `overlay_outline.cc` | **partial** | hit target per object writes `ctx.object`; outline colours are the theme's; no box / lasso select |
| 8 | Header: editor type, mode, View / Select / Add / Object menus | `VIEW3D_HT_header`, `VIEW3D_MT_*` | **partial** | the menus exist with a handful of entries each; Blender's full lists (`VIEW3D_MT_view` ≈ 25, `VIEW3D_MT_add` 13 submenus, `VIEW3D_MT_object` ≈ 30) not yet ported |
| 9 | Header right: visibility, gizmo, overlay, X-ray, shading buttons and popovers | `VIEW3D_HT_header` right half, `VIEW3D_PT_shading`, `VIEW3D_PT_overlay` | **partial** | shading buttons switch modes; overlay / gizmo popover contents are the shape of Blender's panels but few toggles act |
| 10 | Tool header (active tool settings) | `VIEW3D_HT_tool_header` | **partial** | exists; tool settings are static |
| 11 | Toolbar (T) with Blender's tool list | `_defs_view3d_generic`, `toolsystem` | **partial** | tools listed and the choice is kept in state; no tool changes what a drag does |
| 12 | Sidebar (N): Item / Tool / View | `VIEW3D_PT_context_properties`, `VIEW3D_PT_view3d_properties` | **partial** | the tab rail exists; the tabs' panels (Item transform, View lens / clip) are not filled — the Properties editor's Object tab has the transform read-out |
| 13 | Camera view (Numpad 0, `persp = CAMERA`) with frame and passepartout | `view3d_draw.cc::view3d_draw_border`, `ED_view3d_calc_camera_border`, `view3d_center_camera_exec`, `ED_view3d_persp_switch_from_camera` | **partial** | the Animation workspace's left viewport looks through the scene camera (`scene/camera_view`): the frame is `view3d_camera_border` at the file's `camzoom` when the saved view is the camera view (phiphx reads `RegionView3D.camzoom`), else the frame-to-fit zoom; the passepartout `TH_CAMERA_PASSEPARTOUT` at `passepartalpha` 0.5, the outline solid `TH_BACK` under dashed `TH_VIEW_OVERLAY`, the overlay reads "Camera Perspective" (`view3d_get_name`). The gizmo's camera button enters and leaves it for the area; an orbit drag while in it leaves it carrying the camera's view over (`BCameraView::leave` — yaw, pitch, pivot, zoom, projection — then the area's key false), as Blender does on the first orbit. Not yet: Numpad 0 (no hotkey map or hovered-area yet), `camdx` / `camdy`, composition guides, safe areas, the sensor rectangle |
| 14 | One view per area (each 3D Viewport keeps its own `RegionView3D`) | `RegionView3D` | **partial** | the first per-area slot is in: `View3dKey::Camera(scope, slot)` (`ctx.view3d.<space>.<slot>.camera`) carried by the canvas (`BCameraSlot`) from `area::current` at build, defaulting to `BSpaceType::opens_in_camera_view` — so the Animation workspace's left and centre viewports differ. Orbit, pan and zoom are still one shared view: leaving a camera view seeds that shared orbit |
| 15 | Armature drawing (bones, octahedral / stick) | `overlay_armature.cc` | **todo** | rigs load (the THYMUS rig has 530 curves) but their bones are not drawn |
| 16 | Object origins, relationship lines, bounds | `overlay_extra.cc` | **todo** | |
| 17 | Edit mode (vertices / edges / faces) | `overlay_edit_mesh.cc` | **todo** | the mode dropdown lists it; nothing changes yet |
| 18 | Frame rate and playback in the viewport (`ctx.playing` advances `ctx.frame`, objects move) | `screen_ops.cc::screen_animation_step`, depsgraph | **partial** | the ticker advances the frame and the Timeline follows; objects do not yet move with their F-curves — the curves are read (`timeline.md` §3) but not applied to transforms |

## 1. What the viewport is today

The canvas (`view/draw/canvas/view3d/`) composes: the floor and axes
(`floor.rs`), the objects in the chosen shading mode (`draw.rs` decides
between the GPU tracer and the CPU painter, `mesh/` paints), the overlay
extras (`extra.rs` strokes what `scene/extra` produces), the text overlay
(`canvas/overlay.rs`), the navigation gizmo (`canvas/gizmo`, in the corner
`corner.rs` chooses), the orbit surface (`orbit.rs`) and the per-object hit
targets (`hit.rs`). Everything projects through one camera model
(`scene/camera.rs`: Blender's `RegionView3D` orbit — yaw, pitch, dolly, pan,
pivot, perspective or orthographic), fed from `View3dKey` state.

Material and Rendered modes are the phiray path tracer; how far they are from
Cycles, measured frame against frame, is `rendering.md` §7, §18 and §19.

## 2. What Blender does that phiber does not yet

Row 13 is in for the case a demo shows: the Animation workspace's left
viewport is a **camera view**, as `startup.blend` saves it. `scene/camera_view`
turns the active camera's `matrix_world` into the orbit model — the eye is
the camera, the pivot ten units ahead along its line of sight, yaw and pitch
from its back axis (`SceneView::angles`), the field of view from its sensor
over its lens, fitted to the render's larger side — and `BCameraFrame` is
`view3d_camera_border`: both view planes share one lens, so the frame is
`render × viewfac_region / (viewfac_camera × zoom)` wide, centred, at the
`camzoom` `view3d_center_camera_exec` would choose (four pixels to spare).
`canvas/view3d/frame.rs` paints the passepartout in four quads and the
outline. What remains of row 13 is listed in the table; row 14 has its first
per-area slot (`View3dKey::Camera`) and the rest of the view still shared.

Then the armature (row 15) — a rigged character with no bones drawn is not
what Blender shows — and moving objects during playback (row 18), which is
the one that makes the animation editors *mean* something in the viewport.

## 3. Guards

`layout_fingerprint.rs` pins the viewport's chrome at two window sizes,
`gizmo_interaction_regression.rs` and `viewport_pan_props.rs` drive the
gizmo and pan through the built window, `runtime_layout_regression.rs`
checks the fill chain (no surround padding, the canvas reaches every pane
edge), and `canvas/view3d/rendered/tests.rs` renders the fixture scene
through the tracer under lavapipe.
