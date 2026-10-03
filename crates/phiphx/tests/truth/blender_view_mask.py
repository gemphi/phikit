"""Render the silhouette Blender shows for a file's saved 3D view.

    python blender_view_mask.py -- FILE OUT.png camera
    python blender_view_mask.py -- FILE OUT.png view "<viewinv 4x4, m[col][row]>" DIST

`camera` renders through the scene camera (files saved in camera view);
`view` rebuilds the saved perspective user view (RegionView3D.viewinv/dist,
default 50 mm lens: a 25 mm lens on a 36 mm sensor gives the same width).
Modifiers render at their viewport `levels`. Output: 480x270, alpha = mask.
Used by crates/phiber/tests/viewport_vs_blender.rs.
"""
import sys

import bpy
import mathutils

args = sys.argv[sys.argv.index("--") + 1:]
path, out, mode = args[0], args[1], args[2]
try:
    bpy.ops.wm.open_mainfile(filepath=path, load_ui=False)
except RuntimeError:
    pass
sc = bpy.context.scene
for o in sc.objects:
    for m in o.modifiers:
        if m.type == "SUBSURF":
            m.render_levels = m.levels
if mode == "view":
    inv, dist = eval(args[3]), float(args[4])
    cam = bpy.data.objects.new("View", bpy.data.cameras.new("View"))
    cam.data.sensor_fit, cam.data.sensor_width, cam.data.lens = "HORIZONTAL", 36.0, 25.0
    cam.data.clip_start, cam.data.clip_end = 0.01, 10000.0
    sc.collection.objects.link(cam)
    cam.matrix_world = mathutils.Matrix([[inv[c][r] for c in range(4)] for r in range(4)])
    sc.camera = cam
r = sc.render
r.engine, sc.cycles.samples, sc.cycles.use_denoising = "CYCLES", 1, False
r.resolution_x, r.resolution_y, r.film_transparent, r.filter_size = 480, 270, True, 0.01
r.image_settings.media_type, r.image_settings.file_format, r.image_settings.color_mode = "IMAGE", "PNG", "RGBA"
r.filepath = out
bpy.ops.render.render(write_still=True)
