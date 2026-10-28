# Which collection instances Blender 5.2 draws — for `tests/instance_truth.rs`,
# with every instance `depsgraph.object_instances` lists in
# `truth/instance_visibility.txt` (instancer, object, world matrix).
#
# The collection "Kit" (not in the scene) holds a cube, a cube disabled in
# viewports, a child collection disabled in viewports, and an empty that
# instances "Leaf", whose empty instances "Twig": an instance two levels
# deep. Eight empties instance "Kit": one plain, one disabled in viewports,
# one hidden (the eye), one in an excluded collection, one in a collection
# disabled in viewports, one in a collection hidden by the eye, one in a
# second scene, and one turned and scaled. (An empty instancing the
# collection it is in cannot be made here: RNA refuses to point
# `instance_collection` at a collection holding the object.)
# The collection "Twin" is in the scene as well, and two empties at the
# origin instance it with no offset: its "Tile" is drawn three times at one
# place, itself and one copy per instancer (`make_duplis_collection` runs
# once per instancer; nothing merges coincident copies).
# Usage: blender -b --python make_instance_visibility.py -- <out.blend> <truth.txt>
import bpy, sys, math
out_blend, out_truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene

def mesh(name, coll, loc, size=0.4):
    bpy.ops.mesh.primitive_cube_add(size=size, location=loc)
    ob = bpy.context.active_object
    ob.name = name
    for c in ob.users_collection:
        c.objects.unlink(ob)
    coll.objects.link(ob)
    return ob

def empty(name, coll, target, loc=(0, 0, 0), rot=(0, 0, 0), scale=1.0):
    e = bpy.data.objects.new(name, None)
    e.location, e.rotation_euler, e.scale = loc, rot, (scale,) * 3
    e.instance_type = "COLLECTION"
    e.instance_collection = target
    coll.objects.link(e)
    return e

kit, leaf, twig = (bpy.data.collections.new(n) for n in ("Kit", "Leaf", "Twig"))
kit.instance_offset = (0.25, 0.0, 0.0)
leaf.instance_offset = (0.0, 0.5, 0.0)
mesh("Bolt", kit, (1.0, 0.0, 0.0))
mesh("Ghost", kit, (0.0, 1.0, 0.0)).hide_viewport = True
shut_kid = bpy.data.collections.new("KitShut")
shut_kid.hide_viewport = True
kit.children.link(shut_kid)
mesh("Nut", shut_kid, (0.0, 0.0, 1.0))
empty("Inner", kit, leaf, loc=(0.0, 2.0, 0.0), rot=(0.0, 0.0, 0.5))
mesh("Petal", leaf, (0.5, 0.0, 0.0), 0.2)
empty("Stem", leaf, twig, loc=(0.0, 0.0, 1.0), rot=(0.3, 0.0, 0.0), scale=0.5)
mesh("Bud", twig, (0.0, 0.3, 0.0), 0.1)

empty("Shown", sc.collection, kit, loc=(3.0, 0.0, 0.0))
empty("Turned", sc.collection, kit, loc=(-2.0, 4.0, 0.5), rot=(0.3, 0.0, -0.7), scale=1.5)
empty("Disabled", sc.collection, kit, loc=(6.0, 0.0, 0.0)).hide_viewport = True
eye = empty("Eyed", sc.collection, kit, loc=(9.0, 0.0, 0.0))
for name in ("Off", "Shut", "Dim"):
    c = bpy.data.collections.new(name)
    sc.collection.children.link(c)
empty("Excluded", bpy.data.collections["Off"], kit, loc=(12.0, 0.0, 0.0))
empty("Restricted", bpy.data.collections["Shut"], kit, loc=(15.0, 0.0, 0.0))
empty("Dimmed", bpy.data.collections["Dim"], kit, loc=(18.0, 0.0, 0.0))
bpy.data.collections["Shut"].hide_viewport = True
twin = bpy.data.collections.new("Twin")
sc.collection.children.link(twin)
mesh("Tile", twin, (0.0, -3.0, 0.0))
empty("Atop", sc.collection, twin)
empty("Atop.001", sc.collection, twin)
other = bpy.data.scenes.new("Other")
empty("Elsewhere", other.collection, kit, loc=(21.0, 0.0, 0.0))

vl = bpy.context.view_layer
eye.hide_set(True)
vl.layer_collection.children["Off"].exclude = True
vl.layer_collection.children["Dim"].hide_viewport = True
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(filepath=out_blend, compress=True)
dg = bpy.context.evaluated_depsgraph_get()
with open(out_truth, "w") as out:
    for inst in dg.object_instances:
        if inst.is_instance:
            m = inst.matrix_world
            cols = [m[r][c] for c in range(4) for r in range(4)]
            out.write(f"{inst.parent.name}\t{inst.object.name}\t{' '.join(f'{v:.6f}' for v in cols)}\n")
