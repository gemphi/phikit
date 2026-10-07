# Blender's own answer for the object drivers in `fixtures/OBJDRIVERS.blend`:
# each object's evaluated matrix translation, rotation and scale at a frame,
# with the drivers run — the reference for `tests/object_driver_truth.rs`.
#
#   blender -b OBJDRIVERS.blend -P blender_object_drivers.py -- OUT.txt
#
# One line per object per frame: `frame name loc_x loc_y loc_z rot_x rot_y
# rot_z scale_x scale_y scale_z`, taken from the *evaluated* depsgraph copy,
# which is where a driver's result lands.
import bpy, sys, os

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
out = args[0] if args else "object_drivers.txt"
if len(args) > 1:
    bpy.ops.wm.open_mainfile(filepath=args[1])

sc = bpy.context.scene
names = sorted(o.name for o in sc.objects)
lines = []
for frame in (1, 6, 11, 16, 21):
    sc.frame_set(frame)
    dg = bpy.context.evaluated_depsgraph_get()
    for name in names:
        ob = sc.objects[name].evaluated_get(dg)
        m = ob.matrix_world
        loc, rot, scale = m.decompose()
        e = rot.to_euler("XYZ")
        lines.append(
            f"{frame}\t{name}\t{loc.x:.6f}\t{loc.y:.6f}\t{loc.z:.6f}"
            f"\t{e.x:.6f}\t{e.y:.6f}\t{e.z:.6f}"
            f"\t{scale.x:.6f}\t{scale.y:.6f}\t{scale.z:.6f}"
        )

with open(out, "w") as fh:
    fh.write("\n".join(lines) + "\n")
print(f"-> {out} ({len(lines)} lines)")
