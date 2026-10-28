"""Curve objects with bevel, extrusion, fill, caps, profile, bevel object
and taper, for the curve-to-mesh path (`blend/curve/displist`,
`tests/curve_mesh_truth.rs`):

- "Bezier Round Caps": an open 3D Bézier with tilt and radius on its
  points and one vector-handle segment, a round bevel of 0.1 (resolution 3),
  caps filled, minimum twist.
- "Bezier Tangent Half": an open 3D Bézier, tangent twist, fill HALF,
  extruded 0.05 with a 0.04 bevel.
- "Closed 2D Extrude Fill": a flat closed Bézier with a hole spline,
  extruded 0.2 with a 0.03 bevel, fill BOTH, offset -0.02.
- "Open 2D Front": a flat open Bézier, fill FRONT, extruded 0.1 with a
  0.05 bevel, caps filled.
- "Closed 2D Back Flat": a flat closed poly, fill BACK, no bevel: the
  outline filled alone.
- "Closed 2D None": a flat closed Bézier, fill NONE, a 0.04 bevel.
- "Profile Bevel": a closed 3D poly with a custom profile bevel of 0.1
  (resolution 4), as BC-15's roofs have.
- "Bevel Object": an open 3D Bézier bevelled by "Bevel Shape" (a closed
  flat Bézier, scaled), caps filled.
- "Tapered": an open 3D Bézier, 0.08 bevel, tapered by "Taper Shape",
  radius multiplied, caps filled.
- "Poly Spline": an open 3D poly, Z-up twist, 0.05 bevel, under an Array
  of 2.
- "NURBS Spline": a cyclic order-4 NURBS and an open endpoint one, 0.05
  bevel, extruded 0.02, minimum twist smoothed 2, caps filled.

Run: blender -b --python make_curve_mesh.py -- curve_mesh.blend"""
import bpy, sys

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene


def curve(name, dims="3D", loc=(0, 0, 0)):
    cu = bpy.data.curves.new(name, "CURVE")
    cu.dimensions = dims
    ob = bpy.data.objects.new(name, cu)
    ob.location = loc
    sc.collection.objects.link(ob)
    return ob, cu


def bezier(cu, pts, cyclic=False, resolution=8, vector=()):
    """`pts`: (co, left, right[, tilt, radius]); `vector`: point indices
    with both handles VECTOR."""
    sp = cu.splines.new("BEZIER")
    sp.bezier_points.add(len(pts) - 1)
    for i, p in enumerate(pts):
        b = sp.bezier_points[i]
        t = "VECTOR" if i in vector else "FREE"
        b.handle_left_type = t
        b.handle_right_type = t
        b.co = p[0]
        b.handle_left = p[1]
        b.handle_right = p[2]
        if len(p) > 3:
            b.tilt = p[3]
            b.radius = p[4]
    sp.use_cyclic_u = cyclic
    sp.resolution_u = resolution
    return sp


def poly(cu, pts, cyclic=False, kind="POLY", order=4, endpoint=False, resolution=6):
    sp = cu.splines.new(kind)
    sp.points.add(len(pts) - 1)
    for i, p in enumerate(pts):
        sp.points[i].co = (p[0], p[1], p[2], p[3] if len(p) > 3 else 1.0)
        if len(p) > 4:
            sp.points[i].tilt = p[4]
    sp.use_cyclic_u = cyclic
    if kind == "NURBS":
        sp.order_u = order
        sp.use_endpoint_u = endpoint
    sp.resolution_u = resolution
    return sp


def sym(c, d):
    """A point with handles mirrored by `d`."""
    return (c, tuple(a - b for a, b in zip(c, d)), tuple(a + b for a, b in zip(c, d)))


