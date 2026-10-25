"""A two-bone armature ("root" 0→1 m up z, "tip" 1→2 m) deforming a
column of 0.2 m square, 2 m tall in four rings, weighted root below 1 m and
tip above, held by an action of two rest keys: the smallest skinned figure, for `scene/build.rs`'s tests.
Run: python3 make_skin_rig.py skin_rig.blend"""
import bpy, bmesh, sys

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
arm = bpy.data.armatures.new("Arm")
rig = bpy.data.objects.new("Figure Rig", arm)
sc.collection.objects.link(rig)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.mode_set(mode="EDIT")
root = arm.edit_bones.new("root"); root.head, root.tail = (0, 0, 0), (0, 0, 1)
tip = arm.edit_bones.new("tip"); tip.head, tip.tail = (0, 0, 1), (0, 0, 2); tip.parent = root
bpy.ops.object.mode_set(mode="OBJECT")

mesh = bpy.data.meshes.new("Figure Body")
bm = bmesh.new()
rings = []
for z in (0.0, 0.5, 1.0, 1.5, 2.0):
    rings.append([bm.verts.new((x, y, z)) for x, y in ((-0.1, -0.1), (0.1, -0.1), (0.1, 0.1), (-0.1, 0.1))])
for a, b in zip(rings, rings[1:]):
    for k in range(4):
        bm.faces.new((a[k], a[(k + 1) % 4], b[(k + 1) % 4], b[k]))
bm.to_mesh(mesh)
bm.free()
body = bpy.data.objects.new("Figure Body", mesh)
sc.collection.objects.link(body)
body.parent = rig
groups = {n: body.vertex_groups.new(name=n) for n in ("root", "tip")}
for v in mesh.vertices:
    groups["root" if v.co.z < 1.0 else "tip"].add([v.index], 1.0, "REPLACE")
body.modifiers.new("Armature", "ARMATURE").object = rig
# Import keeps a rig's meshes as skins only when the rig moves, so it holds
# an action — two keys of the rest pose, still in fact.
for f in (1, 2):
    rig.pose.bones["root"].keyframe_insert("rotation_quaternion", frame=f)
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-1])
