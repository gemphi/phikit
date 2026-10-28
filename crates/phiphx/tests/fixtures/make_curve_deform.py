"""Arrayed bars bent along curves by the Curve modifier — every deform axis,
the curve flags the deform reads, and vertex-group weighting — and every
evaluated vertex as Blender puts it, for `tests/curve_deform_truth.rs`.

Each bar is a 0.3 × 0.2 × 0.1 box with one loop cut, arrayed 24 times
along the axis it bends on (Array → Curve), and has a curve of its own,
because the flags live on the curve. Bars and curves stand at transforms of
their own, so `init_curve_deform`'s object-space matrices are exercised.

- "PosX": +X along an open 3D Bézier S with varied radius and tilt,
  Blender's default flags (`CU_DEFORM_BOUNDS_OFF | CU_PATH_RADIUS`).
- "NegY": −Y, bounds on (the bar's own box), `CU_STRETCH`: the bar is
  stretched over the whole curve.
- "PosZ": +Z, bounds on, radius off, the bar longer than the curve, so it
  runs past the open end.
- "NegX": −X round a cyclic Bézier circle (2D), bounds on, radius on — the
  bar wraps more than once.
- "PosY": +Y, a vertex group ramping 0 → 1 along the bar, `CU_PATH` and
  `CU_PATH_CLAMP` set on the curve (the deform ignores the clamp).
- "NegZ": −Z along a NURBS curve, the same ramp inverted, `CU_STRETCH`.

Run: blender -b --python make_curve_deform.py -- curve_deform.blend curve_deform.txt"""
import bpy, bmesh, sys, math

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
AXES = {"POS_X": 0, "POS_Y": 1, "POS_Z": 2, "NEG_X": 0, "NEG_Y": 1, "NEG_Z": 2}


def bar(name, axis, loc, rot, ramp=False):
    """A box, one loop cut across the bend axis, arrayed along it."""
    k = AXES[axis]
    size = [0.2, 0.1, 0.1]
    size[k] = 0.3
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        v.co = [v.co[i] * size[i] for i in range(3)]
    cut = [e for e in bm.edges if abs((e.verts[0].co - e.verts[1].co)[k]) > 1e-6]
    bmesh.ops.subdivide_edges(bm, edges=cut, cuts=1, use_grid_fill=True)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    sc.collection.objects.link(ob)
    ob.location, ob.rotation_euler = loc, rot
    arr = ob.modifiers.new("Array", "ARRAY")
    arr.count = 24
    off = [0.0, 0.0, 0.0]
    off[k] = 1.0
    arr.relative_offset_displace = off
    return ob


def ramp_group(ob, axis):
    """Weights 0 at the bar's start to 1 at its end (after the array, by the
    copy each vertex lands in — set per base vertex here, so the ramp is
    across one box, repeated)."""
    k = AXES[axis]
    g = ob.vertex_groups.new(name="Ramp")
    lo = min(v.co[k] for v in ob.data.vertices)
    hi = max(v.co[k] for v in ob.data.vertices)
    for v in ob.data.vertices:
        g.add([v.index], (v.co[k] - lo) / (hi - lo), "REPLACE")
    return g


def bezier(name, pts, loc, rot, cyclic=False, dims="3D", radii=None, tilts=None):
    cu = bpy.data.curves.new(name, "CURVE")
    cu.dimensions = dims
    sp = cu.splines.new("BEZIER")
    sp.bezier_points.add(len(pts) - 1)
    for i, p in enumerate(pts):
        bp = sp.bezier_points[i]
        bp.co = p
        bp.handle_left_type = bp.handle_right_type = "AUTO"
        bp.radius = radii[i] if radii else 1.0
        bp.tilt = tilts[i] if tilts else 0.0
    sp.use_cyclic_u = cyclic
    ob = bpy.data.objects.new(name, cu)
    sc.collection.objects.link(ob)
    ob.location, ob.rotation_euler = loc, rot
    return ob, cu


