# Makes `fixtures/clamp_to.blend` and `truth/clamp_to.txt` (Blender 5.0):
# objects under the Clamp To constraint (`clampto_evaluate`) over curves of
# each kind — an open Bézier rail along X (the automatic axis), a NURBS
# tower up Z, a poly line whose radius widens its bounds, and a closed
# Bézier loop clamped cyclically — each curve moved, turned and scaled, and
# the owners keyed through, before and past the curves' bounds. One owner
# has a parent; one names its axis; one follows another constraint first.
# Every object's world matrix at six frames; the file is saved at frame 13.
# For `blend/curve/tests.rs`. Run with the bpy module:
#
#   python3 blender_clamp_to.py ../fixtures/clamp_to.blend clamp_to.txt
import bpy, sys

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_homefile(app_template="")
for ob in list(bpy.data.objects):
    bpy.data.objects.remove(ob)
scene = bpy.context.scene
scene.frame_start, scene.frame_end = 1, 40


def curve(name, loc=(0, 0, 0), rot=(0, 0, 0), scale=(1, 1, 1), path=True):
    cu = bpy.data.curves.new(name, "CURVE")
    cu.dimensions = "3D"
    cu.use_path = path
    ob = bpy.data.objects.new(name, cu)
    scene.collection.objects.link(ob)
    ob.location, ob.rotation_euler, ob.scale = loc, rot, scale
    return ob, cu


def points(cu, kind, pts, cyclic=False, radii=None):
    sp = cu.splines.new(kind)
    sp.points.add(len(pts) - 1)
    for i, (p, co) in enumerate(zip(sp.points, pts)):
        p.co = co
        if radii:
            p.radius = radii[i]
    sp.use_cyclic_u = cyclic
    return sp


def owner(name, keys, parent=None, loc=None):
    o = bpy.data.objects.new(name, None)
    scene.collection.objects.link(o)
    if parent:
        o.parent = parent
    for f, co in keys:
        o.location = co
        o.keyframe_insert("location", frame=f)
    if loc:
        o.location = loc
    return o


def clamp(o, target, axis="CLAMPTO_AUTO", cyclic=False):
    c = o.constraints.new("CLAMP_TO")
    c.target, c.main_axis, c.use_cyclic = target, axis, cyclic
    return c


# An open Bézier rail, longest along X: the automatic axis is X.
rail, cu = curve("Rail", loc=(1.0, 0.5, 0.5), rot=(0.1, 0.0, 0.3), scale=(1.5, 1.5, 1.5))
sp = cu.splines.new("BEZIER")
sp.bezier_points.add(2)
for bp, (co, hl, hr, r) in zip(sp.bezier_points, [
    ((-2, 0, 0), (-2.5, -0.3, 0), (-1.5, 0.3, 0.1), 1.0),
    ((0, 1, 0.5), (-0.7, 1, 0.4), (0.7, 1, 0.6), 1.2),
    ((2.5, -0.5, 0), (2, -0.2, 0.1), (3, -0.8, -0.1), 0.8),
]):
    bp.handle_left_type = bp.handle_right_type = "FREE"
    bp.co, bp.handle_left, bp.handle_right, bp.radius = co, hl, hr, r
sp.resolution_u = 12
owner("RailAuto", [(1, (-6, 0, 0)), (40, (8, 1, 2))])
clamp(bpy.data.objects["RailAuto"], rail)
owner("RailY", [(1, (0, -5, 1)), (40, (1, 6, -1))])
clamp(bpy.data.objects["RailY"], rail, "CLAMPTO_Y")

# A NURBS tower up Z: the automatic axis is Z. Its owner hangs from a
# parent that turns, so the owner's world place is what is clamped.
tower, cu = curve("Tower", loc=(-2, 2, 0), rot=(0.0, 0.2, 0.0))
sp = points(cu, "NURBS", [(0, 0, 0, 1), (0.5, 0.3, 1, 1), (-0.2, 0.6, 2, 1), (0.4, 0.1, 3, 1), (0, 0, 4.5, 1)])
sp.order_u, sp.use_endpoint_u, sp.resolution_u = 3, True, 10
pivot = bpy.data.objects.new("Pivot", None)
scene.collection.objects.link(pivot)
pivot.location = (0.5, 0, -1)
for f, z in [(1, 0.0), (40, 1.2)]:
    pivot.rotation_euler = (0, 0, z)
    pivot.keyframe_insert("rotation_euler", frame=f)
owner("Climber", [(1, (0, 0, -3)), (40, (1, 1, 9))], parent=pivot)
clamp(bpy.data.objects["Climber"], tower)

# A poly line with radii: the bounds reach out by each point's radius.
poly, cu = curve("Poly", loc=(0, -3, 0), scale=(1, 2, 1))
points(cu, "POLY", [(0, 0, 0, 1), (1, 0.5, 0.2, 1), (2, -0.5, 0.4, 1), (3, 0, 0, 1)], radii=[0.5, 2.0, 1.0, 1.5])
owner("PolyX", [(1, (-3, -3, 0)), (40, (6, -2, 1))])
clamp(bpy.data.objects["PolyX"], poly, "CLAMPTO_X")

# A closed loop, clamped cyclically: past either end it comes round again.
loop, cu = curve("Loop", loc=(3, 3, 1), rot=(0, 0, 0.5))
sp = cu.splines.new("BEZIER")
k = 0.5523 * 2
sp.bezier_points.add(3)
for bp, (co, hl, hr) in zip(sp.bezier_points, [
    ((-2, 0, 0), (-2, -k, 0), (-2, k, 0)),
    ((0, 2, 0.3), (-k, 2, 0.3), (k, 2, 0.3)),
    ((2, 0, 0), (2, k, 0), (2, -k, 0)),
    ((0, -2, -0.3), (k, -2, -0.3), (-k, -2, -0.3)),
]):
    bp.handle_left_type = bp.handle_right_type = "FREE"
    bp.co, bp.handle_left, bp.handle_right = co, hl, hr
sp.use_cyclic_u, sp.resolution_u = True, 12
owner("LoopCyclic", [(1, (-9, 0, 0)), (40, (13, 1, 0))])
clamp(bpy.data.objects["LoopCyclic"], loop, "CLAMPTO_X", cyclic=True)
owner("LoopFlat", [(1, (-9, 0, 0)), (40, (13, 1, 0))])
clamp(bpy.data.objects["LoopFlat"], loop, "CLAMPTO_X")

# Clamp To after Copy Location: it reads the owner where the stack has it.
o = owner("AfterCopy", [(1, (0, 0, 0))])
c = o.constraints.new("COPY_LOCATION")
c.target = bpy.data.objects["RailAuto"]
clamp(o, rail, "CLAMPTO_Z")

scene.frame_set(13)
bpy.ops.wm.save_as_mainfile(filepath=out)


def mat(m):
    return " ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4))


with open(truth, "w") as f:
    for frame in (1, 7, 13, 20, 27, 35):
        scene.frame_set(frame)
        graph = bpy.context.evaluated_depsgraph_get()
        for o in scene.objects:
            f.write(f"{frame}\tOB\t{o.name}\t{mat(o.evaluated_get(graph).matrix_world)}\n")
