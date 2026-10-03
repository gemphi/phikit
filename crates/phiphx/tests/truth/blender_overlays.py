# Makes `fixtures/overlays.blend` and `truth/overlays.txt` (Blender 5.0):
# what the 3D viewport draws over the scene and where.
#
# - The Layout viewport's overlay and gizmo switches, each set away from
#   Blender's default where it has one (`View3D.overlay.flag`, `flag`,
#   `flag2`, `gridflag`, `gizmo_flag`, `gizmo_show_object`).
# - The 3D cursor, moved and turned in ZXY order (`Scene.cursor`).
# - Relationship lines (`overlay_relation.hh`): each child's line runs to
#   where it hangs from its parent (`parent_display_origin`) — an object, a
#   bone's tail, a bone's deform with relative parenting — and each
#   expanded constraint's line to its target: an object, a bone at
#   `head_tail`, a point on a Follow Path curve. A folded constraint and
#   Clamp To draw none.
#
# The display origin is not in the RNA, so it is taken from what is:
# `matrix_world = parent_matrix @ matrix_parent_inverse @ matrix_basis`
# for an object without constraints. A Follow Path owner standing at the
# origin sits exactly on its target point. Three frames; the file is saved
# at frame 12. For `blend/scene/overlay_tests.rs`. Run with the bpy module:
#
#   python3 blender_overlays.py ../fixtures/overlays.blend overlays.txt
import bpy, sys
from mathutils import Vector

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_homefile(app_template="")
for ob in list(bpy.data.objects):
    bpy.data.objects.remove(ob)
scene = bpy.context.scene
scene.frame_start, scene.frame_end = 1, 24
link = scene.collection.objects.link


def empty(name, loc=(0, 0, 0), parent=None):
    o = bpy.data.objects.new(name, None)
    link(o)
    o.location = loc
    if parent:
        o.parent = parent
    return o


# The cursor, away from the origin and turned.
scene.cursor.location = (1.5, -2.0, 0.5)
scene.cursor.rotation_mode = "ZXY"
scene.cursor.rotation_euler = (0.3, 0.2, 1.0)

# An object parent that moves, and a child hanging from it.
base = empty("Base", (2, 0, 0))
base.rotation_euler = (0, 0, 0.4)
for f, x in [(1, 2.0), (24, 3.0)]:
    base.location.x = x
    base.keyframe_insert("location", frame=f)
child = bpy.data.objects.new("Child", bpy.data.meshes.new("ChildMesh"))
link(child)
child.parent = base
child.location = (1, 1, 0.5)

# A rig whose bone turns; one child on its tail, one relative to its deform.
arm = bpy.data.armatures.new("RigData")
rig = bpy.data.objects.new("Rig", arm)
link(rig)
rig.location = (-2, 1, 0)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.mode_set(mode="EDIT")
root = arm.edit_bones.new("root")
root.head, root.tail = (0, 0, 0), (0, 0, 1)
limb = arm.edit_bones.new("limb")
limb.head, limb.tail, limb.parent = (0, 0, 1), (0, 1, 1.5), root
limb.use_relative_parent = False
rel = arm.edit_bones.new("rel")
rel.head, rel.tail, rel.parent = (0, 0, 1), (1, 0, 1), root
rel.use_relative_parent = True
bpy.ops.object.mode_set(mode="OBJECT")
pb = rig.pose.bones["root"]
pb.rotation_mode = "XYZ"
for f, z in [(1, 0.0), (24, 0.8)]:
    pb.rotation_euler = (0, 0, z)
    pb.keyframe_insert("rotation_euler", frame=f)
on_tail = empty("OnTail", (0.2, 0, 0))
on_tail.parent, on_tail.parent_type, on_tail.parent_bone = rig, "BONE", "limb"
on_rel = empty("OnRel", (0, 0.3, 0))
on_rel.parent, on_rel.parent_type, on_rel.parent_bone = rig, "BONE", "rel"