def nurbs(name, pts, loc, rot):
    cu = bpy.data.curves.new(name, "CURVE")
    cu.dimensions = "3D"
    sp = cu.splines.new("NURBS")
    sp.points.add(len(pts) - 1)
    for i, p in enumerate(pts):
        sp.points[i].co = (*p, 1.0)
    sp.order_u, sp.use_endpoint_u = 4, True
    ob = bpy.data.objects.new(name, cu)
    sc.collection.objects.link(ob)
    ob.location, ob.rotation_euler = loc, rot
    return ob, cu


def bend(ob, curve, axis, group=None, invert=False):
    md = ob.modifiers.new("Curve", "CURVE")
    md.object, md.deform_axis = curve, axis
    if group:
        md.vertex_group, md.invert_vertex_group = group, invert
    return md


S = [(0, 0, 0), (2, 1.5, 0.5), (4, -1, 1), (6, 0.5, 0), (8, 0, -0.5)]

ob = bar("PosX", "POS_X", (0.5, 0.2, 0.1), (0.1, 0.0, 0.2))
cob, cu = bezier("Curve PosX", S, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), radii=[1, 1.5, 0.5, 2, 1], tilts=[0, 0.5, -0.3, 1.0, 0])
bend(ob, cob, "POS_X")

ob = bar("NegY", "NEG_Y", (0.0, 10.0, 0.0), (0.0, 0.0, 0.0))
cob, cu = bezier("Curve NegY", [(p[1], -p[0], p[2]) for p in S], (0.3, 10.2, -0.4), (0.0, 0.3, 0.0), tilts=[0, 0.3, 0.6, 0.3, 0])
cu.use_deform_bounds, cu.use_stretch = True, True
bend(ob, cob, "NEG_Y")

ob = bar("PosZ", "POS_Z", (20.0, 0.0, 0.0), (0.0, 0.2, 0.0))
cob, cu = bezier("Curve PosZ", [(0, 0, 0), (1, 0.5, 2), (0, 1, 4)], (20.0, 0.5, 0.0), (0.0, 0.0, 0.4), radii=[1, 2, 1])
cu.use_deform_bounds, cu.use_radius = True, False
bend(ob, cob, "POS_Z")

ob = bar("NegX", "NEG_X", (0.0, -12.0, 0.0), (0.0, 0.0, 0.0))
r = 2.0
circle = [(r * math.cos(a), r * math.sin(a), 0.0) for a in (0, math.pi / 2, math.pi, 3 * math.pi / 2)]
cob, cu = bezier("Curve NegX", circle, (0.0, -12.0, 0.5), (0.2, 0.0, 0.0), cyclic=True, dims="2D", radii=[1, 1.4, 0.8, 1.2])
cu.use_deform_bounds = True
bend(ob, cob, "NEG_X")

ob = bar("PosY", "POS_Y", (-10.0, 0.0, 0.0), (0.0, 0.0, 0.3))
g = ramp_group(ob, "POS_Y")
cob, cu = bezier("Curve PosY", [(p[1], p[0], -p[2]) for p in S], (-10.0, 0.0, 0.0), (0.0, 0.0, 0.3), radii=[1, 0.6, 1.4, 1, 1])
cu.use_path, cu.use_path_clamp = True, True
bend(ob, cob, "POS_Y", g.name)

ob = bar("NegZ", "NEG_Z", (10.0, -10.0, 0.0), (0.3, 0.0, 0.0))
g = ramp_group(ob, "NEG_Z")
cob, cu = nurbs("Curve NegZ", [(0, 0, 0), (1, 0, -1.5), (0, 1, -3), (-1, 0, -4.5), (0, 0, -6)], (10.0, -10.0, 0.0), (0.0, 0.0, 0.0))
cu.use_stretch = True
bend(ob, cob, "NEG_Z", g.name, invert=True)

bpy.ops.wm.save_as_mainfile(filepath=out)
graph = bpy.context.evaluated_depsgraph_get()
with open(truth, "w") as f:
    for ob in sc.objects:
        if ob.type != "MESH":
            continue
        mesh = ob.evaluated_get(graph).to_mesh()
        f.write(f"{ob.name}\t{len(mesh.vertices)}\t" + " ".join(repr(c) for v in mesh.vertices for c in v.co) + "\n")
