"""PROCEDURAL.blend: three planes whose materials read Noise / Voronoi
textures per pixel (`shade/procedural.rs`, `tests/procedural_truth.rs`).

  Ramped  — Principled Base Color ← Color Ramp (black → orange → white, ease)
            ← Noise(Scale 8, Detail 3, Roughness 0.6, Lacunarity 2.5) whose
            Vector ← Mapping(Location (0.1, 0.2, 0), Scale (2, 2, 1)) ←
            Texture Coordinate ▸ Object; Normal ← Bump(Strength 0.6,
            Distance 0.3) ▸ Height ← Voronoi ▸ Distance (Scale 12,
            Randomness 0.7), Generated.
  Mixed   — Mix Shader(Fac ← Noise ▸ Fac through a reroute; Diffuse red,
            Glossy white) on the output.
  Colored — Principled Base Color ← Voronoi ▸ Color (UV coordinates).

and one plane per texture node and setting (batch 143), each on
Principled Base Color: Checkered (Checker ▸ Color, red / blue, scale 3),
Bricks (Brick ▸ Color: offset 0.3 every 3rd row, squash 0.8 every 4th,
mortar 0.05), Graded (Gradient, Quadratic Sphere), Magical (Magic ▸ Color,
depth 4, distortion 1.5), Waved (Wave ▸ Fac: rings along Y, saw,
distortion 2, phase 1), Ridged (Noise ▸ Fac, Ridged Multifractal, offset
0.5, gain 2), Edged (Voronoi ▸ Distance, Distance to Edge, normalized),
Smoothed (Voronoi ▸ Distance, Smooth F1, Minkowski exponent 1.5,
smoothness 0.4), Hashed (White Noise ▸ Value, 4D, W 1.5), Gabored
(Gabor ▸ Value, 3D, Scale 12, Frequency 4, Anisotropy 0.25, Orientation
(0, 1, 0)), Mathed (a Noise ▸ Fac through a Math node — Multiply 2.5,
Clamp on — batch 160), and Turned (a Checker through a Mapping whose Type is
Texture, Location (0.1, 0, 0), Rotation 45° about Z, Scale (2, 2, 1) —
batch 151's rotation and mapping type).

Run with `python -c "import math

import bpy"` available (pip install bpy):
  python make_procedural.py
"""

import math

import bpy

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene


def plane(name, x, material):
    bpy.ops.mesh.primitive_plane_add(location=(x, 0, 0))
    ob = bpy.context.object
    ob.name = name
    ob.data.materials.append(material)
    return ob


def tree(name):
    ma = bpy.data.materials.new(name)
    ma.use_nodes = True
    nt = ma.node_tree
    for n in list(nt.nodes):
        nt.nodes.remove(n)
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    return ma, nt, out


# Ramped.
ma, nt, out = tree("Ramped")
bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
noise = nt.nodes.new("ShaderNodeTexNoise")
noise.inputs["Scale"].default_value = 8.0
noise.inputs["Detail"].default_value = 3.0
noise.inputs["Roughness"].default_value = 0.6
noise.inputs["Lacunarity"].default_value = 2.5
ramp = nt.nodes.new("ShaderNodeValToRGB")
ramp.color_ramp.interpolation = "EASE"
ramp.color_ramp.elements[0].color = (0.0, 0.0, 0.0, 1.0)
ramp.color_ramp.elements[1].position = 1.0
ramp.color_ramp.elements[1].color = (1.0, 1.0, 1.0, 1.0)
mid = ramp.color_ramp.elements.new(0.5)
mid.color = (1.0, 0.5, 0.0, 1.0)
nt.links.new(noise.outputs["Fac"], ramp.inputs["Fac"])
nt.links.new(ramp.outputs["Color"], bsdf.inputs["Base Color"])
mapping = nt.nodes.new("ShaderNodeMapping")
mapping.inputs["Location"].default_value = (0.1, 0.2, 0.0)
mapping.inputs["Scale"].default_value = (2.0, 2.0, 1.0)
coord = nt.nodes.new("ShaderNodeTexCoord")
nt.links.new(coord.outputs["Object"], mapping.inputs["Vector"])
nt.links.new(mapping.outputs["Vector"], noise.inputs["Vector"])
bump = nt.nodes.new("ShaderNodeBump")
bump.inputs["Strength"].default_value = 0.6
bump.inputs["Distance"].default_value = 0.3
voronoi = nt.nodes.new("ShaderNodeTexVoronoi")
voronoi.inputs["Scale"].default_value = 12.0
voronoi.inputs["Randomness"].default_value = 0.7
nt.links.new(voronoi.outputs["Distance"], bump.inputs["Height"])
nt.links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])
plane("Ramped", -2.5, ma)

# Mixed.
ma, nt, out = tree("Mixed")
mix = nt.nodes.new("ShaderNodeMixShader")
nt.links.new(mix.outputs["Shader"], out.inputs["Surface"])
diffuse = nt.nodes.new("ShaderNodeBsdfDiffuse")
diffuse.inputs["Color"].default_value = (0.8, 0.1, 0.1, 1.0)
glossy = nt.nodes.new("ShaderNodeBsdfMetallic")
nt.links.new(diffuse.outputs["BSDF"], mix.inputs[1])
nt.links.new(glossy.outputs["BSDF"], mix.inputs[2])
noise = nt.nodes.new("ShaderNodeTexNoise")
reroute = nt.nodes.new("NodeReroute")
nt.links.new(noise.outputs["Fac"], reroute.inputs[0])
nt.links.new(reroute.outputs[0], mix.inputs["Fac"])
plane("Mixed", 0.0, ma)

