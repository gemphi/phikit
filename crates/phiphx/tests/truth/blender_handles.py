# Blender truth for `blend/anim/handles.rs`: keys inserted with
# `keyframe_insert` (auto-clamped handles, Continuous Acceleration
# smoothing, Blender 5.0's defaults) and the handles Blender computes.
#
#   python3 blender_handles.py OUT.txt        (bpy 5.0 as a module)
import bpy, sys

CASES = [
    [(1, 0.0), (20, 3.0), (40, 1.0)],
    [(1, 0.0), (10, 1.0), (20, 3.0), (30, 6.0)],
    [(1, 0.0), (4, 2.0), (30, 2.5), (31, 5.0), (60, 10.0)],
    [(1, 5.0), (8, 2.0), (15, 2.5), (22, -1.0), (40, -1.0), (41, 3.0), (70, 0.0)],
    [(10, 2.0)],
    [(1, 0.0), (10, 5.0)],
    [(1, 0.0), (2, 0.5), (3, 1.5), (5, 4.0), (9, 4.1), (12, 9.0), (13, 9.5), (25, 20.0)],
    [(0, 1.0), (7, 1.0), (14, 1.0)],
]
out = []
for n, case in enumerate(CASES):
    bpy.ops.wm.read_factory_settings(use_empty=False)
    cube = bpy.data.objects["Cube"]
    for frame, value in case:
        bpy.context.scene.frame_set(frame)
        cube.location.x = value
        cube.keyframe_insert("location", index=0)
    act = cube.animation_data.action
    fc = act.layers[0].strips[0].channelbag(act.slots[0]).fcurves[0]
    for k in fc.keyframe_points:
        out.append("case %d key %s %s %s %s %s" % (n, "%.4f,%.4f" % tuple(k.co), "%.4f,%.4f" % tuple(k.handle_left), "%.4f,%.4f" % tuple(k.handle_right), k.handle_left_type, k.interpolation))
    out.append("case %d eval %s" % (n, " ".join("%.4f" % fc.evaluate(f) for f in range(int(case[0][0]) - 2, int(case[-1][0]) + 3, 3))))
open(sys.argv[-1], "w").write("\n".join(out) + "\n")
