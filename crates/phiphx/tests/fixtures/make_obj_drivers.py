"""OBJDRIVERS.blend: object-level drivers on object transform channels, for
`phiphx/tests/object_driver_truth.rs`.

  Driver      — a cube keyed moving in X from 0 to 10 over frames 1..21.
  Wheel       — its rotation_euler[2] driven by Driver's location[0], a
                Single Property variable, Average of one, through a
                Generator (so the curve maps the driver's value).
  Scaled      — its scale[0] driven by a Python expression over the same
                variable, `1 + x / 10`.
  Summed      — its location[2] driven by the Sum of two variables: the
                Driver's X and Y locations.
  Still       — nothing, so the list has an object with no drivers in it.
"""
import math

import bpy

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
sc.frame_start, sc.frame_end = 1, 21


def cube(name, x):
    bpy.ops.mesh.primitive_cube_add(size=1, location=(x, 0, 0))
    ob = bpy.context.object
    ob.name = name
    return ob


driver_ob = cube("Driver", 0.0)
wheel = cube("Wheel", 3.0)
scaled = cube("Scaled", 6.0)
summed = cube("Summed", 9.0)
cube("Still", 12.0)

# Driver: keyed X from 0 at frame 1 to 10 at frame 21, linear.
driver_ob.location = (0.0, 2.0, 0.0)
driver_ob.keyframe_insert("location", index=0, frame=1)
driver_ob.location = (10.0, 2.0, 0.0)
driver_ob.keyframe_insert("location", index=0, frame=21)
# Blender 5.0 keeps an action's curves under its slotted layers.
action = driver_ob.animation_data.action
curves = getattr(action, "fcurves", None)
if curves is None:
    curves = [fc for layer in action.layers for strip in layer.strips for cb in strip.channelbags for fc in cb.fcurves]
for fc in curves:
    for kp in fc.keyframe_points:
        kp.interpolation = "LINEAR"


def single(fcu, name, target, path):
    v = fcu.driver.variables.new()
    v.name = name
    v.type = "SINGLE_PROP"
    v.targets[0].id = target
    v.targets[0].data_path = path
    return v


# Wheel: rotation_euler[2] = Average(x) through a Generator of 2*x.
fcu = wheel.driver_add("rotation_euler", 2)
fcu.driver.type = "AVERAGE"
single(fcu, "x", driver_ob, "location[0]")
gen = fcu.modifiers.new("GENERATOR")
gen.mode = "POLYNOMIAL"
gen.poly_order = 1
gen.coefficients[0] = 0.5
gen.coefficients[1] = 2.0

# Scaled: scale[0] = 1 + x / 10, a Python expression.
fcu = scaled.driver_add("scale", 0)
fcu.driver.type = "SCRIPTED"
fcu.driver.expression = "1 + x / 10"
single(fcu, "x", driver_ob, "location[0]")

# Summed: location[2] = Sum(x, y) of the Driver's X and Y.
fcu = summed.driver_add("location", 2)
fcu.driver.type = "SUM"
single(fcu, "x", driver_ob, "location[0]")
single(fcu, "y", driver_ob, "location[1]")

bpy.ops.wm.save_as_mainfile(filepath=__file__.replace("make_obj_drivers.py", "OBJDRIVERS.blend"), compress=True)
