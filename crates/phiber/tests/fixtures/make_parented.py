import bpy, sys
bpy.ops.wm.read_factory_settings(use_empty=False)
cube = bpy.data.objects["Cube"]
mesh = bpy.data.meshes.new("ArmMesh")
mesh.from_pydata([(0,0,0),(1,0,0),(0,1,0)], [], [(0,1,2)])
arm = bpy.data.objects.new("Arm", mesh)
bpy.data.collections["Collection"].objects.link(arm)
arm.parent = cube
hand = bpy.data.objects.new("Hand", None)
bpy.data.collections["Collection"].objects.link(hand)
hand.parent = arm
bpy.data.objects["Light"].parent = cube
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-1], compress=True)
