# Makes `fixtures/crease.blend` and `truth/crease.txt` (Blender 5.0): cubes
# under a Subdivision Surface with creases of every kind the rules tell
# apart, and every vertex of each as Blender evaluates it — for
# `tests/subsurf_creases.rs`. Run with the bpy module:
#
#   python3 blender_crease.py ../fixtures/crease.blend crease.txt
#
# - Semi: the top four edges at crease 0.5 (sharpness 2.5: sharp for two
#   levels, a half at the third), three levels.
# - Mixed: one edge at 0.3 (sharpness 0.9, a fraction at once), one at 1.0
#   (infinitely sharp), and a vertex at 0.7 (sharpness 4.9), two levels.
# - Chain: three creased edges meeting at one corner (a corner rule), one of
#   them fractional, two levels, limit surface off.
# - Off: Semi's creases with "Use Creases" off.
# - Deep: the top edges at 0.6 (sharpness 3.6), two levels at quality 2,
#   so the sharpness left at the second level (1.6) is dropped.
# - Shallow: an edge at 0.7 (sharpness 4.9) and a vertex at 0.5 (2.5), one
#   level at quality 4, so the limit is taken three levels finer.
# - Gregory: no creases, four levels at quality 2 — Blender's grid past the
#   isolation level, on OpenSubdiv's Gregory end caps.
import bpy, sys

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_homefile(app_template="")
for ob in list(bpy.data.objects):
    bpy.data.objects.remove(ob)

CUBE = [(-1, -1, -1), (1, -1, -1), (1, 1, -1), (-1, 1, -1), (-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1)]
FACES = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (2, 3, 7, 6), (0, 4, 7, 3), (1, 2, 6, 5)]


def cube(name, x, edges, verts, levels, creases=True, limit=True, quality=3):
    me = bpy.data.meshes.new(name)
    me.from_pydata([(v[0] + x, v[1], v[2]) for v in CUBE], [], FACES)
    ce = me.attributes.new("crease_edge", "FLOAT", "EDGE")
    for e in me.edges:
        ce.data[e.index].value = edges.get(tuple(sorted(e.vertices)), 0.0)
    if verts:
        cv = me.attributes.new("crease_vert", "FLOAT", "POINT")
        for v, c in verts.items():
            cv.data[v].value = c
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    md = ob.modifiers.new("Subdivision", "SUBSURF")
    md.levels, md.render_levels = levels, levels
    md.use_creases, md.use_limit_surface, md.quality = creases, limit, quality
    return ob


top = {(4, 5): 0.5, (5, 6): 0.5, (6, 7): 0.5, (4, 7): 0.5}
cube("Semi", 0, top, None, 3)
cube("Mixed", 4, {(0, 1): 0.3, (2, 6): 1.0}, {7: 0.7}, 2)
cube("Chain", 8, {(1, 5): 1.0, (5, 6): 1.0, (4, 5): 0.4}, None, 2, limit=False)
cube("Off", 12, top, None, 3, creases=False)
cube("Deep", 16, {k: 0.6 for k in top}, None, 2, quality=2)
cube("Gregory", 24, {}, None, 4, quality=2)
cube("Shallow", 20, {(0, 1): 0.7}, {2: 0.5}, 1, quality=4)

bpy.ops.wm.save_as_mainfile(filepath=out)
graph = bpy.context.evaluated_depsgraph_get()
with open(truth, "w") as f:
    for ob in bpy.context.scene.objects:
        mesh = ob.evaluated_get(graph).to_mesh()
        f.write(f"{ob.name}\t{len(mesh.vertices)}\t" + " ".join(f"{c:.6f}" for v in mesh.vertices for c in v.co) + "\n")
