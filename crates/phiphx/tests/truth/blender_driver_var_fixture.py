# Builds `fixtures/DRIVERVARS.blend`: one object per driver variable kind, so
# every `dvar_eval_*` path has a case the reader can be held to.
#
#   python -I blender_driver_var_fixture.py <out.blend>
import bpy, sys, math

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
out = args[0] if args else "DRIVERVARS.blend"

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
sc.frame_start, sc.frame_end = 1, 21


def cube(name, loc=(0, 0, 0)):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    ob = bpy.context.object
    ob.name = name
    return ob


# SOURCE: the object every variable reads. Keyed location, rotation and scale.
src = cube("SOURCE", (0, 0, 0))
src.rotation_mode = "XYZ"
for frame, (lx, rz, sx) in {1: (0.0, 0.0, 1.0), 11: (3.0, math.radians(70.0), 2.5), 21: (-1.0, math.radians(-40.0), 0.5)}.items():
    src.location = (lx, lx * 0.5, 0.25 * lx)
    src.rotation_euler = (0.2, -0.3, rz)
    src.scale = (sx, sx * 0.5 + 0.5, 1.0)
    src.keyframe_insert("location", frame=frame)
    src.keyframe_insert("rotation_euler", frame=frame)
    src.keyframe_insert("scale", frame=frame)

# A parent, so world space and local space differ for SOURCE's child.
parent = cube("PARENT", (2, -1, 0.5))
parent.rotation_euler = (0.0, 0.0, math.radians(30.0))
child = cube("CHILD", (0.75, 0.0, 0.0))
child.parent = parent

# OTHER: the second end of the Loc Diff / Rot Diff variables.
other = cube("OTHER", (-2.0, 1.5, 0.75))
other.rotation_mode = "XYZ"
other.rotation_euler = (0.0, 0.5, -0.9)

# PROPHOST carries a custom property, driven off SOURCE's X location; a
# second driver then reads that property. The depsgraph orders the two.
host = cube("PROPHOST", (0, 3, 0))
host["gain"] = 0.0


def driver(ob, path, index=-1):
    fc = ob.driver_add(path, index)
    while fc.driver.variables:
        fc.driver.variables.remove(fc.driver.variables[0])
    fc.modifiers.remove(fc.modifiers[0]) if fc.modifiers else None
    return fc


def single(fc, name, ob, path):
    v = fc.driver.variables.new()
    v.name = name
    v.type = "SINGLE_PROP"
    v.targets[0].id = ob
    v.targets[0].data_path = path
    return v


def transform(fc, name, ob, chan, space="WORLD_SPACE", rot="AUTO"):
    v = fc.driver.variables.new()
    v.name = name
    v.type = "TRANSFORMS"
    v.targets[0].id = ob
    v.targets[0].transform_type = chan
    v.targets[0].transform_space = space
    v.targets[0].rotation_mode = rot
    return v


def pair(fc, name, kind, a, b, space_a="WORLD_SPACE", space_b="WORLD_SPACE"):
    v = fc.driver.variables.new()
    v.name = name
    v.type = kind
    v.targets[0].id = a
    v.targets[0].transform_space = space_a
    v.targets[1].id = b
    v.targets[1].transform_space = space_b
    return v


# TRANSCHAN reads SOURCE's world location X, world scale X, average scale and
# its Z rotation in Auto mode — one per channel of its own.
tc = cube("TRANSCHAN", (0, -3, 0))
tc.rotation_mode = "XYZ"
for i, chan in enumerate(["LOC_X", "SCALE_X", "SCALE_AVG"]):
    fc = driver(tc, "location", i)
    transform(fc, "v", src, chan)
    fc.driver.expression = "v"
fc = driver(tc, "rotation_euler", 2)
transform(fc, "v", src, "ROT_Z")
fc.driver.expression = "v"
# All three spaces on the parented child: World, Transform and Local are
# three different answers there, not two.
fc = driver(tc, "scale", 0)
transform(fc, "w", child, "LOC_X", space="WORLD_SPACE")
transform(fc, "l", child, "LOC_X", space="LOCAL_SPACE")
fc.driver.expression = "1.0 + w - l"
fc = driver(tc, "rotation_euler", 0)
transform(fc, "t", child, "LOC_X", space="TRANSFORM_SPACE")
fc.driver.expression = "t"
# Quaternion and Swing/Twist rotation modes.
fc = driver(tc, "scale", 1)
transform(fc, "q", src, "ROT_W", rot="QUATERNION")
fc.driver.expression = "q"
fc = driver(tc, "scale", 2)
transform(fc, "t", src, "ROT_Z", rot="SWING_TWIST_Z")
fc.driver.expression = "1.0 + t"

# DIFFS reads the distance and the angle between SOURCE and OTHER.
diffs = cube("DIFFS", (3, -3, 0))
fc = driver(diffs, "location", 0)
pair(fc, "d", "LOC_DIFF", src, other)
fc.driver.expression = "d"
fc = driver(diffs, "location", 1)
pair(fc, "a", "ROTATION_DIFF", src, other)
fc.driver.expression = "a"
fc = driver(diffs, "location", 2)
pair(fc, "d", "LOC_DIFF", src, other, space_a="LOCAL_SPACE", space_b="LOCAL_SPACE")
fc.driver.expression = "d"

# The custom-property chain: SOURCE.location.x → PROPHOST["gain"] → READER.
fc = driver(host, '["gain"]')
single(fc, "x", src, "location[0]")
fc.driver.expression = "x * 2.0"
reader = cube("READER", (-3, -3, 0))
fc = driver(reader, "location", 2)
single(fc, "g", host, '["gain"]')
fc.driver.expression = "g"
# A Min / Max / Average / Sum driver over two variables.
for i, kind in enumerate(["MIN", "MAX"]):
    fc = driver(reader, "scale", i)
    transform(fc, "a", src, "LOC_X")
    transform(fc, "b", other, "LOC_X")
    fc.driver.type = kind
fc = driver(reader, "scale", 2)
transform(fc, "a", src, "LOC_X")
transform(fc, "b", other, "LOC_X")
fc.driver.type = "AVERAGE"

bpy.ops.wm.save_as_mainfile(filepath=out)
print(f"-> {out}")
