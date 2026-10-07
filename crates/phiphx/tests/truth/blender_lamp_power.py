# What one lamp of each kind, at a known power and distance, actually puts on
# a surface — Cycles' own numbers, in scene-linear radiance, for
# `phiray/tests/lamp_power.rs` to match (`rendering.md` §54).
#
#   blender -b -P blender_lamp_power.py [-- OUT.txt]
#
# The scene is built here rather than loaded, so nothing about it can drift:
# a 20 x 20 m plane at z = 0 with a pure Lambertian albedo of 0.8, an
# orthographic camera 10 m above it looking straight down, and one lamp at
# (0, 0, 2) pointing down. The reported number is the centre pixel's linear
# radiance — no view transform, no film curve — so it is the one quantity a
# kernel can be compared against without arguing about tone mapping.
import bpy, sys, os, math

out = None
if "--" in sys.argv:
    rest = sys.argv[sys.argv.index("--") + 1:]
    out = rest[0] if rest else None

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
sc.render.engine = 'CYCLES'
sc.cycles.device = 'CPU'
sc.cycles.samples = 256
# One bounce only: the number must be the lamp's direct light on the plane,
# not the plane lighting itself through the (absent) world.
sc.cycles.max_bounces = 1
sc.cycles.diffuse_bounces = 1
sc.render.resolution_x = sc.render.resolution_y = 32
sc.render.resolution_percentage = 100
sc.view_settings.view_transform = 'Standard'
sc.render.film_transparent = False
if hasattr(sc.render.image_settings, "media_type"):
    sc.render.image_settings.media_type = 'IMAGE'
sc.render.image_settings.file_format = 'OPEN_EXR'
sc.render.image_settings.color_depth = '32'

# A black world, so the only light is the lamp.
w = bpy.data.worlds.new("black"); sc.world = w
w.use_nodes = True
bg = w.node_tree.nodes["Background"]
bg.inputs["Color"].default_value = (0, 0, 0, 1)

bpy.ops.mesh.primitive_plane_add(size=20.0, location=(0, 0, 0))
plane = bpy.context.object
ma = bpy.data.materials.new("grey"); ma.use_nodes = True
nt = ma.node_tree; nt.nodes.clear()
outn = nt.nodes.new("ShaderNodeOutputMaterial")
# The same surface the tracer's `Tri::with_pbr(1.0, 0.0)` is: a Principled
# at Base Color 0.8, Roughness 1, Metallic 0 and the default Specular IOR
# Level 0.5 (f0 = 0.04). A *pure* diffuse BSDF would be the simpler
# reference, but it is not the same surface — phiray carries a GGX lobe on
# top of its Lambert, and against a pure diffuse that reads as 12 % too
# bright for a reason that is not the lamp (`rendering.md` §54).
diff = nt.nodes.new("ShaderNodeBsdfPrincipled")
diff.inputs["Base Color"].default_value = (0.8, 0.8, 0.8, 1.0)
diff.inputs["Roughness"].default_value = 1.0
diff.inputs["Metallic"].default_value = 0.0
diff.inputs["Specular IOR Level"].default_value = 0.5
nt.links.new(diff.outputs["BSDF"], outn.inputs["Surface"])
plane.data.materials.append(ma)

cam = bpy.data.cameras.new("top"); cam.type = 'ORTHO'; cam.ortho_scale = 2.0
cob = bpy.data.objects.new("top", cam); sc.collection.objects.link(cob)
cob.location = (0, 0, 10); cob.rotation_euler = (0, 0, 0); sc.camera = cob

HEIGHT = 2.0
CASES = [
    ("point",  'POINT', 1000.0, {"shadow_soft_size": 0.0}),
    ("point_r", 'POINT', 1000.0, {"shadow_soft_size": 0.25}),
    ("spot",   'SPOT',  1000.0, {"shadow_soft_size": 0.0, "spot_size": math.radians(60.0), "spot_blend": 0.0}),
    ("sun",    'SUN',     5.0,  {"angle": 0.0}),
    ("area",   'AREA',  100.0,  {"shape": 'SQUARE', "size": 1.0}),
]