# Constraints: an object target, a bone at head_tail, a folded one, a path.
track = empty("Track", (0, -3, 1))
c = track.constraints.new("TRACK_TO")
c.target = base
copy = empty("Copy", (0, 0, 0))
c = copy.constraints.new("COPY_LOCATION")
c.target, c.subtarget, c.head_tail = rig, "limb", 0.5
c.use_offset = True
folded = empty("Folded", (1, -1, 0))
c = folded.constraints.new("DAMPED_TRACK")
c.target, c.show_expanded = base, False
cu = bpy.data.curves.new("Rail", "CURVE")
cu.dimensions, cu.use_path = "3D", True
sp = cu.splines.new("POLY")
sp.points.add(2)
for p, co in zip(sp.points, [(0, 0, 0, 1), (2, 1, 0, 1), (4, 0, 1, 1)]):
    p.co = co
rail = bpy.data.objects.new("Rail", cu)
link(rail)
rail.location = (0, 4, 0)
cu.path_duration = 24
cu.eval_time = 0.0
cu.keyframe_insert("eval_time", frame=1)
cu.eval_time = 24.0
cu.keyframe_insert("eval_time", frame=24)
rider = empty("Rider")
c = rider.constraints.new("FOLLOW_PATH")
c.target = rail
clamped = empty("Clamped", (1, 4, 0))
c = clamped.constraints.new("CLAMP_TO")
c.target = rail

# A hidden parent: Blender draws no line to it.
hidden = empty("HiddenParent", (0, 0, 3))
orphan = empty("Orphan", (1, 0, 0), parent=hidden)
hidden.hide_set(True)

# The Layout viewport's switches, each away from the default where it can be.
view = next(a for a in bpy.data.screens["Layout"].areas if a.type == "VIEW_3D").spaces[0]
ov = view.overlay
ov.show_floor, ov.show_axis_z, ov.show_text, ov.show_stats = False, True, False, True
ov.show_bones, ov.show_outline_selected, ov.show_object_origins_all = False, False, True
view.show_gizmo_navigate, view.show_gizmo_object_rotate = False, True

scene.frame_set(12)
bpy.ops.object.select_all(action="DESELECT")
child.select_set(True)
bpy.context.view_layer.objects.active = child
bpy.ops.wm.save_as_mainfile(filepath=out)


def v3(v):
    return " ".join(f"{x:.6f}" for x in v)


with open(truth, "w") as f:
    for name in ["show_overlays", "show_floor", "show_axis_x", "show_axis_y", "show_axis_z", "show_text",
                 "show_stats", "show_cursor", "show_extras", "show_bones", "show_relationship_lines",
                 "show_outline_selected", "show_object_origins", "show_object_origins_all"]:
        f.write(f"OVL\t{name}\t{int(getattr(ov, name))}\n")
    for name in ["show_gizmo", "show_gizmo_navigate", "show_gizmo_context", "show_gizmo_tool",
                 "show_gizmo_object_translate", "show_gizmo_object_rotate", "show_gizmo_object_scale"]:
        f.write(f"OVL\t{name}\t{int(getattr(view, name))}\n")
    m = scene.cursor.matrix
    f.write("CURSOR\t" + " ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4)) + "\n")
    f.write("HIDDEN\tHiddenParent\n")
    for frame in (1, 12, 24):
        scene.frame_set(frame)
        graph = bpy.context.evaluated_depsgraph_get()
        for o in scene.objects:
            e = o.evaluated_get(graph)
            if o.parent and not o.constraints:
                pm = e.matrix_world @ e.matrix_basis.inverted() @ e.matrix_parent_inverse.inverted()
                f.write(f"{frame}\tPARENT\t{o.name}\t{o.parent.name}\t{v3(e.matrix_world.translation)}\t{v3(pm.translation)}\n")
            for c in o.constraints:
                if not c.show_expanded or c.type == "CLAMP_TO":
                    continue
                if c.type == "FOLLOW_PATH":
                    at = e.matrix_world.translation
                elif getattr(c, "subtarget", ""):
                    t = c.target.evaluated_get(graph)
                    bone = t.pose.bones[c.subtarget]
                    at = t.matrix_world @ bone.head.lerp(bone.tail, c.head_tail)
                else:
                    at = c.target.evaluated_get(graph).matrix_world.translation
                f.write(f"{frame}\tTARGET\t{o.name}\t{c.name}\t{v3(at)}\t{v3(e.matrix_world.translation)}\n")
