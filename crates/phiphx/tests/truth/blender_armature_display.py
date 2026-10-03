# Makes `fixtures/armature_display.blend` and `truth/armature_display.txt`,
# and `truth/thymus_bones.txt` from THYMUS (Blender 5.0): which bones an
# armature shows in Object mode and where each one's shape is drawn — the
# display matrix `overlay_armature.cc` builds (`draw_bone_update_disp_matrix
# _default` / `_custom_shape`) — and each custom shape's edges, for
# `blend/rig/display_tests.rs`.
#
#   python3 blender_armature_display.py ../fixtures/armature_display.blend armature_display.txt THYMUS.blend thymus_bones.txt
import bpy, sys
from mathutils import Matrix, Euler

out, truth, thymus, thymus_truth = sys.argv[-4:]


def mat(m):
    return " ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4))


def shown(arm, pb):
    """`animrig::bone_is_visible` for a pose bone: not hidden
    (`PoseBone.hide`), and in a visible collection (or in none, unless a
    collection is soloed)."""
    if pb.hide:
        return False
    b = pb.bone
    solo = any(c.is_solo for c in arm.collections_all)
    if not b.collections:
        return not solo
    return any((c.is_solo if solo else c.is_visible_effectively) for c in b.collections)


def disp(ob, pb):
    """`draw_bone_update_disp_matrix_custom_shape` / `_default`, in world."""
    arm = ob.data
    if pb.custom_shape and not arm.show_bone_custom_shapes is False:
        src = pb.custom_shape_transform or pb
        length = pb.bone.length if pb.use_custom_shape_bone_size else 1.0
        m = src.matrix.copy()
        t = pb.custom_shape_translation
        m = m @ Matrix.Translation(t)
        m = m @ Euler(pb.custom_shape_rotation_euler, "XYZ").to_matrix().to_4x4()
        s = [v * length for v in pb.custom_shape_scale_xyz]
        m = m @ Matrix.Diagonal((s[0], s[1], s[2], 1.0))
        return ob.matrix_world @ m, pb.custom_shape.name
    m = pb.matrix @ Matrix.Diagonal((pb.bone.length, pb.bone.length, pb.bone.length, 1.0))
    return ob.matrix_world @ m, "-"


def shapes(f, rigs):
    """Each custom shape a shown bone draws: its edge count and the sum of
    its edges' coordinates, in its own space."""
    seen = set()
    for name in rigs:
        ob = bpy.data.objects[name]
        for pb in ob.pose.bones:
            w = pb.custom_shape
            if not w or w.name in seen or not shown(ob.data, pb) or w.type != "MESH":
                continue
            seen.add(w.name)
            me = w.data
            total = sum(sum(me.vertices[e.vertices[0]].co) + sum(me.vertices[e.vertices[1]].co) for e in me.edges)
            f.write(f"SHAPE\t{w.name}\t{len(me.edges)}\t{total:.5f}\n")


def write(f, frames, rigs):
    scene = bpy.context.scene
    for frame in frames:
        scene.frame_set(frame)
        graph = bpy.context.evaluated_depsgraph_get()
        for name in rigs:
            ob = bpy.data.objects[name].evaluated_get(graph)
            for pb in ob.pose.bones:
                if not shown(ob.data, pb):
                    f.write(f"{frame}\t{name}\t{pb.name}\t0\t-\t-\n")
                    continue
                m, shape = disp(ob, pb)
                f.write(f"{frame}\t{name}\t{pb.name}\t1\t{shape}\t{mat(m)}\n")


# A small armature of every case: nested and soloed collections, a hidden
# bone, a bone in no collection, custom shapes offset, turned and scaled,
# one drawn at another bone, one not scaled by its length, a wire one.
bpy.ops.wm.read_homefile(app_template="")
for o in list(bpy.data.objects):
    bpy.data.objects.remove(o)
