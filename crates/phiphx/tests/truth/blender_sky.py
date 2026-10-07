# Blender's own Sky Texture, sampled as a flat equirectangular image — the
# reference for `phiphx::sky::BSky::bake` (`rendering.md` §62).
#
#   blender -b -P blender_sky.py -- OUT.txt [elevation_deg] [rotation_deg]
#
# The sky is *not* rendered through a camera: a world with a Sky Texture is
# rendered with a 2:1 equirectangular camera, which makes the frame itself
# the environment image Cycles would sample. 64 × 32 at 1 sample is enough —
# the sky is noiseless, being a texture — and the file prints one line per
# pixel as `x y r g b` so the Rust side can compare without an EXR reader.
import bpy, sys, os, math

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
out = args[0]
elevation = float(args[1]) if len(args) > 1 else 15.0
rotation = float(args[2]) if len(args) > 2 else 0.0
W, H = 64, 32

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
sc.render.engine = 'CYCLES'
sc.cycles.samples = 1
sc.cycles.device = 'CPU'
sc.render.resolution_x, sc.render.resolution_y = W, H
sc.render.resolution_percentage = 100
sc.render.film_transparent = False
sc.view_settings.view_transform = 'Standard'
sc.render.image_settings.file_format = 'OPEN_EXR'
sc.render.image_settings.color_depth = '32'

world = bpy.data.worlds.new("Sky")
sc.world = world
world.use_nodes = True
nt = world.node_tree
for n in list(nt.nodes):
    nt.nodes.remove(n)
bg = nt.nodes.new("ShaderNodeBackground")
out_node = nt.nodes.new("ShaderNodeOutputWorld")
nt.links.new(bg.outputs["Background"], out_node.inputs["Surface"])
sky = nt.nodes.new("ShaderNodeTexSky")
# The single-scattering Nishita model, which is what the Rust side bakes.
for name in ("SINGLE_SCATTERING", "NISHITA"):
    try:
        sky.sky_type = name
        break
    except TypeError:
        continue
sky.sun_elevation = math.radians(elevation)
sky.sun_rotation = math.radians(rotation)
sky.altitude = 0
sky.air_density = 1.0
sky.aerosol_density = 1.0
sky.ozone_density = 1.0
# The disc is compared on its own (`disc_radiance`); leaving it on would put
# one blown-out pixel in the frame and nothing else would be measurable.
sky.sun_disc = False
nt.links.new(sky.outputs["Color"], bg.inputs["Color"])

# An equirectangular camera looking along +X with Z up: the frame is then
# the environment image itself, u running right to left as `env_at` samples.
cam_data = bpy.data.cameras.new("Cam")
cam_data.type = 'PANO'
try:
    cam_data.panorama_type = 'EQUIRECTANGULAR'
except AttributeError:
    cam_data.cycles.panorama_type = 'EQUIRECTANGULAR'
cam = bpy.data.objects.new("Cam", cam_data)
sc.collection.objects.link(cam)
cam.location = (0, 0, 0)
cam.rotation_euler = (math.radians(90), 0, 0)
sc.camera = cam

path = os.path.abspath(out + ".exr")
sc.render.filepath = path
bpy.ops.render.render(write_still=True)

im = bpy.data.images.load(path)
px = list(im.pixels)
with open(out, "w") as fh:
    fh.write(f"# {W} {H} elevation {elevation} rotation {rotation}\n")
    for y in range(H):
        for x in range(W):
            i = (y * W + x) * 4
            fh.write(f"{x} {y} {px[i]:.8g} {px[i + 1]:.8g} {px[i + 2]:.8g}\n")
print(f"-> {out}")
