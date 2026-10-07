"""SKY.blend: a world whose Background is a Sky Texture, for the reader test
(`phiphx/tests/sky_node_truth.rs`). Single scattering, the sun 22.5 degrees
up and 40 degrees round, thick air, light aerosols, heavy ozone, a disc of
one degree at double intensity, 1200 m up — every field off its default so
a reader that dropped one is caught."""
import math

import bpy

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
world = bpy.data.worlds.new("Sky")
sc.world = world
world.use_nodes = True
nt = world.node_tree
for n in list(nt.nodes):
    nt.nodes.remove(n)
out = nt.nodes.new("ShaderNodeOutputWorld")
bg = nt.nodes.new("ShaderNodeBackground")
bg.inputs["Strength"].default_value = 1.5
nt.links.new(bg.outputs["Background"], out.inputs["Surface"])
sky = nt.nodes.new("ShaderNodeTexSky")
sky.sky_type = "SINGLE_SCATTERING"
sky.sun_elevation = math.radians(22.5)
sky.sun_rotation = math.radians(40.0)
sky.altitude = 1200
sky.air_density = 1.7
sky.aerosol_density = 0.4
sky.ozone_density = 3.0
sky.sun_disc = True
sky.sun_size = math.radians(1.0)
sky.sun_intensity = 2.0
nt.links.new(sky.outputs["Color"], bg.inputs["Color"])

# A second world naming an analytic model, which the reader must decline
# rather than read as the atmosphere one.
hosek = bpy.data.worlds.new("Hosek")
hosek.use_nodes = True
ht = hosek.node_tree
for n in list(ht.nodes):
    ht.nodes.remove(n)
hout = ht.nodes.new("ShaderNodeOutputWorld")
hbg = ht.nodes.new("ShaderNodeBackground")
ht.links.new(hbg.outputs["Background"], hout.inputs["Surface"])
hsky = ht.nodes.new("ShaderNodeTexSky")
hsky.sky_type = "HOSEK_WILKIE"
ht.links.new(hsky.outputs["Color"], hbg.inputs["Color"])

bpy.ops.wm.save_as_mainfile(filepath=__file__.replace("make_sky.py", "SKY.blend"), compress=True)
