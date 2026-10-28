"""Open grids under Solidify then Subdivision Surface, one per crease setting
the Solidify writes, for `tests/solidify_creases_truth.rs`; Blender's own
evaluated vertices and triangle counts are dumped beside the file as
`solidify_creases_truth.txt`.

Every grid is 4 x 4 faces, 2 m across, its vertices lifted by a fixed
ripple so the shell's normals differ, under a Solidify of thickness 0.2 with
the rim on and a Subdivision of 2 levels after it:
- "Inner": edge crease inner 1.0 (the CW-POSES `panel_underbelly_geo`).
- "Outer": edge crease outer 0.6, offset +1.
- "Rim": edge crease rim 0.4 and outer 0.7, on a grid with its own creases:
  0.5 on one boundary edge (0.5 + 0.7 holds at 1), 0.8 on an inner edge and
  0.9 on one vertex, which the shell copies.
- "Flip": edge crease inner 1.0, offset 0, normals flipped.
- "No Rim": the rim off with crease inner 1.0 (Blender writes none), the
  grid's own creases copied to the shell.
- "Flat Limit": as "Rim" with the limit surface off.

Run: blender -b --python make_solidify_creases.py -- solidify_creases.blend"""
import bpy, bmesh, sys, os, math

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene


def grid(name, x, own_creases=False):
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_grid(bm, x_segments=4, y_segments=4, size=1.0)
    for v in bm.verts:
        v.co.z = 0.15 * math.sin(2.1 * v.co.x + 0.4) * math.cos(1.7 * v.co.y)
    bm.edges.index_update()
    boundary = [e.index for e in bm.edges if e.is_boundary]
    inner = [e.index for e in bm.edges if not e.is_boundary]
    bm.to_mesh(me)
    bm.free()
    if own_creases:
        me.attributes.new("crease_edge", "FLOAT", "EDGE").data[boundary[1]].value = 0.5
        me.attributes["crease_edge"].data[inner[3]].value = 0.8
        me.attributes.new("crease_vert", "FLOAT", "POINT").data[12].value = 0.9
    ob = bpy.data.objects.new(name, me)
    sc.collection.objects.link(ob)
    ob.location = (x, 0.0, 0.0)
    so = ob.modifiers.new("Solidify", "SOLIDIFY")
    so.thickness, so.offset, so.use_rim = 0.2, -1.0, True
    ss = ob.modifiers.new("Subdivision", "SUBSURF")
    ss.levels = 2
    return so, ss


so, ss = grid("Inner", 0.0)
so.edge_crease_inner = 1.0
so, ss = grid("Outer", 3.0)
so.edge_crease_outer, so.offset = 0.6, 1.0
so, ss = grid("Rim", 6.0, own_creases=True)
so.edge_crease_rim, so.edge_crease_outer = 0.4, 0.7
so, ss = grid("Flip", 9.0)
so.edge_crease_inner, so.offset, so.use_flip_normals = 1.0, 0.0, True
so, ss = grid("No Rim", 12.0, own_creases=True)
so.edge_crease_inner, so.use_rim = 1.0, False
so, ss = grid("Flat Limit", 15.0, own_creases=True)
so.edge_crease_rim, so.edge_crease_outer = 0.4, 0.7
ss.use_limit_surface = False

path = sys.argv[-1]
bpy.ops.wm.save_as_mainfile(filepath=path)

# Blender's own evaluated positions, object space, one line per object:
# name, vertices, triangles, coordinates.
dg = bpy.context.evaluated_depsgraph_get()
with open(os.path.join(os.path.dirname(os.path.abspath(path)), "solidify_creases_truth.txt"), "w") as out:
    for ob in sorted(sc.objects, key=lambda o: o.name):
        ev = ob.evaluated_get(dg).to_mesh()
        ev.calc_loop_triangles()
        cos = " ".join(f"{c:.6f}" for v in ev.vertices for c in v.co)
        out.write(f"{ob.name}\t{len(ev.vertices)}\t{len(ev.loop_triangles)}\t{cos}\n")
