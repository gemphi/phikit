"""Simple Deform in every mode, for `tests/simple_deform_truth.rs`.

Each object is a 2 m cube with every edge cut four times (152 vertices, so
the limits and the lock axes cut through interior vertices), at its own
location, rotation and scale, under one Simple Deform:
- "Twist Z": Twist 60 degrees on Z.
- "Twist X Limits": Twist 90 degrees on X, limits 0.2..0.8.
- "Bend Z": Bend 90 degrees on Z (the limit runs along X).
- "Bend X Limits": Bend 120 degrees on X (the limit runs along Z), limits
  0.1..0.9.
- "Bend Y": Bend -75 degrees on Y.
- "Taper Y Lock X": Taper 0.5 on Y with X locked.
- "Taper X Neg": Taper -0.6 on X.
- "Stretch Z Lock Y": Stretch 0.4 on Z with Y locked.
- "Stretch X": Stretch -0.3 on X.
- "Twist Origin": Twist 45 degrees on Z about "Pivot", an empty moved and
  rotated away from the cube.
- "Twist Group Invert": Twist 80 degrees on Z over a vertex group that
  ramps 0..1 along X, inverted.
- "Bend Group": Bend 60 degrees on X over the same ramp, not inverted.

Blender's evaluated vertices are dumped by `../truth/blender_eval_dump.py`
into `simple_deform_truth.txt` once the file is saved (a second Blender
run, so the truth is what an opened file shows).

Run: blender -b --python make_simple_deform.py -- simple_deform.blend"""
import bpy, bmesh, sys, os, math

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene


def cube(name, loc, rot, scale, group=False):
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=2.0)
    bmesh.ops.subdivide_edges(bm, edges=bm.edges[:], cuts=4, use_grid_fill=True)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    sc.collection.objects.link(ob)
    ob.location, ob.rotation_euler, ob.scale = loc, rot, scale
    if group:
        vg = ob.vertex_groups.new(name="Ramp")
        for v in me.vertices:
            vg.add([v.index], (v.co.x + 1.0) / 2.0, "REPLACE")
    return ob


def deform(ob, method, angle=None, factor=None, axis="Z", limits=(0.0, 1.0), lock=(), origin=None, group=None, invert=False):
    md = ob.modifiers.new("Deform", "SIMPLE_DEFORM")
    md.deform_method = method
    if angle is not None:
        md.angle = math.radians(angle)
    if factor is not None:
        md.factor = factor
    md.deform_axis = axis
    md.limits = limits
    md.lock_x = "X" in lock
    md.lock_y = "Y" in lock
    md.lock_z = "Z" in lock
    if origin is not None:
        md.origin = origin
    if group is not None:
        md.vertex_group = group
        md.invert_vertex_group = invert
    return md


pivot = bpy.data.objects.new("Pivot", None)
sc.collection.objects.link(pivot)
pivot.location = (0.4, -0.3, 0.2)
pivot.rotation_euler = (0.3, -0.2, 0.5)

deform(cube("Twist Z", (0, 0, 0), (0.1, 0.2, 0.3), (1, 1, 1)), "TWIST", angle=60, axis="Z")
deform(cube("Twist X Limits", (3, 0, 0), (0, 0, 0.4), (1, 1.2, 0.8)), "TWIST", angle=90, axis="X", limits=(0.2, 0.8))
deform(cube("Bend Z", (6, 0, 0), (0.2, 0, 0), (1, 1, 1)), "BEND", angle=90, axis="Z")
deform(cube("Bend X Limits", (9, 0, 0), (0, 0.3, 0), (1.5, 1, 1)), "BEND", angle=120, axis="X", limits=(0.1, 0.9))
deform(cube("Bend Y", (12, 0, 0), (0, 0, 0), (1, 1, 1)), "BEND", angle=-75, axis="Y")
deform(cube("Taper Y Lock X", (0, 4, 0), (0.1, 0, 0), (1, 1, 1)), "TAPER", factor=0.5, axis="Y", lock=("X",))
deform(cube("Taper X Neg", (3, 4, 0), (0, 0, 0), (1, 1, 1)), "TAPER", factor=-0.6, axis="X")
deform(cube("Stretch Z Lock Y", (6, 4, 0), (0, 0.2, 0), (1, 1, 1)), "STRETCH", factor=0.4, axis="Z", lock=("Y",))
deform(cube("Stretch X", (9, 4, 0), (0, 0, 0), (0.8, 1, 1)), "STRETCH", factor=-0.3, axis="X")
deform(cube("Twist Origin", (0, 8, 0), (0, 0, 0.3), (1, 1, 1)), "TWIST", angle=45, axis="Z", origin=pivot)
deform(cube("Twist Group Invert", (3, 8, 0), (0, 0, 0), (1, 1, 1), group=True), "TWIST", angle=80, axis="Z", group="Ramp", invert=True)
deform(cube("Bend Group", (6, 8, 0), (0, 0, 0), (1, 1, 1), group=True), "BEND", angle=60, axis="X", group="Ramp")

path = sys.argv[sys.argv.index("--") + 1]
bpy.ops.wm.save_as_mainfile(filepath=path)
