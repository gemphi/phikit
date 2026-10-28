"""Half a figure, mirrored by its modifier stack, posed by a rig: the shape of
every garment and accessory in the CW-POSES files (`Chatty Pants`: Mirror,
Armature, Subdivision; `Earring pearl`: Mirror, Armature), small enough to
check by hand.

A two-bone armature ("root" 0→1 m up z, "tip" 1→2 m), held by an action of
two rest keys so the import keeps its meshes as skins, deforming:
- "Half Body": the x ≥ 0 half of a 0.2 m square column 2 m tall in five
  rings — 12 quads, its seam on x = 0 — under Mirror X (merge on), Armature,
  Subdivision level 1. Blender evaluates it to 96 quads, 192 triangles.
- "Half Pearl": the same half column under Mirror X and Armature alone —
  24 quads, 48 triangles: exactly twice what was saved.
- "Twin Body": a second "Half Body" (its mesh shared) standing at x = 1 with
  scale (−1, 1, 1), a negative determinant as `Female-Mannequin-Body-01.004`
  has; the same 192 triangles.

Run: blender -b --python make_mirror_rig.py -- mirror_rig.blend"""
import bpy, bmesh, sys

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
arm = bpy.data.armatures.new("Arm")
rig = bpy.data.objects.new("Half Rig", arm)
sc.collection.objects.link(rig)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.mode_set(mode="EDIT")
root = arm.edit_bones.new("root"); root.head, root.tail = (0, 0, 0), (0, 0, 1)
tip = arm.edit_bones.new("tip"); tip.head, tip.tail = (0, 0, 1), (0, 0, 2); tip.parent = root
bpy.ops.object.mode_set(mode="OBJECT")


def half_column(name):
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    rings = []
    for z in (0.0, 0.5, 1.0, 1.5, 2.0):
        rings.append([bm.verts.new((x, y, z)) for x, y in ((0.0, -0.1), (0.1, -0.1), (0.1, 0.1), (0.0, 0.1))])
    for a, b in zip(rings, rings[1:]):
        for k in range(3):
            bm.faces.new((a[k], a[k + 1], b[k + 1], b[k]))
    bm.to_mesh(mesh)
    bm.free()
    return mesh


def figure(name, mesh, subdivide):
    ob = bpy.data.objects.new(name, mesh)
    sc.collection.objects.link(ob)
    ob.parent = rig
    groups = {n: ob.vertex_groups.new(name=n) for n in ("root", "tip")}
    for v in mesh.vertices:
        groups["root" if v.co.z < 1.0 else "tip"].add([v.index], 1.0, "REPLACE")
    mirror = ob.modifiers.new("Mirror", "MIRROR")
    mirror.use_axis = (True, False, False)
    ob.modifiers.new("Armature", "ARMATURE").object = rig
    if subdivide:
        ob.modifiers.new("Subdivision", "SUBSURF").levels = 1
    return ob


body_mesh = half_column("Half Body")
figure("Half Body", body_mesh, True)
figure("Half Pearl", half_column("Half Pearl"), False)
twin = figure("Twin Body", body_mesh, True)
twin.location = (1.0, 0.0, 0.0)
twin.scale = (-1.0, 1.0, 1.0)
# Import keeps a rig's meshes as skins only when the rig moves, so it holds
# an action — two keys of the rest pose, still in fact.
for f in (1, 2):
    rig.pose.bones["root"].keyframe_insert("rotation_quaternion", frame=f)
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-1])
