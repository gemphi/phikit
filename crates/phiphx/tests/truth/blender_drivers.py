# Blender truth for `blend/anim/read.rs::drivers`: every driven object's
# driver F-curves — path, index, key count, and the Generator modifier's
# polynomial coefficients when it has one.
#
#   python3 blender_drivers.py FILE.blend OUT.txt      (bpy module)
import bpy, sys

bpy.ops.wm.open_mainfile(filepath=sys.argv[1])
with open(sys.argv[2], "w") as f:
    for ob in bpy.data.objects:
        ad = ob.animation_data
        if not ad or not ad.drivers:
            continue
        f.write("object\t%s\t%d\n" % (ob.name, len(ad.drivers)))
        for fc in ad.drivers:
            gen = next((m for m in fc.modifiers if m.type == 'GENERATOR' and m.mode == 'POLYNOMIAL'), None)
            coefficients = " ".join("%g" % c for c in gen.coefficients) if gen else "-"
            f.write("driver\t%s\t%d\t%d\t%s\n" % (fc.data_path, fc.array_index, len(fc.keyframe_points), coefficients))