def read_centre(path):
    im = bpy.data.images.load(path)
    w_, h_ = im.size
    px = list(im.pixels)
    i = ((h_ // 2) * w_ + (w_ // 2)) * 4
    v = px[i:i + 3]
    bpy.data.images.remove(im)
    return v

lines = []
for name, kind, power, fields in CASES:
    data = bpy.data.lights.new(name, type=kind)
    data.energy = power
    for k, v in fields.items():
        setattr(data, k, v)
    ob = bpy.data.objects.new(name, data)
    sc.collection.objects.link(ob)
    ob.location = (0, 0, HEIGHT)
    # Down the -Z axis, which is a lamp's own direction.
    ob.rotation_euler = (0, 0, 0)
    f = os.path.abspath(f"/tmp/lamp_{name}.exr")
    sc.render.filepath = f
    bpy.ops.render.render(write_still=True)
    v = read_centre(f + (".exr" if not f.endswith(".exr") else ""))
    line = f"{name} {kind} power={power} height={HEIGHT} albedo=0.8 radiance={v[0]:.6f}"
    print(line)
    lines.append(line)
    bpy.data.objects.remove(ob, do_unlink=True)

# The world, the same way. A uniform dome of radiance L over a Lambertian
# albedo `a` gives irradiance πL (the hemisphere), so the surface's own
# radiance is a·E/π = a·L exactly — no distance, no geometry, no constant to
# get wrong. The plane is the only thing in the scene, so what comes back is
# the dome plus whatever the plane bounces onto itself.
for name, colour, strength in [("world_1", (1.0, 1.0, 1.0, 1.0), 1.0),
                               ("world_half", (0.5, 0.5, 0.5, 1.0), 1.0),
                               ("world_magenta", (1.0, 0.0, 1.0, 1.0), 1.0)]:
    bg.inputs["Color"].default_value = colour
    bg.inputs["Strength"].default_value = strength
    f = os.path.abspath(f"/tmp/lamp_{name}.exr")
    sc.render.filepath = f
    bpy.ops.render.render(write_still=True)
    v = read_centre(f)
    line = f"{name} WORLD colour={colour[:3]} strength={strength} albedo=0.8 radiance=({v[0]:.6f}, {v[1]:.6f}, {v[2]:.6f})"
    print(line)
    lines.append(line)

# A **non-uniform** environment image. A flat world is the easy case; an
# HDRI is the one every real file uses, and it goes through different code
# on our side — sampled with a table, prefiltered, weighted against the
# cosine ray. The sky here is a bright band around the zenith over a dim
# rest, written as an EXR so both renderers read the same bytes
# (`rendering.md` §59). `sky.exr` lands beside this script.
import math as _m
ENV_W, ENV_H = 64, 32
sky = bpy.data.images.new("sky", width=ENV_W, height=ENV_H, float_buffer=True)
px = [0.0] * (ENV_W * ENV_H * 4)
for y in range(ENV_H):
    # Equirectangular: row 0 is the bottom in Blender's buffer, so elevation
    # runs from -90 at y=0 to +90 at y=ENV_H-1.
    elev = (y + 0.5) / ENV_H * _m.pi - _m.pi / 2
    v = 4.0 if elev > _m.radians(50.0) else 0.1
    for x in range(ENV_W):
        i = (y * ENV_W + x) * 4
        px[i:i + 4] = [v, v, v, 1.0]
sky.pixels = px
sky_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "sky.exr"))
sky.filepath_raw = sky_path
sky.file_format = 'OPEN_EXR'
sky.save()
print("wrote", sky_path)