# 1. Open 3D Bézier, round bevel, caps, tilt and radius.
ob, cu = curve("Bezier Round Caps", loc=(0, 0, 0))
bezier(cu, [
    sym((0, 0, 0), (0.4, 0.1, 0.2)) + (0.0, 1.0),
    sym((1.5, 0.8, 0.5), (0.3, 0.4, -0.1)) + (0.6, 0.7),
    sym((2.5, -0.3, 1.2), (0.5, 0, 0.3)) + (-0.4, 1.3),
    sym((3.4, 0.5, 0.6), (0.2, 0.3, 0)) + (0.2, 1.0),
], vector=(3,))
cu.bevel_depth = 0.1
cu.bevel_resolution = 3
cu.use_fill_caps = True
cu.twist_mode = "MINIMUM"

# 2. Open 3D Bézier, tangent twist, HALF.
ob, cu = curve("Bezier Tangent Half", loc=(0, 3, 0))
bezier(cu, [
    sym((0, 0, 0), (0.5, 0.2, 0.3)),
    sym((1.2, 1.0, 0.4), (0.4, 0.2, 0.4)),
    sym((2.4, 0.2, -0.3), (0.3, -0.4, 0.2)),
], resolution=10)
cu.twist_mode = "TANGENT"
cu.fill_mode = "HALF"
cu.extrude = 0.05
cu.bevel_depth = 0.04
cu.bevel_resolution = 2

# 3. Closed 2D with a hole, extruded and bevelled, fill BOTH, offset.
ob, cu = curve("Closed 2D Extrude Fill", "2D", loc=(0, 6, 0))
bezier(cu, [sym((0, 0, 0), (0.5, -0.3, 0)), sym((1.5, 0.2, 0), (0.2, 0.5, 0)),
            sym((1.0, 1.4, 0), (-0.5, 0.1, 0)), sym((-0.2, 1.0, 0), (-0.1, -0.5, 0))], cyclic=True)
bezier(cu, [sym((0.4, 0.4, 0), (0.15, 0, 0)), sym((0.9, 0.5, 0), (0, 0.15, 0)),
            sym((0.5, 0.9, 0), (-0.15, 0, 0))], cyclic=True, resolution=6)
cu.extrude = 0.2
cu.bevel_depth = 0.03
cu.bevel_resolution = 2
cu.fill_mode = "BOTH"
cu.offset = -0.02

# 4. Open 2D, fill FRONT, caps.
ob, cu = curve("Open 2D Front", "2D", loc=(4, 6, 0))
bezier(cu, [sym((0, 0, 0), (0.4, 0.3, 0)), sym((1.2, 0.6, 0), (0.3, -0.3, 0)), sym((2.0, 0.0, 0), (0.3, 0.2, 0))])
cu.fill_mode = "FRONT"
cu.extrude = 0.1
cu.bevel_depth = 0.05
cu.bevel_resolution = 1
cu.use_fill_caps = True

# 5. Closed 2D poly, fill BACK, no bevel.
ob, cu = curve("Closed 2D Back Flat", "2D", loc=(8, 6, 0))
poly(cu, [(0, 0, 0), (1, 0, 0), (1.3, 0.8, 0), (0.5, 0.4, 0), (-0.1, 1.0, 0)], cyclic=True)
cu.fill_mode = "BACK"

# 6. Closed 2D, fill NONE, bevel.
ob, cu = curve("Closed 2D None", "2D", loc=(12, 6, 0))
bezier(cu, [sym((0, 0, 0), (0.4, -0.2, 0)), sym((1.0, 0.6, 0), (0, 0.4, 0)), sym((0, 1.0, 0), (-0.4, 0, 0))], cyclic=True)
cu.fill_mode = "NONE"
cu.bevel_depth = 0.04
cu.bevel_resolution = 2

