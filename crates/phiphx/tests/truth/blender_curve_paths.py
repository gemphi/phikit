# Makes `fixtures/curve_paths.blend` and `truth/curve_paths.txt` (Blender
# 5.0): objects that ride curve paths — children of a curve with Path
# Animation (`ob_parcurve`) and objects under Follow Path — over curves of
# every kind a path can be: a closed 3D Bézier ring whose evaluation time a
# Generator drives (as THYMUS's turntable path does), an open NURBS path with
# tilt and radius, a poly line with smoothed twist, a flat closed Bézier, and
# an open Bézier with vector handles and tangent twist. Every object's world
# matrix at six frames; the file is saved at frame 12. For
# `blend/curve/tests.rs`. Run with the bpy module:
#
#   python3 blender_curve_paths.py ../fixtures/curve_paths.blend curve_paths.txt
import bpy, sys
from mathutils import Matrix

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_homefile(app_template="")
for ob in list(bpy.data.objects):
    bpy.data.objects.remove(ob)
scene = bpy.context.scene
scene.frame_start, scene.frame_end = 1, 50


def curve(name, dims="3D", loc=(0, 0, 0), rot=(0, 0, 0), scale=(1, 1, 1)):
    cu = bpy.data.curves.new(name, "CURVE")
    cu.dimensions = dims
    cu.use_path = True
    ob = bpy.data.objects.new(name, cu)
    scene.collection.objects.link(ob)
    ob.location, ob.rotation_euler, ob.scale = loc, rot, scale
    return ob, cu


def bezier(cu, pts, cyclic, types=("FREE", "FREE")):
    sp = cu.splines.new("BEZIER")
    sp.bezier_points.add(len(pts) - 1)
    for bp, (co, hl, hr, tilt, radius) in zip(sp.bezier_points, pts):
        bp.handle_left_type, bp.handle_right_type = types
        bp.co, bp.handle_left, bp.handle_right = co, hl, hr
        bp.tilt, bp.radius = tilt, radius
    sp.use_cyclic_u = cyclic
    sp.resolution_u = 12
    return sp


def empty(name, parent=None, loc=(0, 0, 0), track=None, up=None, inverse=None):
    o = bpy.data.objects.new(name, None)
    scene.collection.objects.link(o)
    o.location = loc
    if track:
        o.track_axis = track
    if up:
        o.up_axis = up
    if parent:
        o.parent = parent
        o.matrix_parent_inverse = inverse or Matrix.Identity(4)
    return o


def keyed_time(cu, keys):
    for f, v in keys:
        cu.eval_time = v
        cu.keyframe_insert("eval_time", frame=f)


# A closed 3D ring, its time a Generator over the frame.
ring, cu = curve("Ring", loc=(0.5, -0.5, 1.0), rot=(0.2, 0.0, 0.3), scale=(3, 3, 3))
k = 0.5523
bezier(cu, [
    ((-1, 0, 0), (-1, -k, 0), (-1, k, 0.1), 0.0, 1.0),
    ((0, 1, 0.2), (-k, 1, 0.2), (k, 1, 0.2), 0.4, 1.3),
    ((1, 0, 0), (1, k, 0.1), (1, -k, -0.1), -0.3, 0.8),
    ((0, -1, -0.2), (k, -1, -0.2), (-k, -1, -0.2), 0.2, 1.1),
], True)
cu.use_path_follow, cu.use_radius, cu.path_duration = True, True, 60
cu.twist_mode = "MINIMUM"
cu.animation_data_create()
act = bpy.data.actions.new("RingTime")
cu.animation_data.action = act
fc = act.fcurve_ensure_for_datablock(cu, "eval_time")
fc.modifiers.new("GENERATOR")
empty("Rider", ring, (0.0, 0.2, 0.0), inverse=Matrix.Rotation(0.3, 4, "Z"))
empty("RiderCam", ring, track="NEG_Z", up="Y")
empty("RiderX", ring, (0.1, 0, 0), track="POS_X", up="Z")

# An open NURBS path, tilted and with radius, clamped.
snake, cu = curve("Snake", loc=(-2, 1, 0))
sp = cu.splines.new("NURBS")
sp.points.add(5)
for p, (co, tilt, r) in zip(sp.points, [((0, 0, 0, 1), 0.0, 1.0), ((1, 1, 0.5, 1), 0.5, 1.5), ((2, 0, 1, 2), -0.4, 0.7),
                                        ((3, -1, 0.5, 1), 0.8, 1.2), ((4, 0, 0, 1), 0.1, 1.0), ((5, 1, -0.5, 1), -0.2, 0.9)]):
    p.co, p.tilt, p.radius = co, tilt, r
