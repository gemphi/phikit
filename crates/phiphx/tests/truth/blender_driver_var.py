# Blender's own answer for `fixtures/DRIVERVARS.blend` — every driver
# variable kind (`dvar_eval_singleProp`, `dvar_eval_transChan`,
# `dvar_eval_locDiff`, `dvar_eval_rotDiff`) and the custom-property chain
# the depsgraph orders. The reference for `tests/driver_var_truth.rs`.
#
#   python -I blender_driver_var.py <out.txt> <fixture.blend>
#
# One line per object per frame: `frame name` then the sixteen elements of
# `matrix_world`, column by column (Blender's own `m[col][row]` layout), from
# the *evaluated* depsgraph copy, which is where a driver's result lands. The
# matrix rather than a decomposition, since a Min driver can drive a scale
# negative and no decomposition of a mirrored matrix is unique.
import bpy, sys

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
out = args[0] if args else "driver_var.txt"
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
        cols = [f"{m[row][col]:.6f}" for col in range(4) for row in range(4)]
        lines.append(f"{frame}\t{name}\t" + "\t".join(cols))

with open(out, "w") as fh:
    fh.write("\n".join(lines) + "\n")
print(f"-> {out} ({len(lines)} lines)")
