# Builds `fixtures/MATDRIVERS.blend`: material drivers of every kind a file
# can carry, so each write path has a case the reader can be held to.
#
#   python -I blender_mat_driver_fixture.py <out.blend>
#
# Blender stores a material's drivers on two different IDs: a driver on the
# material's own property (`roughness`, `diffuse_color`) lives on
# `Material.animation_data`, and a driver on a shader socket lives on
# `Material.node_tree.animation_data` with the path
# `nodes["<name>"].inputs[<i>].default_value`. Both are here.
#
# The three clamp rules Blender applies when a driver writes are each
# covered, because they disagree: a float or vector socket is NOT clamped
# even when its RNA range says 0..1, an RGBA socket's components clamp at 0
# below but pass above 1, and a Material's own float property clamps to its
# RNA hard range at both ends.
import bpy, math, sys

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
out = args[0] if args else "MATDRIVERS.blend"

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
sc.frame_start, sc.frame_end = 1, 21


def cube(name, loc=(0, 0, 0)):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    ob = bpy.context.object
    ob.name = name
    return ob


# ── SOURCE: the object every driver reads. Keyed well past 0..1 on Z so the
# clamp rules are visible, and keyed in X and scale for the other variables.
src = cube("SOURCE")
src.rotation_mode = "XYZ"
for frame, (x, z, s) in {1: (0.0, -2.0, 1.0), 11: (1.5, 0.6, 2.0), 21: (-0.5, 3.0, 0.25)}.items():
    src.location = (x, 0.0, z)
    src.rotation_euler = (0.0, 0.0, math.radians(35.0) * x)
    src.scale = (s, s, s)
    for path in ("location", "rotation_euler", "scale"):
        src.keyframe_insert(path, frame=frame)

# PROPHOST carries a custom property driven off SOURCE — the chain that says
# the material pass runs after the objects' custom-property drivers.
host = cube("PROPHOST", (0, 3, 0))
host["gain"] = 0.0
host.id_properties_ui("gain").update(min=-100.0, max=100.0)


def z_var(driver, name="z", kind="LOC_Z", target=None, space="WORLD_SPACE"):
    v = driver.variables.new()
    v.name = name
    v.type = "TRANSFORMS"
    t = v.targets[0]
    t.id = target or src
    t.transform_type = kind
    t.transform_space = space
    return v


def prop_var(driver, name, target, path):
    v = driver.variables.new()
    v.name = name
    v.type = "SINGLE_PROP"
    v.targets[0].id = target
    v.targets[0].data_path = path
    return v


def scripted(fcurve, expression, setup=z_var):
    fcurve.driver.type = "SCRIPTED"
    fcurve.driver.expression = expression
    setup(fcurve.driver)
    return fcurve


def material(name, obj_loc):
    ob = cube(name.replace("MAT_", "OB_"), obj_loc)
    m = bpy.data.materials.new(name)
    ob.data.materials.append(m)
    m.use_nodes = True
    return ob, m, m.node_tree, m.node_tree.nodes["Principled BSDF"]


# ── MAT_OWN: drivers on the Material ID's own properties. Each one is
# clamped to its RNA hard range, so `z` running -2..3 shows both ends.
_, own, _, _ = material("MAT_OWN", (3, 0, 0))
scripted(own.driver_add("roughness"), "z * 0.5")              # hard 0..1, clamps both ends
scripted(own.driver_add("metallic"), "1.0 - z")               # hard 0..1, clamps both ends
scripted(own.driver_add("diffuse_color", 0), "z")             # hard 0..inf: 0 below, free above
scripted(own.driver_add("diffuse_color", 2), "0.5 - z * 0.2")
scripted(own.driver_add("specular_intensity"), "z * 0.4")     # hard 0..1

# ── MAT_SOCK: drivers on the Principled BSDF's sockets. A float socket is
# NOT clamped by its declared range; an RGBA socket's component clamps at 0.
_, sock, sock_tree, sock_node = material("MAT_SOCK", (6, 0, 0))
scripted(sock_node.inputs[2].driver_add("default_value"), "z * 0.5")   # Roughness, unclamped
scripted(sock_node.inputs[1].driver_add("default_value"), "1.0 - z")   # Metallic, unclamped
scripted(sock_node.inputs[3].driver_add("default_value"), "1.45 + z * 0.1")  # IOR
scripted(sock_node.inputs[28].driver_add("default_value"), "z * 2.0")  # Emission Strength
scripted(sock_node.inputs[0].driver_add("default_value", 0), "z")      # Base Color R, clamps at 0
scripted(sock_node.inputs[0].driver_add("default_value", 2), "0.25 * z")
scripted(sock_node.inputs[19].driver_add("default_value"), "abs(z) * 0.3")  # Coat Weight
# A Sum driver over two Transform Channel variables, not an expression.
fc = sock_node.inputs[24].driver_add("default_value")           # Sheen Weight
fc.driver.type = "SUM"
z_var(fc.driver, "a", "LOC_X")
z_var(fc.driver, "b", "SCALE_Z")

# ── MAT_EDGE: the cases that are easy to get wrong.
_, edge, edge_tree, edge_node = material("MAT_EDGE", (9, 0, 0))
# 1. A driver on a LINKED socket. Blender writes `default_value` anyway, but
#    the link feeds the shader, so the written value never reaches a pixel.
noise = edge_tree.nodes.new("ShaderNodeTexNoise")
edge_tree.links.new(noise.outputs["Fac"], edge_node.inputs[2])
scripted(edge_node.inputs[2].driver_add("default_value"), "z")
# 2. A MUTED driver: the socket keeps the value the file saved.
edge_node.inputs[1].default_value = 0.35
muted = scripted(edge_node.inputs[1].driver_add("default_value"), "z")
muted.mute = True
# 3. The driver's own F-curve, with keys: the driver's value is the curve's
#    X, not the frame. 0 → 0.9, 2 → 0.1, so a rising `z` falls here.
shaped = scripted(edge_node.inputs[20].driver_add("default_value"), "z")  # Coat Roughness
for x, y in ((0.0, 0.9), (2.0, 0.1)):
    k = shaped.keyframe_points.insert(x, y)
    k.interpolation = "LINEAR"
# 4. A socket driven off a custom property that is itself driven — the chain.
scripted(edge_node.inputs[28].driver_add("default_value"), "gain",
         lambda d: prop_var(d, "gain", host, '["gain"]'))
# and the property's own driver, on the object
scripted(host.driver_add('["gain"]'), "z * 0.75")

# ── A second material slot on one object, so slot order is exercised too.
ob_two = bpy.data.objects["OB_SOCK"]
second = bpy.data.materials.new("MAT_SLOT2")
ob_two.data.materials.append(second)
second.use_nodes = True
scripted(second.driver_add("roughness"), "0.5 - z * 0.25")

bpy.ops.wm.save_as_mainfile(filepath=out, compress=True)
print("wrote", out)