# Colored.
ma, nt, out = tree("Colored")
bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
voronoi = nt.nodes.new("ShaderNodeTexVoronoi")
coord = nt.nodes.new("ShaderNodeTexCoord")
nt.links.new(coord.outputs["UV"], voronoi.inputs["Vector"])
nt.links.new(voronoi.outputs["Color"], bsdf.inputs["Base Color"])
plane("Colored", 2.5, ma)



def textured(name, x, idname, output, setup):
    ma, nt, out = tree(name)
    bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
    nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    tex = nt.nodes.new(idname)
    setup(tex)
    nt.links.new(tex.outputs[output], bsdf.inputs["Base Color"])
    plane(name, x, ma)


def checker(t):
    t.inputs["Color1"].default_value = (1.0, 0.0, 0.0, 1.0)
    t.inputs["Color2"].default_value = (0.0, 0.0, 1.0, 1.0)
    t.inputs["Scale"].default_value = 3.0


def brick(t):
    t.offset, t.offset_frequency, t.squash, t.squash_frequency = 0.3, 3, 0.8, 4
    t.inputs["Mortar Size"].default_value = 0.05


def magic(t):
    t.turbulence_depth = 4
    t.inputs["Distortion"].default_value = 1.5


def wave(t):
    t.wave_type, t.rings_direction, t.wave_profile = "RINGS", "Y", "SAW"
    t.inputs["Distortion"].default_value = 2.0
    t.inputs["Phase Offset"].default_value = 1.0


def ridged(t):
    t.noise_type = "RIDGED_MULTIFRACTAL"
    t.inputs["Offset"].default_value = 0.5
    t.inputs["Gain"].default_value = 2.0


def edged(t):
    t.feature, t.normalize = "DISTANCE_TO_EDGE", True


def hashed(t):
    """White Noise ▸ Value in four dimensions, with W set."""
    t.noise_dimensions = "4D"
    t.inputs["W"].default_value = 1.5


def gabor(t):
    """Gabor ▸ Value in 3D, off its default Frequency, Anisotropy and
    Orientation."""
    t.gabor_type = "3D"
    t.inputs["Scale"].default_value = 12.0
    t.inputs["Frequency"].default_value = 4.0
    t.inputs["Anisotropy"].default_value = 0.25
    t.inputs["Orientation 3D"].default_value = (0.0, 1.0, 0.0)


def smoothed(t):
    t.feature, t.distance = "SMOOTH_F1", "MINKOWSKI"
    t.inputs["Exponent"].default_value = 1.5
    t.inputs["Smoothness"].default_value = 0.4


textured("Checkered", 5.0, "ShaderNodeTexChecker", "Color", checker)
textured("Bricks", 7.5, "ShaderNodeTexBrick", "Color", brick)
textured("Graded", 10.0, "ShaderNodeTexGradient", "Fac", lambda t: setattr(t, "gradient_type", "QUADRATIC_SPHERE"))
textured("Magical", 12.5, "ShaderNodeTexMagic", "Color", magic)
textured("Waved", 15.0, "ShaderNodeTexWave", "Fac", wave)
textured("Ridged", 17.5, "ShaderNodeTexNoise", "Fac", ridged)
textured("Edged", 20.0, "ShaderNodeTexVoronoi", "Distance", edged)
textured("Smoothed", 22.5, "ShaderNodeTexVoronoi", "Distance", smoothed)
textured("Hashed", 27.5, "ShaderNodeTexWhiteNoise", "Value", hashed)
textured("Gabored", 30.0, "ShaderNodeTexGabor", "Value", gabor)

# Mathed: a Math node between the texture and the socket — the node that,
# unhandled, lost the texture from the material altogether (batch 160).
ma, nt, out = tree("Mathed")
bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
noise = nt.nodes.new("ShaderNodeTexNoise")
noise.inputs["Scale"].default_value = 6.0
# `math` would shadow the module this script imports.
op = nt.nodes.new("ShaderNodeMath")
op.operation = "MULTIPLY"
op.use_clamp = True
op.inputs[1].default_value = 2.5
nt.links.new(noise.outputs["Fac"], op.inputs[0])
nt.links.new(op.outputs["Value"], bsdf.inputs["Base Color"])
plane("Mathed", 32.5, ma)

# Turned: the Mapping node whole — Type, Location, Rotation, Scale.
ma, nt, out = tree("Turned")
bsdf = nt.nodes.new("ShaderNodeBsdfPrincipled")
nt.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
checker = nt.nodes.new("ShaderNodeTexChecker")
nt.links.new(checker.outputs["Color"], bsdf.inputs["Base Color"])
mapping = nt.nodes.new("ShaderNodeMapping")
mapping.vector_type = "TEXTURE"
mapping.inputs["Location"].default_value = (0.1, 0.0, 0.0)
mapping.inputs["Rotation"].default_value = (0.0, 0.0, math.radians(45.0))
mapping.inputs["Scale"].default_value = (2.0, 2.0, 1.0)
coord = nt.nodes.new("ShaderNodeTexCoord")
nt.links.new(coord.outputs["Generated"], mapping.inputs["Vector"])
nt.links.new(mapping.outputs["Vector"], checker.inputs["Vector"])
plane("Turned", 25.0, ma)

bpy.ops.wm.save_as_mainfile(filepath=__file__.replace("make_procedural.py", "PROCEDURAL.blend"), compress=True)