env_node = w.node_tree.nodes.new("ShaderNodeTexEnvironment")
env_node.image = sky
w.node_tree.links.new(env_node.outputs["Color"], bg.inputs["Color"])
bg.inputs["Strength"].default_value = 1.0
for name, bounces in [("env_b1", 1), ("env_b4", 4)]:
    sc.cycles.max_bounces = bounces
    sc.cycles.diffuse_bounces = bounces
    f = os.path.abspath(f"/tmp/lamp_{name}.exr")
    sc.render.filepath = f
    bpy.ops.render.render(write_still=True)
    v = read_centre(f)
    line = f"{name} WORLD_IMAGE band=4.0 above 50deg, 0.1 below bounces={bounces} albedo=0.8 radiance={v[0]:.6f}"
    print(line)
    lines.append(line)
w.node_tree.links.remove(bg.inputs["Color"].links[0])
bg.inputs["Color"].default_value = (0, 0, 0, 1)
sc.cycles.max_bounces = 1
sc.cycles.diffuse_bounces = 1

# The specular lobe on its own. Base Color black, so the diffuse term is
# zero and what comes back is only the microfacet reflection of the lamp.
# Lamp and camera are both straight overhead, which puts the half-vector on
# the normal — the lobe's peak, where its normalization shows most — and
# the roughness sweeps across it (`rendering.md` §58).
bg.inputs["Color"].default_value = (0, 0, 0, 1)
diff.inputs["Base Color"].default_value = (0.0, 0.0, 0.0, 1.0)
spec_lamp = bpy.data.lights.new("spec", type='POINT')
spec_lamp.energy = 1000.0
spec_lamp.shadow_soft_size = 0.0
spec_ob = bpy.data.objects.new("spec", spec_lamp)
sc.collection.objects.link(spec_ob)
spec_ob.location = (0, 0, HEIGHT)
for rough in (0.1, 0.3, 0.6, 1.0):
    diff.inputs["Roughness"].default_value = rough
    f = os.path.abspath(f"/tmp/lamp_spec{int(rough*100):03d}.exr")
    sc.render.filepath = f
    bpy.ops.render.render(write_still=True)
    v = read_centre(f)
    line = f"spec_r{int(rough*100):03d} SPECULAR roughness={rough} f0=0.04 power=1000 height={HEIGHT} radiance={v[0]:.6f}"
    print(line)
    lines.append(line)
bpy.data.objects.remove(spec_ob, do_unlink=True)
diff.inputs["Base Color"].default_value = (0.8, 0.8, 0.8, 1.0)
diff.inputs["Roughness"].default_value = 1.0

# The same uniform dome, with something in the way. Two walls a metre high
# and a metre apart make a trench along Y; the camera looks straight down
# into it and the centre pixel is the floor between them. That floor sees
# only the strip of sky the trench leaves open, so what reaches it is a
# fraction of the dome plus whatever bounces off the walls. This is the
# number that says whether a renderer's indirect light is a real path or a
# stand-in — an ambient-occlusion approximation with a floor cannot get it
# right (`rendering.md` §56).
bg.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
bg.inputs["Strength"].default_value = 1.0
walls = []
for x in (-0.5, 0.5):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(x, 0, 0.5))
    wall = bpy.context.object
    wall.scale = (0.02, 10.0, 1.0)
    wall.data.materials.append(ma)
    walls.append(wall)
for name, bounces in [("trench_b1", 1), ("trench_b4", 4), ("trench_b12", 12)]:
    sc.cycles.max_bounces = bounces
    sc.cycles.diffuse_bounces = bounces
    f = os.path.abspath(f"/tmp/lamp_{name}.exr")
    sc.render.filepath = f
    bpy.ops.render.render(write_still=True)
    v = read_centre(f)
    line = f"{name} WORLD_TRENCH walls=1m apart, 1m high bounces={bounces} albedo=0.8 radiance={v[0]:.6f}"
    print(line)
    lines.append(line)
for wall in walls:
    bpy.data.objects.remove(wall, do_unlink=True)
sc.cycles.max_bounces = 1
sc.cycles.diffuse_bounces = 1
bg.inputs["Color"].default_value = (0, 0, 0, 1)

text = "\n".join(lines) + "\n"
if out:
    open(out, "w").write(text)
print(text)
