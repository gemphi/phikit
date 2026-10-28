"""Arrays with "Merge" on, and one large n-gon, for `tests/array_merge_truth.rs`;
Blender's own evaluated vertices and triangle counts are dumped beside the
file as `array_merge_truth.txt`.

- "Touching": a 1 m strip of 2 x 1 quads, rippled, under an Array of 5
  copies at relative offset 1 merging within 0.01, then a Subdivision of 1
  (limit surface off, which keeps the strip's one-face corners out of it):
  each copy's left edge lands on the one before's right edge.
- "Near": the same at relative offset 1.004, merging within 0.01: the
  copies stop 4 mm short and still merge (the target keeps its place).
- "Collapse": the strip under an Array of 4 copies at a constant 0.3 m,
  merging within 0.35: each copy's vertices go to the nearest of the copy
  before, so faces collapse and land on faces already there.
- "Partial": the same within 0.25: only the lower row's vertices reach.
- "Apart": as "Touching" at relative offset 1.05: too far, nothing merges.
- "Hub": one 192-corner n-gon, a wheel's hub cap (BC-15's `Front_Wheel`),
  with no modifier: 190 triangles.

Run: blender -b --python make_array_merge.py -- array_merge.blend"""
import bpy, bmesh, sys, os, math

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene


def strip(name, y):
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    vs = [[bm.verts.new((0.5 * i, float(j), 0.0)) for j in range(2)] for i in range(3)]
    for i in range(2):
        bm.faces.new((vs[i][0], vs[i + 1][0], vs[i + 1][1], vs[i][1]))
    vs[1][1].co.z = 0.2
    vs[1][0].co.z = -0.1
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    sc.collection.objects.link(ob)
    ob.location = (0.0, y, 0.0)
    return ob


def subsurf(ob):
    ss = ob.modifiers.new("Subdivision", "SUBSURF")
    ss.levels, ss.use_limit_surface = 1, False


def array(ob, count, dist, relative=None, constant=None):
    ar = ob.modifiers.new("Array", "ARRAY")
    ar.count, ar.use_merge_vertices, ar.merge_threshold = count, True, dist
    ar.use_relative_offset = relative is not None
    ar.use_constant_offset = constant is not None
    if relative is not None:
        ar.relative_offset_displace = (relative, 0.0, 0.0)
    if constant is not None:
        ar.constant_offset_displace = (constant, 0.0, 0.0)
    return ar


ob = strip("Touching", 0.0)
array(ob, 5, 0.01, relative=1.0)
subsurf(ob)
ob = strip("Near", 3.0)
array(ob, 5, 0.01, relative=1.004)
subsurf(ob)
ob = strip("Collapse", 6.0)
array(ob, 4, 0.35, constant=0.3)
ob = strip("Partial", 7.5)
array(ob, 4, 0.25, constant=0.3)
ob = strip("Apart", 9.0)
array(ob, 5, 0.01, relative=1.05)
subsurf(ob)

me = bpy.data.meshes.new("Hub")
bm = bmesh.new()
bmesh.ops.create_circle(bm, cap_ends=True, segments=192, radius=0.4)
bm.to_mesh(me)
bm.free()
hub = bpy.data.objects.new("Hub", me)
sc.collection.objects.link(hub)
hub.location = (0.0, 12.0, 0.0)

path = sys.argv[-1]
bpy.ops.wm.save_as_mainfile(filepath=path)

# Blender's own evaluated positions, object space, one line per object:
# name, vertices, triangles, coordinates.
dg = bpy.context.evaluated_depsgraph_get()
with open(os.path.join(os.path.dirname(os.path.abspath(path)), "array_merge_truth.txt"), "w") as out:
    for ob in sorted(sc.objects, key=lambda o: o.name):
        ev = ob.evaluated_get(dg).to_mesh()
        ev.calc_loop_triangles()
        cos = " ".join(f"{c:.6f}" for v in ev.vertices for c in v.co)
        out.write(f"{ob.name}\t{len(ev.vertices)}\t{len(ev.loop_triangles)}\t{cos}\n")