sp.order_u, sp.use_endpoint_u, sp.resolution_u = 4, True, 8
cu.use_path_follow, cu.use_path_clamp, cu.use_radius, cu.path_duration = True, True, True, 40
cu.twist_mode = "Z_UP"
keyed_time(cu, [(1, -5.0), (45, 48.0)])
empty("Crawler", snake, track="POS_Y", up="Z")

# A poly line, smoothed minimum twist, not following: position only.
zig, cu = curve("Zig", loc=(1, 2, -1))
sp = cu.splines.new("POLY")
sp.points.add(4)
for p, co in zip(sp.points, [(0, 0, 0, 1), (1, 0.5, 0.2, 1), (2, -0.5, 0.6, 1), (3, 0.5, 0.2, 1), (4, 0, 1, 1)]):
    p.co = co
cu.twist_mode, cu.twist_smooth, cu.path_duration = "MINIMUM", 2.0, 30
cu.use_path_follow = False
keyed_time(cu, [(1, 0.0), (30, 30.0)])
empty("Zigger", zig, (0, 0, 0.2))

# A flat closed Bézier drawn clockwise (Blender turns it round).
flat, cu = curve("Flat", dims="2D", loc=(0, -3, 0))
bezier(cu, [
    ((0, 1, 0), (-0.5, 1, 0), (0.5, 1, 0), 0.0, 1.0),
    ((1, -1, 0), (1.3, -0.6, 0), (0.7, -1.4, 0), 0.0, 1.0),
    ((-1, -1, 0), (-0.7, -1.4, 0), (-1.3, -0.6, 0), 0.0, 1.0),
], True)
cu.use_path_follow, cu.path_duration = True, 25
keyed_time(cu, [(1, 3.0), (25, 28.0)])
empty("Flatter", flat, track="NEG_Y", up="X")

# An open Bézier with a vector corner and tangent twist.
hook, cu = curve("Hook", loc=(3, 0, 0))
sp = bezier(cu, [
    ((0, 0, 0), (-0.3, -0.3, 0), (0.3, 0.3, 0.2), 0.0, 1.0),
    ((1, 1, 0.5), (0.6, 0.8, 0.4), (1.4, 1.2, 0.6), 0.3, 1.0),
    ((2, 0, 0), (1.7, 0.4, 0), (2.3, -0.4, 0), 0.0, 1.0),
], False)
sp.bezier_points[1].handle_left_type = "VECTOR"
sp.bezier_points[0].handle_right_type = "VECTOR"
cu.twist_mode, cu.use_path_follow, cu.path_duration = "TANGENT", True, 20
keyed_time(cu, [(1, -2.0), (40, 26.0)])
empty("Hooker", hook, track="POS_Z", up="Y")

# Follow Path.
o = empty("FollowRing", loc=(0.1, 0.0, 0.3))
c = o.constraints.new("FOLLOW_PATH")
c.target, c.use_curve_follow, c.forward_axis, c.up_axis, c.offset = ring, True, "FORWARD_Y", "UP_Z", 10
o = empty("FixedSnake")
o.scale = (0.5, 0.5, 0.5)
c = o.constraints.new("FOLLOW_PATH")
c.target, c.use_fixed_location, c.offset_factor = snake, True, 0.3
c.use_curve_radius, c.use_curve_follow, c.forward_axis, c.up_axis = True, True, "TRACK_NEGATIVE_Z", "UP_Y"
o = empty("FollowZig", loc=(0, 0, 0.5))
c = o.constraints.new("FOLLOW_PATH")
c.target, c.offset = zig, -4

scene.frame_set(12)
bpy.ops.wm.save_as_mainfile(filepath=out)


def mat(m):
    return " ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4))


with open(truth, "w") as f:
    for frame in (1, 9, 12, 20, 33, 47):
        scene.frame_set(frame)
        graph = bpy.context.evaluated_depsgraph_get()
        for o in scene.objects:
            f.write(f"{frame}\tOB\t{o.name}\t{mat(o.evaluated_get(graph).matrix_world)}\n")
