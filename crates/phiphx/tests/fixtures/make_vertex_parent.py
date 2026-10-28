"""Vertex parenting, for `tests/vertex_parent_truth.rs`: children hung
from one or three vertices of parents whose evaluated meshes differ from
their saved ones.

Parents:
- "Sheet": a 4 x 4 grid (25 vertices) at (1, 2, 0.5), rotated, scaled,
  under a Subdivision level 1 — the evaluated corner vertices move off the
  saved ones.
- "Skin": a plane (4 vertices) skinned to "Rig", a one-bone armature whose
  bone is rotated 40 degrees in its saved pose — the evaluated mesh is the
  posed one.

Children, each with its own location and rotation:
- "On One": an empty on Sheet's vertex 7 (VERTEX).
- "On Three": an empty on Sheet's vertices 3, 12, 20 (VERTEX_3) with a
  non-trivial parent inverse.
- "Block": a cube on Sheet's vertices 0, 4, 24 (VERTEX_3), a mesh object.
- "Grand": an empty parented to "On Three" as an object (PAROBJECT).
- "Lamp": a point light on Sheet's vertex 24 (VERTEX).
- "Cam": a camera on Sheet's vertices 10, 14, 22 (VERTEX_3).
- "Posed": an empty on Skin's vertices 1, 2, 3 (VERTEX_3).
- "Posed One": an empty on Skin's vertex 2 (VERTEX).
- "Past": an empty on Sheet's vertex 999 (past the mesh: Blender's
  `give_parvert` leaves the parent's origin).

Blender's world matrices come from a second run of
`../truth/blender_eval_dump.py` on the saved file.

Run: blender -b --python make_vertex_parent.py -- vertex_parent.blend"""
import bpy, bmesh, sys, math
from mathutils import Matrix

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene


def link(name, data):
    ob = bpy.data.objects.new(name, data)
    sc.collection.objects.link(ob)
    return ob


me = bpy.data.meshes.new("Sheet")
bm = bmesh.new()
bmesh.ops.create_grid(bm, x_segments=4, y_segments=4, size=1.5)
bm.to_mesh(me)
bm.free()
sheet = link("Sheet", me)
sheet.location = (1.0, 2.0, 0.5)
sheet.rotation_euler = (0.3, -0.2, 0.6)
sheet.scale = (1.2, 0.9, 1.0)
sub = sheet.modifiers.new("Subdivision", "SUBSURF")
sub.levels = 1

arm = bpy.data.armatures.new("Rig")
rig = link("Rig", arm)
rig.location = (-2.0, 0.0, 0.0)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.mode_set(mode="EDIT")
b = arm.edit_bones.new("Bone")
b.head, b.tail = (0.0, 0.0, 0.0), (0.0, 0.0, 1.0)
bpy.ops.object.mode_set(mode="OBJECT")
rig.pose.bones["Bone"].rotation_mode = "XYZ"
rig.pose.bones["Bone"].rotation_euler = (math.radians(40), 0.0, 0.0)

me = bpy.data.meshes.new("Skin")
bm = bmesh.new()
bmesh.ops.create_grid(bm, x_segments=1, y_segments=1, size=1.0)
bm.to_mesh(me)
bm.free()
skin = link("Skin", me)
skin.location = (-2.0, 0.0, 1.0)
vg = skin.vertex_groups.new(name="Bone")
vg.add([v.index for v in me.vertices], 1.0, "REPLACE")
am = skin.modifiers.new("Armature", "ARMATURE")
am.object = rig
skin.parent = rig


def child(name, data, parent, verts, loc, rot, parentinv=None):
    ob = link(name, data)
    ob.location, ob.rotation_euler = loc, rot
    ob.parent = parent
    if len(verts) == 1:
        ob.parent_type = "VERTEX"
        ob.parent_vertices = (verts[0], 0, 0)
    else:
        ob.parent_type = "VERTEX_3"
        ob.parent_vertices = verts
    ob.matrix_parent_inverse = parentinv if parentinv is not None else Matrix.Identity(4)
    return ob


child("On One", None, sheet, (7,), (0.2, 0.1, 0.3), (0.1, 0.2, 0.3))
three = child("On Three", None, sheet, (3, 12, 20), (0.1, -0.2, 0.4), (0.0, 0.5, 0.0), parentinv=Matrix.Rotation(0.4, 4, "Z") @ Matrix.Translation((0.3, 0.0, -0.2)))
cube = bpy.data.meshes.new("Block")
bm = bmesh.new()
bmesh.ops.create_cube(bm, size=0.4)
bm.to_mesh(cube)
bm.free()
child("Block", cube, sheet, (0, 4, 24), (0.0, 0.0, 0.5), (0.2, 0.0, 0.0))
grand = link("Grand", None)
grand.location, grand.rotation_euler = (0.5, 0.0, 0.0), (0.0, 0.0, 0.7)
grand.parent = three
child("Lamp", bpy.data.lights.new("Lamp", "POINT"), sheet, (24,), (0.0, 0.0, 1.0), (0.0, 0.0, 0.0))
child("Cam", bpy.data.cameras.new("Cam"), sheet, (10, 14, 22), (0.0, 0.0, 2.0), (1.2, 0.0, 0.3))
child("Posed", None, skin, (1, 2, 3), (0.1, 0.1, 0.1), (0.0, 0.0, 0.2))
child("Posed One", None, skin, (2,), (0.0, 0.0, 0.3), (0.0, 0.0, 0.0))
child("Past", None, sheet, (999,), (0.1, 0.0, 0.0), (0.0, 0.0, 0.0))

bpy.context.view_layer.update()
path = sys.argv[sys.argv.index("--") + 1]
bpy.ops.wm.save_as_mainfile(filepath=path)