scene = bpy.context.scene
scene.frame_start, scene.frame_end = 1, 30
arm = bpy.data.armatures.new("Arm")
ob = bpy.data.objects.new("Rig", arm)
scene.collection.objects.link(ob)
ob.location, ob.rotation_euler, ob.scale = (0.5, -1, 0.2), (0.1, 0, 0.4), (1.2, 1.2, 1.2)
bpy.context.view_layer.objects.active = ob
bpy.ops.object.mode_set(mode="EDIT")
eb = arm.edit_bones
spec = [("root", None, (0, 0, 0), (0, 1, 0)), ("spine", "root", (0, 1, 0), (0, 1.5, 1)),
        ("head", "spine", (0, 1.5, 1), (0, 1.6, 1.8)), ("arm.L", "spine", (0.2, 1.4, 0.8), (1.2, 1.2, 0.9)),
        ("arm.R", "spine", (-0.2, 1.4, 0.8), (-1.2, 1.2, 0.9)), ("loose", None, (1, 0, 0), (1, 0, 0.7)),
        ("ctrl", None, (0, -1, 0), (0, -1, 0.5)), ("hidden", "root", (0.5, 0.5, 0), (0.5, 0.5, 0.6))]
for name, parent, head, tail in spec:
    b = eb.new(name)
    b.head, b.tail = head, tail
    if parent:
        b.parent = eb[parent]
bpy.ops.object.mode_set(mode="OBJECT")
body = arm.collections.new("Body")
limbs = arm.collections.new("Limbs", parent=body)
face = arm.collections.new("Face")
extra = arm.collections.new("Extra")
for n in ("root", "spine", "hidden"):
    body.assign(arm.bones[n])
for n in ("arm.L", "arm.R"):
    limbs.assign(arm.bones[n])
face.assign(arm.bones["head"])
extra.assign(arm.bones["ctrl"])
extra.assign(arm.bones["arm.R"])
face.is_visible = False
ob.pose.bones["hidden"].hide = True
# The bone's own hide is Edit mode's: the pose bone still shows.
arm.bones["loose"].hide = True
arm.display_type = "OCTAHEDRAL"
# Shapes: a circle, and a mesh with faces drawn as wire.
bpy.ops.mesh.primitive_circle_add(vertices=16, radius=1.0)
circle = bpy.context.active_object
circle.name = "WGT-circle"
bpy.ops.mesh.primitive_cube_add(size=0.5)
cube = bpy.context.active_object
cube.name = "WGT-cube"
for w in (circle, cube):
    w.location = (5, 5, 5)
pose = ob.pose.bones
pose["spine"].custom_shape = circle
pose["spine"].custom_shape_translation = (0.1, 0.2, 0.0)
pose["spine"].custom_shape_rotation_euler = (0.3, 0.0, 0.5)
pose["spine"].custom_shape_scale_xyz = (1.5, 1.0, 0.5)
pose["arm.L"].custom_shape = cube
pose["arm.L"].use_custom_shape_bone_size = False
pose["arm.R"].custom_shape = circle
pose["arm.R"].custom_shape_transform = pose["arm.L"]
pose["ctrl"].custom_shape = circle
arm.bones["ctrl"].show_wire = True
# Keys, so the pose moves.
for f, rot in [(1, 0.0), (30, 1.2)]:
    for n in ("spine", "arm.L", "arm.R"):
        pose[n].rotation_mode = "XYZ"
        pose[n].rotation_euler = (rot, 0.3 * rot, 0)
        pose[n].keyframe_insert("rotation_euler", frame=f)
    pose["root"].location = (0, 0, rot)
    pose["root"].keyframe_insert("location", frame=f)
# A second armature, soloed collection, stick display.
arm2 = bpy.data.armatures.new("Arm2")
ob2 = bpy.data.objects.new("Stick", arm2)
scene.collection.objects.link(ob2)
ob2.location = (3, 0, 0)
bpy.context.view_layer.objects.active = ob2
bpy.ops.object.mode_set(mode="EDIT")
for name, head, tail in [("a", (0, 0, 0), (0, 0, 1)), ("b", (0, 0, 1), (0, 0, 2)), ("c", (1, 0, 0), (1, 0, 1))]:
    b = arm2.edit_bones.new(name)
    b.head, b.tail = head, tail
bpy.ops.object.mode_set(mode="OBJECT")
arm2.display_type = "STICK"
s1, s2 = arm2.collections.new("One"), arm2.collections.new("Two")
s1.assign(arm2.bones["a"])
s2.assign(arm2.bones["b"])
s2.is_solo = True
scene.frame_set(9)
bpy.ops.wm.save_as_mainfile(filepath=out)
with open(truth, "w") as f:
    write(f, (1, 9, 20, 30), ["Rig", "Stick"])
    shapes(f, ["Rig", "Stick"])

bpy.ops.wm.open_mainfile(filepath=thymus)
with open(thymus_truth, "w") as f:
    write(f, (1, 30, 60), ["rig"])
    shapes(f, ["rig"])
