"""Grids under the Mask modifier, one per setting the port tells apart, and
what Blender evaluates for each — for `tests/mask_truth.rs`.

The grid is 2 m, 10 x 10 vertices (81 quads), a little wavy in z, with two
materials in a checker and a "Mask" group whose weight is a bump across it
(0 at two corners, 1 in the middle), so the kept region is not convex.
Its edges are given to `from_pydata` in first-seen face order, the order
phiphx's edge list has, so the smooth cut's new vertices compare by index.
- "Group": Vertex Group mode, threshold 0.3.
- "Invert": the same, inverted.
- "Smooth": Smooth, threshold 0.3 — the faces that straddle it are cut.
- "Smooth Invert": Smooth, inverted, threshold 0.6.
- "Ngon": a single hexagon weighted 0, 1, 0, 1, 1, 0 under Smooth at 0.5:
  two runs, so two sub-faces out of one face.
- "Bones": Armature mode on a rig with bones "a" and "b" (groups "a" on
  x < 0, "b" on x > 0, overlapping in the middle), "b" alone selected,
  threshold 0.2.
- "Subsurf": a closed UV sphere (16 x 8, triangle fans at the poles) with
  "Mask" on its cap, under Subdivision 2 and then a Mask inverted at
  threshold 0 — the `EYE-L` stack in the CW-POSES files (its cornea cut
  away), on the subdivided weights. Closed, as the eye is, so no boundary
  rule of the Subdivision is in play.
- "Missing": a group name the mesh does not have: the mesh is unchanged.

Truth lines: name, vertex count, triangle count, each triangle's material
index in order, then every vertex.

Run: blender -b --python make_mask.py -- mask.blend mask.tsv"""
import bpy, math, sys

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
mats = [bpy.data.materials.new(n) for n in ("M0", "M1")]
N = 10


def first_seen_edges(faces):
    seen, edges = set(), []
    for f in faces:
        for k in range(len(f)):
            a, b = f[k], f[(k + 1) % len(f)]
            if (min(a, b), max(a, b)) not in seen:
                seen.add((min(a, b), max(a, b)))
                edges.append((a, b))
    return edges


def mesh_object(name, verts, faces, weights):
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, first_seen_edges(faces), faces)
    for m in mats:
        me.materials.append(m)
    for p in me.polygons:
        p.material_index = p.index % 2
    uv = me.uv_layers.new(name="UVMap")
    for loop in me.loops:
        co = verts[loop.vertex_index]
        uv.data[loop.index].uv = (co[0] * 0.5 + 0.5, co[1] * 0.5 + 0.5)
    ob = bpy.data.objects.new(name, me)
    sc.collection.objects.link(ob)
    for group, w in weights.items():
        g = ob.vertex_groups.new(name=group)
        for i, x in enumerate(w):
            if x > 0:
                g.add([i], x, "REPLACE")
    return ob


def grid(name, **groups):
    verts = [(-1 + 2 * i / (N - 1), -1 + 2 * j / (N - 1), 0.1 * math.sin(3 * i / N + 2 * j / N)) for j in range(N) for i in range(N)]
    faces = [(j * N + i, j * N + i + 1, (j + 1) * N + i + 1, (j + 1) * N + i) for j in range(N - 1) for i in range(N - 1)]
    bump = [max(0.0, 1.0 - math.hypot(x - 0.1, y + 0.05) / 1.1) for x, y, _ in verts]
    weights = {"Mask": bump}
    weights.update({g: [f(x) for x, _, _ in verts] for g, f in groups.items()})
    return mesh_object(name, verts, faces, weights)


def mask(ob, **settings):
    md = ob.modifiers.new("Mask", "MASK")
    for k, v in settings.items():
        setattr(md, k, v)
    return md


mask(grid("Group"), vertex_group="Mask", threshold=0.3)
mask(grid("Invert"), vertex_group="Mask", threshold=0.3, invert_vertex_group=True)
mask(grid("Smooth"), vertex_group="Mask", threshold=0.3, use_smooth=True)
mask(grid("Smooth Invert"), vertex_group="Mask", threshold=0.6, use_smooth=True, invert_vertex_group=True)
hexagon = [(math.cos(k * math.pi / 3), math.sin(k * math.pi / 3), 0) for k in range(6)]
ngon = mesh_object("Ngon", hexagon, [tuple(range(6))], {"Mask": [0, 1, 0, 1, 1, 0]})
mask(ngon, vertex_group="Mask", threshold=0.5, use_smooth=True)

arm = bpy.data.armatures.new("Arm")
rig = bpy.data.objects.new("Rig", arm)
sc.collection.objects.link(rig)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.mode_set(mode="EDIT")
for n, x in (("a", -0.5), ("b", 0.5)):
    eb = arm.edit_bones.new(n)
    eb.head, eb.tail = (x, 0, 0), (x, 0, 1)
bpy.ops.object.mode_set(mode="OBJECT")
for n, sel in (("a", False), ("b", True)):
    pb = rig.pose.bones[n]
    if hasattr(pb, "select"):
        pb.select = sel
    else:
        pb.bone.select = sel
bones = grid("Bones", a=lambda x: min(1.0, max(0.0, 0.6 - x)), b=lambda x: min(1.0, max(0.0, 0.6 + x)))
mask(bones, mode="ARMATURE", armature=rig, threshold=0.2)

SEG, RINGS = 16, 8
ball = [(0, 0, -1)] + [(math.sin(math.pi * r / RINGS) * math.cos(2 * math.pi * s / SEG),
                        math.sin(math.pi * r / RINGS) * math.sin(2 * math.pi * s / SEG),
                        -math.cos(math.pi * r / RINGS)) for r in range(1, RINGS) for s in range(SEG)] + [(0, 0, 1)]
ring = lambda r, s: 1 + (r - 1) * SEG + s % SEG
ball_faces = [(0, ring(1, s + 1), ring(1, s)) for s in range(SEG)]
ball_faces += [(ring(r, s), ring(r, s + 1), ring(r + 1, s + 1), ring(r + 1, s)) for r in range(1, RINGS - 1) for s in range(SEG)]
ball_faces += [(len(ball) - 1, ring(RINGS - 1, s), ring(RINGS - 1, s + 1)) for s in range(SEG)]
sub = mesh_object("Subsurf", ball, ball_faces, {"Mask": [max(0.0, z - 0.55) for _, _, z in ball]})
sub.modifiers.new("Subdivision", "SUBSURF").levels = 2
mask(sub, vertex_group="Mask", invert_vertex_group=True, threshold=0.0)
mask(grid("Missing"), vertex_group="Missing")

bpy.ops.wm.save_as_mainfile(filepath=out)
graph = bpy.context.evaluated_depsgraph_get()
with open(truth, "w") as f:
    for ob in sc.objects:
        if ob.type != "MESH":
            continue
        mesh = ob.evaluated_get(graph).to_mesh()
        mesh.calc_loop_triangles()
        tris = "".join(str(mesh.polygons[t.polygon_index].material_index) for t in mesh.loop_triangles)
        coords = " ".join(f"{c:.9g}" for v in mesh.vertices for c in v.co)
        f.write(f"{ob.name}\t{len(mesh.vertices)}\t{len(mesh.loop_triangles)}\t{tris}\t{coords}\n")
