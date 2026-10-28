"""Cloth with a point cache, for `tests/cloth_truth.rs`: what the viewport
shows at the saved frame without simulating.

Each sheet is a 2 m grid of 9 x 9 vertices (81), raised and tilted, two
corners pinned (vertex group "Pin" as the pin group), falling onto
"Floor", a collision plane. The scene steps frames 1..12 so every cache
holds what the solver wrote, then is saved at frame 8:
- "Sheet": cache 1..30 in the file — frame 8 is an exact cached frame.
- "Early": cache 15..40 — frame 8 is before the start, the mesh passes
  through.
- "Late": cache 1..4 — frame 8 is past the end, clamped to 4, the last
  frame.
- "Gap": cache step 2 — frames 1, 3, 5, 7, 9, 11 cached; frame 8 is
  interpolated between 7 and 9.
- "Start": cache 8..30 — frame 8 is the start frame, the mesh passes
  through.
- "Disk": cache 1..30 on disk — the `blendcache_cloth/` folder beside the
  file (written once the file has a path, so the file is saved first).

Blender's evaluated vertices at frame 8 come from a second run of
`../truth/blender_eval_dump.py` on the saved file.

Run: blender -b --python make_cloth.py -- cloth.blend"""
import bpy, bmesh, sys, os, math

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.context.preferences.filepaths.save_version = 0
sc = bpy.context.scene
sc.frame_start, sc.frame_end = 1, 40
path = sys.argv[sys.argv.index("--") + 1]


def link(name, mesh):
    ob = bpy.data.objects.new(name, mesh)
    sc.collection.objects.link(ob)
    return ob


floor_me = bpy.data.meshes.new("Floor")
bm = bmesh.new()
bmesh.ops.create_grid(bm, x_segments=1, y_segments=1, size=6.0)
bm.to_mesh(floor_me)
bm.free()
floor = link("Floor", floor_me)
floor.modifiers.new("Collision", "COLLISION")


def sheet(name, x, start, end, step=1):
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_grid(bm, x_segments=8, y_segments=8, size=1.0)
    bm.to_mesh(me)
    bm.free()
    ob = link(name, me)
    ob.location = (x, 0.0, 1.2)
    ob.rotation_euler = (0.25, 0.1, 0.3)
    pin = ob.vertex_groups.new(name="Pin")
    pin.add([0, 8], 1.0, "REPLACE")
    md = ob.modifiers.new("Cloth", "CLOTH")
    md.settings.vertex_group_mass = "Pin"
    md.settings.quality = 4
    md.point_cache.frame_start = start
    md.point_cache.frame_end = end
    md.point_cache.frame_step = step
    return ob


sheet("Sheet", 0.0, 1, 30)
sheet("Early", 3.0, 15, 40)
sheet("Late", 6.0, 1, 4)
sheet("Gap", 9.0, 1, 30, step=2)
sheet("Start", 12.0, 8, 30)
disk = sheet("Disk", 15.0, 1, 30)

# A disk cache needs a saved file (Blender refuses the toggle before one),
# so save first, then switch the last sheet's cache to disk.
bpy.ops.wm.save_as_mainfile(filepath=path)
disk.modifiers["Cloth"].point_cache.use_disk_cache = True
assert disk.modifiers["Cloth"].point_cache.use_disk_cache
for f in range(1, 13):
    sc.frame_set(f)
sc.frame_set(8)
sc.frame_current = 8
bpy.ops.wm.save_as_mainfile(filepath=path)
