# Two empties instancing one collection that is not in the scene — for
# `object/instance_tests.rs`, with Blender's own instance matrices
# (`depsgraph.object_instances`) in `truth/collection_instances.txt`
# (Blender 5.2). The collection holds a cube and a cone parented to it,
# and has an instance offset; the empties are turned and scaled.
# Usage: python make_collection_instances.py <out.blend> <truth.txt>
import bpy, sys, math
bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
seat = bpy.data.collections.new("Seat")
seat.instance_offset = (0.5, 0.0, 0.25)
bpy.ops.mesh.primitive_cube_add(size=0.5, location=(1.0, 0.0, 0.0))
cube = bpy.context.active_object
cube.name = "Pan"
bpy.ops.mesh.primitive_cone_add(radius1=0.1, depth=0.4, location=(1.0, 0.2, 0.6))
cone = bpy.context.active_object
cone.name = "Back"
cone.parent = cube
cone.matrix_parent_inverse = cube.matrix_world.inverted()
cube.rotation_euler = (0.0, 0.0, 0.4)
for ob in (cube, cone):
    sc.collection.objects.unlink(ob)
    seat.objects.link(ob)
for name, loc, rot, scale in (("Row.A", (3.0, 0.0, 1.0), (0.0, 0.0, math.pi / 2), 1.0), ("Row.B", (-2.0, 4.0, 0.5), (0.3, 0.0, -0.7), 1.5)):
    e = bpy.data.objects.new(name, None)
    e.instance_type = "COLLECTION"
    e.instance_collection = seat
    e.location, e.rotation_euler, e.scale = loc, rot, (scale,) * 3
    sc.collection.objects.link(e)
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-2], compress=True)
dg = bpy.context.evaluated_depsgraph_get()
with open(sys.argv[-1], "w") as out:
    for inst in dg.object_instances:
        if inst.is_instance:
            m = inst.matrix_world
            cols = [m[r][c] for c in range(4) for r in range(4)]
            out.write(f"{inst.parent.name}/{inst.object.name}\t{' '.join(f'{v:.6f}' for v in cols)}\n")
