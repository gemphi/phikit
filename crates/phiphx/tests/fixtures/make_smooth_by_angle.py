"""Make `smooth_by_angle.blend` and `../truth/smooth_by_angle.txt` (Blender 5.2):
meshes under the bundled "Smooth by Angle" Geometry Nodes group, one per
case the node tree tells apart, and what Blender evaluates for each
(`../truth/blender_smooth_by_angle.py`'s `dump`) — for
`tests/smooth_by_angle_truth.rs`.

    blender -b --python make_smooth_by_angle.py -- smooth_by_angle.blend ../truth/smooth_by_angle.txt

- Cube30 / Cube100: a cube at 30° (every edge sharp) and 100° (none).
- Cyl20 / Cyl45: a 12-sided cylinder with 12-gon caps — sides meet at 30°,
  the caps at 90° — at 20° (all sharp) and 45° (only the cap rims).
- Sphere40: a UV sphere, 8 segments by 6 rings, at 40°: faces meet at 45°
  round a ring and 30° along a meridian, so the meridian edges go sharp
  and the ring edges stay smooth, with triangle fans at the poles.
- Grid10: a rippled 6 x 6 grid (non-planar quads) at 10°, with a loose
  edge and a loose vertex, which Blender marks sharp.
- Marked / Ignore: a cube at 100° with two edges already sharp — kept when
  "Ignore Sharpness" is off, cleared when it is on.
- Flat / IgnoreFlat: a cube at 100° with two faces flat-shaded — their
  shared edge goes sharp through the face → edge rule when "Ignore
  Sharpness" is off; everything is smooth when it is on.
- Fan: three quads round one edge (non-manifold: angle 0, but the corner
  normals split there) and a fourth folded 60° off one of them, at 30°.
"""
import glob
import math
import os
import sys

import bpy
import bmesh

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene

root = bpy.utils.system_resource("DATAFILES")
for hit in glob.glob(os.path.join(root, "assets", "**", "*.blend"), recursive=True):
    with bpy.data.libraries.load(hit, link=False) as (src, dst):
        dst.node_groups = [n for n in src.node_groups if n == "Smooth by Angle"]
GROUP = bpy.data.node_groups["Smooth by Angle"]

CUBE = [(-1, -1, -1), (1, -1, -1), (1, 1, -1), (-1, 1, -1), (-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1)]
CUBE_FACES = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (2, 3, 7, 6), (0, 4, 7, 3), (1, 2, 6, 5)]


def set_input(md, name, value):
    ident = next(i.identifier for i in md.node_group.interface.items_tree if i.item_type == "SOCKET" and i.name == name)
    try:
        md[ident] = value
    except TypeError:
        setattr(getattr(md.properties.inputs, ident), "value", value)


def add(name, x, verts, faces, angle, ignore=False, sharp=(), flat=(), edges=()):
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, list(edges), faces)
    # `from_pydata` makes flat faces; every case starts smooth-shaded but the
    # faces a case names flat.
    for p in me.polygons:
        p.use_smooth = True
    if sharp:
        want = {tuple(sorted(e)) for e in sharp}
        attr = me.attributes.new("sharp_edge", "BOOLEAN", "EDGE")
        for e in me.edges:
            attr.data[e.index].value = tuple(sorted(e.vertices)) in want
    for f in flat:
        me.polygons[f].use_smooth = False
    ob = bpy.data.objects.new(name, me)
    sc.collection.objects.link(ob)
    ob.location = (x, 0.0, 0.0)
    md = ob.modifiers.new("Smooth by Angle", "NODES")
    md.node_group = GROUP
    set_input(md, "Angle", math.radians(angle))
    set_input(md, "Ignore Sharpness", ignore)
    return ob


def cylinder(n=12):
    ring = lambda z: [(math.cos(2 * math.pi * i / n), math.sin(2 * math.pi * i / n), z) for i in range(n)]
    verts = ring(-1.0) + ring(1.0)
    faces = [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    faces.append(tuple(reversed(range(n))))
    faces.append(tuple(range(n, 2 * n)))
    return verts, faces


def sphere():
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=8, v_segments=6, radius=1.0)
    verts = [tuple(v.co) for v in bm.verts]
    faces = [tuple(v.index for v in f.verts) for f in bm.faces]
    bm.free()
    return verts, faces


def grid(n=6):
    verts = []
    for j in range(n + 1):
        for i in range(n + 1):
            x, y = 2.0 * i / n - 1.0, 2.0 * j / n - 1.0
            verts.append((x, y, 0.12 * math.sin(2.3 * x + 0.3) * math.cos(1.9 * y)))
    faces = [(j * (n + 1) + i, j * (n + 1) + i + 1, (j + 1) * (n + 1) + i + 1, (j + 1) * (n + 1) + i) for j in range(n) for i in range(n)]
    loose = len(verts)
    verts += [(1.5, -1.0, 0.0), (1.5, 1.0, 0.0), (1.8, 0.0, 0.3)]
    return verts, faces, [(loose, loose + 1)]


FAN = [(0, 0, 0), (0, 2, 0), (2, 0, 0), (2, 2, 0), (0, 0, 2), (0, 2, 2), (-2, 0, 0), (-2, 2, 0),
       (3, 0, math.sqrt(3)), (3, 2, math.sqrt(3))]
FAN_FACES = [(0, 2, 3, 1), (0, 1, 5, 4), (0, 6, 7, 1), (2, 8, 9, 3)]

add("Cube30", 0.0, CUBE, CUBE_FACES, 30)
add("Cube100", 3.0, CUBE, CUBE_FACES, 100)
cv, cf = cylinder()
add("Cyl20", 6.0, cv, cf, 20)
add("Cyl45", 9.0, cv, cf, 45)
sv, sf = sphere()
add("Sphere40", 12.0, sv, sf, 40)
gv, gf, ge = grid()
add("Grid10", 15.0, gv, gf, 10, edges=ge)
add("Marked", 18.0, CUBE, CUBE_FACES, 100, sharp=[(4, 5), (6, 7)])
add("Ignore", 21.0, CUBE, CUBE_FACES, 100, ignore=True, sharp=[(4, 5), (6, 7)])
add("Flat", 24.0, CUBE, CUBE_FACES, 100, flat=[0, 2])
add("IgnoreFlat", 27.0, CUBE, CUBE_FACES, 100, ignore=True, flat=[0, 2])
add("Fan", 30.0, FAN, FAN_FACES, 30)

bpy.ops.wm.save_as_mainfile(filepath=os.path.abspath(out))
exec(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "truth", "blender_smooth_by_angle.py")).read().split('if __name__')[0])
dump([ob for ob in bpy.data.objects if ob.type == "MESH"], os.path.abspath(truth))
