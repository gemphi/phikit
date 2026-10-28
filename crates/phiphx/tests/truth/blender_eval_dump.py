"""Blender's own evaluated geometry and world matrices of an opened file, one
line per object, for the truth tests and the real-file checks
(`examples/truth_check.rs`):

    verts\t<name>\t<count>\tx y z x y z …     evaluated vertices, object space
    world\t<name>\t16\tm00 m01 m02 m03 m10 …  `matrix_world`, row-major

Run: blender -b FILE --python blender_eval_dump.py -- OUT.tsv [object …]
With no objects named, every mesh object's vertices and every object's
world matrix are written; with names, only those (vertices for meshes,
the world for all). The launcher detaches, so wait for OUT.tsv."""
import bpy, sys

args = sys.argv[sys.argv.index("--") + 1:]
out, names = args[0], set(args[1:])
dg = bpy.context.evaluated_depsgraph_get()
with open(out, "w") as f:
    for ob in bpy.data.objects:
        if names and ob.name not in names:
            continue
        m = ob.matrix_world
        f.write("world\t%s\t16\t%s\n" % (ob.name, " ".join("%.7g" % m[r][c] for r in range(4) for c in range(4))))
        if ob.type != "MESH":
            continue
        ev = ob.evaluated_get(dg)
        me = ev.to_mesh()
        cos = " ".join("%.7g %.7g %.7g" % tuple(v.co) for v in me.vertices)
        f.write("verts\t%s\t%d\t%s\n" % (ob.name, len(me.vertices), cos))
        ev.to_mesh_clear()
