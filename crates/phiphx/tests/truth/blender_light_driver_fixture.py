# Builds `fixtures/LIGHTDRIVERS.blend`: a driver on every light property a
# `Light` can carry, across all four lamp types, plus the world's own colour
# and its Background node's sockets.
#
#   python -I blender_light_driver_fixture.py <out.blend>
#
# A light has no node tree in Blender 5.x, so its drivers are all on the
# `Light` ID itself. A world's are on both: `World.color` on the ID, and the
# Background node's `Strength` and `Color` on `World.node_tree`.
#
# Every driver here is pushed OUT of its property's range, because the hard
# ranges are what distinguishes these properties and they are not uniform:
# `energy` is unbounded (a driver may make it negative), `color` and the
# sizes clamp at 0, `spot_size` has a non-zero hard min of 1°, `angle` and
# `spot_blend` clamp at both ends.
import bpy, math, sys

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
out = args[0] if args else "LIGHTDRIVERS.blend"

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
sc.frame_start, sc.frame_end = 1, 21

# ── SOURCE: keyed well outside 0..1 so each clamp end is reached.
bpy.ops.mesh.primitive_cube_add(size=1.0)
src = bpy.context.object
src.name = "SOURCE"
for frame, (x, z, s) in {1: (0.0, -2.0, 1.0), 11: (1.5, 0.6, 2.0), 21: (-0.5, 3.0, 0.25)}.items():
    src.location = (x, 0.0, z)
    src.scale = (s, s, s)
    for path in ("location", "scale"):
        src.keyframe_insert(path, frame=frame)

# PROPHOST carries a driven custom property, read back by a light driver —
# the depsgraph ordering, proved again through a different ID type.
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(0, 3, 0))
host = bpy.context.object
host.name = "PROPHOST"
host["gain"] = 0.0
host.id_properties_ui("gain").update(min=-100.0, max=100.0)


def z_var(driver, name="z", kind="LOC_Z"):
    v = driver.variables.new()
    v.name = name
    v.type = "TRANSFORMS"
    t = v.targets[0]
    t.id = src
    t.transform_type = kind
    t.transform_space = "WORLD_SPACE"


def prop_var(driver, name, target, path):
    v = driver.variables.new()
    v.name = name
    v.type = "SINGLE_PROP"
    v.targets[0].id = target
    v.targets[0].data_path = path


def scripted(fcurve, expression, setup=z_var):
    fcurve.driver.type = "SCRIPTED"
    fcurve.driver.expression = expression
    setup(fcurve.driver)
    return fcurve


def lamp(kind, name, loc):
    bpy.ops.object.light_add(type=kind, location=loc)
    ob = bpy.context.object
    ob.name = f"OB_{name}"
    ob.data.name = name
    return ob.data


# ── L_POINT: `energy` is the one unbounded property, so it goes negative.
point = lamp("POINT", "L_POINT", (3, 0, 2))
scripted(point.driver_add("energy"), "z * 50.0")             # unbounded: stays negative
scripted(point.driver_add("color", 0), "z")                  # hard 0..inf
scripted(point.driver_add("color", 2), "1.0 - z * 0.25")
scripted(point.driver_add("shadow_soft_size"), "z * 0.2")    # hard 0..inf

# ── L_SPOT: `spot_size` has a hard min of 1° (0.01745 rad), not 0, and a
# hard max of π; `spot_blend` is a plain 0..1.
spot = lamp("SPOT", "L_SPOT", (6, 0, 2))
scripted(spot.driver_add("spot_size"), "z")                  # hard 0.01745..π
scripted(spot.driver_add("spot_blend"), "z * 0.5")           # hard 0..1
scripted(spot.driver_add("energy"), "100.0 + z * 10.0")

# ── L_SUN: `angle` is 0..π.
sun = lamp("SUN", "L_SUN", (9, 0, 4))
scripted(sun.driver_add("angle"), "z")                       # hard 0..π
scripted(sun.driver_add("color", 1), "0.5 + z * 0.3")

# ── L_AREA: `size` and `size_y` both clamp at 0, and one goes negative.
area = lamp("AREA", "L_AREA", (12, 0, 2))
area.shape = "RECTANGLE"
scripted(area.driver_add("size"), "z")                       # hard 0..inf
scripted(area.driver_add("size_y"), "1.0 - z * 0.5")
# A muted driver: the saved value stands.
area.energy = 42.0
muted = scripted(area.driver_add("energy"), "z")
muted.mute = True

# ── The world: its own colour on the ID, its Background node on the tree.
world = bpy.data.worlds.new("W_DRIVEN")
sc.world = world
world.use_nodes = True
background = world.node_tree.nodes["Background"]
scripted(world.driver_add("color", 1), "z * 0.5")                           # hard 0..inf
scripted(background.inputs[1].driver_add("default_value"), "z")             # Strength, unclamped
scripted(background.inputs[0].driver_add("default_value", 0), "z")          # Color R, clamps at 0
scripted(background.inputs[0].driver_add("default_value", 2), "0.25 * z")
# And one read off the driven custom property.
scripted(point.driver_add("color", 1), "gain", lambda d: prop_var(d, "gain", host, '["gain"]'))
scripted(host.driver_add('["gain"]'), "z * 0.4")

bpy.ops.wm.save_as_mainfile(filepath=out, compress=True)
print("wrote", out)
