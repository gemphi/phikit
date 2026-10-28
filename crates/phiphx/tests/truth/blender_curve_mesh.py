"""Blender's own meshes of every curve object in a file, for
`tests/curve_mesh_truth.rs`.

For each CURVE object: the mesh with its modifiers off ("BASE", the curve
converted alone) and with them ("EVAL"), each as its vertex and triangle
counts and its bounding box in object space; BASE also with its vertices
in order (6 decimals).

Run: blender -b FILE --python blender_curve_mesh.py -- OUT.txt"""
import bpy, sys

out = open(sys.argv[sys.argv.index("--") + 1], "w")


def dump(tag, ob, points):
    dg = bpy.context.evaluated_depsgraph_get()
    e = ob.evaluated_get(dg)
    me = e.to_mesh()
    me.calc_loop_triangles()
    vs = [v.co for v in me.vertices]
    lo = [min(v[i] for v in vs) for i in range(3)] if vs else [0, 0, 0]
    hi = [max(v[i] for v in vs) for i in range(3)] if vs else [0, 0, 0]
    out.write("%s %d %d %s %s\n" % (tag, len(vs), len(me.loop_triangles),
                                    " ".join("%.6f" % x for x in lo), " ".join("%.6f" % x for x in hi)))
    for v in vs if points else []:
        out.write("V %.6f %.6f %.6f\n" % tuple(v))
    e.to_mesh_clear()


for ob in sorted(bpy.data.objects, key=lambda o: o.name):
    if ob.type != "CURVE":
        continue
    out.write("OBJ %s\n" % ob.name)
    shown = [m.show_viewport for m in ob.modifiers]
    for m in ob.modifiers:
        m.show_viewport = False
    dump("BASE", ob, True)
    for m, s in zip(ob.modifiers, shown):
        m.show_viewport = s
    dump("EVAL", ob, False)
out.write("DONE\n")
out.close()