# 7. Closed 3D poly, custom profile bevel.
ob, cu = curve("Profile Bevel", loc=(0, 10, 0))
poly(cu, [(0, 0, 0), (2, 0, 0.2), (2.2, 1.5, 0.1), (0.8, 2.0, 0.4), (-0.3, 1.0, 0)], cyclic=True)
cu.bevel_mode = "PROFILE"
cu.bevel_depth = 0.1
cu.bevel_resolution = 4
prof = cu.bevel_profile
prof.points.add(0.7, 0.2)
prof.points.add(0.3, 0.75)
for p in prof.points:
    p.handle_type_1 = "VECTOR" if p.location[0] > 0.5 else "AUTO"
    p.handle_type_2 = p.handle_type_1
prof.update()

# 8. Bevel object.
bev_ob, bev_cu = curve("Bevel Shape", "2D", loc=(0, 14, 0))
bezier(bev_cu, [sym((-0.1, 0, 0), (0, -0.05, 0)), sym((0, -0.08, 0), (-0.06, 0, 0)),
                sym((0.12, 0, 0), (0, 0.06, 0)), sym((0, 0.1, 0), (0.08, 0, 0))], cyclic=True, resolution=4)
bev_ob.scale = (1.5, 0.7, 1.0)
ob, cu = curve("Bevel Object", loc=(0, 16, 0))
bezier(cu, [sym((0, 0, 0), (0.5, 0, 0.3)), sym((1.5, 1.0, 0.5), (0.5, 0.2, 0)), sym((3, 0, 0), (0.4, -0.4, -0.2))])
cu.bevel_mode = "OBJECT"
cu.bevel_object = bev_ob
cu.use_fill_caps = True

# 9. Taper object.
tap_ob, tap_cu = curve("Taper Shape", "2D", loc=(0, 20, 0))
bezier(tap_cu, [((0, 0.3, 0), (-0.2, 0.3, 0), (0.2, 0.6, 0)), ((0.5, 1.2, 0), (0.3, 1.2, 0), (0.7, 1.2, 0)),
                ((1, 0.2, 0), (0.8, 0.5, 0), (1.2, 0, 0))], resolution=6)
ob, cu = curve("Tapered", loc=(0, 22, 0))
bezier(cu, [sym((0, 0, 0), (0.5, 0.3, 0)) + (0.0, 1.0), sym((1.6, 0.5, 0.4), (0.4, 0, 0.2)) + (0.0, 0.5),
            sym((3, -0.2, 0), (0.4, -0.3, 0)) + (0.0, 1.5)])
cu.bevel_depth = 0.08
cu.bevel_resolution = 2
cu.taper_object = tap_ob
cu.taper_radius_mode = "MULTIPLY"
cu.use_fill_caps = True

# 10. Poly spline under an Array.
ob, cu = curve("Poly Spline", loc=(0, 26, 0))
poly(cu, [(0, 0, 0), (1, 0.3, 0.2, 1, 0.3), (1.8, 1.0, 0.0, 1, -0.2), (2.6, 0.6, 0.6), (3.2, 1.2, 0.3)])
cu.twist_mode = "Z_UP"
cu.bevel_depth = 0.05
cu.bevel_resolution = 2
mod = ob.modifiers.new("Array", "ARRAY")
mod.count = 2

# 11. NURBS splines.
ob, cu = curve("NURBS Spline", loc=(0, 30, 0))
poly(cu, [(0, 0, 0), (1, -0.2, 0.3), (1.6, 0.8, 0), (0.8, 1.6, -0.3, 2.0), (-0.3, 1.0, 0.2)], cyclic=True, kind="NURBS", resolution=5)
poly(cu, [(3, 0, 0), (3.5, 0.6, 0.2), (4.2, 0.2, 0.5), (4.8, 0.9, 0), (5.2, 0.1, 0.1)], kind="NURBS", endpoint=True, resolution=4)
cu.bevel_depth = 0.05
cu.bevel_resolution = 2
cu.extrude = 0.02
cu.twist_mode = "MINIMUM"
cu.twist_smooth = 2.0
cu.use_fill_caps = True

bpy.ops.wm.save_as_mainfile(filepath=sys.argv[sys.argv.index("--") + 1])
