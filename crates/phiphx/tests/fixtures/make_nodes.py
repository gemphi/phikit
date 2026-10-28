"""NODES.blend: one plane per material, for the value / vector / colour
nodes and node groups the fold reads (`shade/fold.rs`, `shade/group.rs`,
`tests/nodes_truth.rs`). The first ten feed an Emission's Color with a
chain of constant inputs, so Cycles' Emit bake of the plane is the chain's
value (`truth/blender_nodes.py` → `truth/nodes.txt`):

  MapRanged     — Combine Color RGB ← (Map Range linear 0.3: 0..1 → 0.2..0.8,
                  Map Range stepped 0.3 at 4 steps, Map Range smoothstep 0.25)
  MapClamped    — (linear 2.0 → 0..1 clamped, linear 1.5 → 0..0.5 unclamped,
                  the same clamped)
  Separated     — Separate Color HSV of (0.2, 0.6, 0.4) → R G B
  SeparatedHSL  — the same in HSL
  Combined      — Combine Color HSV (0.1, 0.5, 0.8)
  CombinedHSL   — Combine Color HSL (0.6, 0.4, 0.5)
  Vectored      — Combine XYZ (0.1, 0.5, 0.9) → Vector Rotate (Z axis, 90°,
                  centre (0.3, 0.3, 0)) → Vector Math Multiply (0.5, 2, 0.5)
  Dotted        — (Vector Math Dot (0.3, 0.4, 0.5)·(0.5, 0.5, 0.5),
                  Separate XYZ (0.7, 0.2, 0.9) ▸ Y, Normal (0, 0, 1) ▸ Dot
                  of (0.6, 0, 0.8))
  Grouped       — group NodeChain(Value 0.3, Hue 0.1): Map Range linear
                  0..1 → 0.2..0.8 → Combine Color HSV (Hue, result, 0.9)
  GroupedLinked — NodeChain(Value ← Value node 0.5, Hue 0.7)
  Nested        — group Outer(Value 0.8) wrapping NodeChain(Value, Hue 0.25)

and seven read structurally (`tests/nodes_truth.rs` alone):

  Displaced     — Material Output Displacement ← Displacement (Scale 0.4,
                  Midlevel 0.5) ▸ Height ← Noise ▸ Fac
  FresnelMixed  — Mix Shader (Fac ← Fresnel IOR 1.8; Diffuse red, Glossy)
  Facing        — Mix Shader (Fac ← Layer Weight ▸ Facing, Blend 0.3)
  Remapped      — Base Color ← Color Ramp (black → white) ← Map Range
                  linear 0..1 → 0..0.5 ← Noise ▸ Fac
  Tailed        — Base Color ← Map Range linear 0..1 → 0.25..0.75 (clamp)
                  ← Noise ▸ Fac
  VertexColored — Base Color ← Vertex Color "Col"
  WhiteNoised   — Base Color ← group Hash(White Noise ▸ Value)
  MixedWood     — Base Color ← Mix RGBA (0.35) of a Color Ramp over a
                  banded Wave (scale 7, distortion 0.4, Y) and a Noise
                  (scale 3, detail 8) over a Mapping scaled (1, 22, 1) —
                  the BASKETBALL-COURT floor, two textures on one socket

Run with `python crates/phiphx/tests/fixtures/make_nodes.py` (pip bpy).
"""
import math
import os

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))


def by_id(sockets, ident):
    """A socket by its identifier (Blender 4.x+ renamed labels: `Fac` reads
    "Factor"), or None when the node's declaration leaves it out."""
    for s in sockets:
        if s.identifier == ident:
            return s
    return None

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.render.engine = "CYCLES"


def plane(name, x):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([(-1, -1, 0), (1, -1, 0), (1, 1, 0), (-1, 1, 0)], [], [(0, 1, 2, 3)])
    uv = mesh.uv_layers.new(name="UVMap")
    for i, co in enumerate([(0, 0), (1, 0), (1, 1), (0, 1)]):
        uv.data[i].uv = co
    col = mesh.color_attributes.new(name="Col", type="BYTE_COLOR", domain="CORNER")
    for i in range(4):
        col.data[i].color = (0.9, 0.3, 0.1, 1.0)
    ob = bpy.data.objects.new(name, mesh)
    ob.location = (x * 2.5, 0, 0)
    scene.collection.objects.link(ob)
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    mesh.materials.append(mat)
    nt = mat.node_tree
    for n in list(nt.nodes):
        nt.nodes.remove(n)
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    return nt, out


def emission(nt, out):
    em = nt.nodes.new("ShaderNodeEmission")
    em.inputs["Strength"].default_value = 1.0
    nt.links.new(em.outputs["Emission"], out.inputs["Surface"])
    return em


def map_range(nt, value, kind="LINEAR", fr=(0.0, 1.0), to=(0.0, 1.0), steps=4.0, clamp=False):
    n = nt.nodes.new("ShaderNodeMapRange")
    n.data_type = "FLOAT"
    n.interpolation_type = kind
    n.clamp = clamp
    n.inputs["Value"].default_value = value
    n.inputs["From Min"].default_value = fr[0]
    n.inputs["From Max"].default_value = fr[1]
    n.inputs["To Min"].default_value = to[0]
    n.inputs["To Max"].default_value = to[1]
    if by_id(n.inputs, "Steps") is not None:
        by_id(n.inputs, "Steps").default_value = steps
    return n


def combine(nt, mode, r, g, b):
    n = nt.nodes.new("ShaderNodeCombineColor")
    n.mode = mode
    for s, v in zip(("Red", "Green", "Blue"), (r, g, b)):
        if isinstance(v, float):
            n.inputs[s].default_value = v
        else:
            nt.links.new(v, n.inputs[s])
    return n


# --- emission chains ---------------------------------------------------
nt, out = plane("MapRanged", 0)
em = emission(nt, out)
c = combine(nt, "RGB", map_range(nt, 0.3, to=(0.2, 0.8)).outputs["Result"],
            map_range(nt, 0.3, "STEPPED").outputs["Result"], map_range(nt, 0.25, "SMOOTHSTEP").outputs["Result"])
nt.links.new(c.outputs["Color"], em.inputs["Color"])

nt, out = plane("MapClamped", 1)
em = emission(nt, out)
c = combine(nt, "RGB", map_range(nt, 2.0, clamp=True).outputs["Result"],
            map_range(nt, 1.5, to=(0.0, 0.5)).outputs["Result"], map_range(nt, 1.5, to=(0.0, 0.5), clamp=True).outputs["Result"])
nt.links.new(c.outputs["Color"], em.inputs["Color"])

for name, mode, x in (("Separated", "HSV", 2), ("SeparatedHSL", "HSL", 3)):
    nt, out = plane(name, x)
    em = emission(nt, out)
    sep = nt.nodes.new("ShaderNodeSeparateColor")
    sep.mode = mode
    sep.inputs["Color"].default_value = (0.2, 0.6, 0.4, 1.0)
    c = combine(nt, "RGB", sep.outputs["Red"], sep.outputs["Green"], sep.outputs["Blue"])
    nt.links.new(c.outputs["Color"], em.inputs["Color"])

for name, mode, hsx, x in (("Combined", "HSV", (0.1, 0.5, 0.8), 4), ("CombinedHSL", "HSL", (0.6, 0.4, 0.5), 5)):
    nt, out = plane(name, x)
    em = emission(nt, out)
    c = combine(nt, mode, *hsx)
    nt.links.new(c.outputs["Color"], em.inputs["Color"])

nt, out = plane("Vectored", 6)
em = emission(nt, out)
xyz = nt.nodes.new("ShaderNodeCombineXYZ")
for s, v in zip("XYZ", (0.1, 0.5, 0.9)):
    xyz.inputs[s].default_value = v
rot = nt.nodes.new("ShaderNodeVectorRotate")
rot.rotation_type = "Z_AXIS"
rot.inputs["Center"].default_value = (0.3, 0.3, 0.0)
rot.inputs["Angle"].default_value = math.radians(90)
nt.links.new(xyz.outputs["Vector"], rot.inputs["Vector"])
mul = nt.nodes.new("ShaderNodeVectorMath")
mul.operation = "MULTIPLY"
mul.inputs[1].default_value = (0.5, 2.0, 0.5)
nt.links.new(rot.outputs["Vector"], mul.inputs[0])
nt.links.new(mul.outputs["Vector"], em.inputs["Color"])

nt, out = plane("Dotted", 7)
em = emission(nt, out)
dot = nt.nodes.new("ShaderNodeVectorMath")
dot.operation = "DOT_PRODUCT"
dot.inputs[0].default_value = (0.3, 0.4, 0.5)
dot.inputs[1].default_value = (0.5, 0.5, 0.5)
sep = nt.nodes.new("ShaderNodeSeparateXYZ")
sep.inputs["Vector"].default_value = (0.7, 0.2, 0.9)
nor = nt.nodes.new("ShaderNodeNormal")
nor.outputs["Normal"].default_value = (0.0, 0.0, 1.0)
nor.inputs["Normal"].default_value = (0.6, 0.0, 0.8)
c = combine(nt, "RGB", dot.outputs["Value"], sep.outputs["Y"], nor.outputs["Dot"])
nt.links.new(c.outputs["Color"], em.inputs["Color"])

# --- node groups ------------------------------------------------------
chain = bpy.data.node_groups.new("NodeChain", "ShaderNodeTree")
chain.interface.new_socket(name="Value", in_out="INPUT", socket_type="NodeSocketFloat")
chain.interface.new_socket(name="Hue", in_out="INPUT", socket_type="NodeSocketFloat")
chain.interface.new_socket(name="Color", in_out="OUTPUT", socket_type="NodeSocketColor")
gi = chain.nodes.new("NodeGroupInput")
go = chain.nodes.new("NodeGroupOutput")
mr = map_range(chain, 0.0, to=(0.2, 0.8))
chain.links.new(gi.outputs["Value"], mr.inputs["Value"])
cc = combine(chain, "HSV", gi.outputs["Hue"], mr.outputs["Result"], 0.9)
chain.links.new(cc.outputs["Color"], go.inputs["Color"])

nt, out = plane("Grouped", 8)
em = emission(nt, out)
g = nt.nodes.new("ShaderNodeGroup")
g.node_tree = chain
g.inputs["Value"].default_value = 0.3
g.inputs["Hue"].default_value = 0.1
nt.links.new(g.outputs["Color"], em.inputs["Color"])

nt, out = plane("GroupedLinked", 9)
em = emission(nt, out)
g = nt.nodes.new("ShaderNodeGroup")
g.node_tree = chain
g.inputs["Hue"].default_value = 0.7
val = nt.nodes.new("ShaderNodeValue")
val.outputs[0].default_value = 0.5
nt.links.new(val.outputs[0], g.inputs["Value"])
nt.links.new(g.outputs["Color"], em.inputs["Color"])

outer = bpy.data.node_groups.new("Outer", "ShaderNodeTree")
outer.interface.new_socket(name="Value", in_out="INPUT", socket_type="NodeSocketFloat")
outer.interface.new_socket(name="Color", in_out="OUTPUT", socket_type="NodeSocketColor")
gi = outer.nodes.new("NodeGroupInput")
go = outer.nodes.new("NodeGroupOutput")
inner = outer.nodes.new("ShaderNodeGroup")
inner.node_tree = chain
inner.inputs["Hue"].default_value = 0.25
outer.links.new(gi.outputs["Value"], inner.inputs["Value"])
outer.links.new(inner.outputs["Color"], go.inputs["Color"])

nt, out = plane("Nested", 10)
em = emission(nt, out)
g = nt.nodes.new("ShaderNodeGroup")
g.node_tree = outer
g.inputs["Value"].default_value = 0.8
nt.links.new(g.outputs["Color"], em.inputs["Color"])

# --- structural --------------------------------------------------------
nt, out = plane("Displaced", 11)
pr = nt.nodes.new("ShaderNodeBsdfPrincipled")
nt.links.new(pr.outputs["BSDF"], out.inputs["Surface"])
disp = nt.nodes.new("ShaderNodeDisplacement")
disp.inputs["Scale"].default_value = 0.4
disp.inputs["Midlevel"].default_value = 0.5
noise = nt.nodes.new("ShaderNodeTexNoise")
noise.inputs["Scale"].default_value = 7.0
nt.links.new(by_id(noise.outputs, "Fac"), disp.inputs["Height"])
nt.links.new(disp.outputs["Displacement"], out.inputs["Displacement"])

for name, x, facing in (("FresnelMixed", 12, False), ("Facing", 13, True)):
    nt, out = plane(name, x)
    mix = nt.nodes.new("ShaderNodeMixShader")
    nt.links.new(mix.outputs["Shader"], out.inputs["Surface"])
    diff = nt.nodes.new("ShaderNodeBsdfDiffuse")
    diff.inputs["Color"].default_value = (0.8, 0.1, 0.1, 1.0)
    gloss = nt.nodes.new("ShaderNodeBsdfGlossy")
    nt.links.new(diff.outputs["BSDF"], mix.inputs[1])
    nt.links.new(gloss.outputs["BSDF"], mix.inputs[2])
    if facing:
        lw = nt.nodes.new("ShaderNodeLayerWeight")
        lw.inputs["Blend"].default_value = 0.3
        nt.links.new(lw.outputs["Facing"], by_id(mix.inputs, "Fac"))
    else:
        fr = nt.nodes.new("ShaderNodeFresnel")
        fr.inputs["IOR"].default_value = 1.8
        nt.links.new(by_id(fr.outputs, "Fac"), by_id(mix.inputs, "Fac"))

nt, out = plane("Remapped", 14)
pr = nt.nodes.new("ShaderNodeBsdfPrincipled")
nt.links.new(pr.outputs["BSDF"], out.inputs["Surface"])
ramp = nt.nodes.new("ShaderNodeValToRGB")
mr = map_range(nt, 0.0, to=(0.0, 0.5))
noise = nt.nodes.new("ShaderNodeTexNoise")
nt.links.new(by_id(noise.outputs, "Fac"), mr.inputs["Value"])
nt.links.new(mr.outputs["Result"], by_id(ramp.inputs, "Fac"))
nt.links.new(ramp.outputs["Color"], pr.inputs["Base Color"])

nt, out = plane("Tailed", 15)
pr = nt.nodes.new("ShaderNodeBsdfPrincipled")
nt.links.new(pr.outputs["BSDF"], out.inputs["Surface"])
mr = map_range(nt, 0.0, to=(0.25, 0.75), clamp=True)
noise = nt.nodes.new("ShaderNodeTexNoise")
nt.links.new(by_id(noise.outputs, "Fac"), mr.inputs["Value"])
nt.links.new(mr.outputs["Result"], pr.inputs["Base Color"])

nt, out = plane("VertexColored", 16)
pr = nt.nodes.new("ShaderNodeBsdfPrincipled")
nt.links.new(pr.outputs["BSDF"], out.inputs["Surface"])
vc = nt.nodes.new("ShaderNodeVertexColor")
vc.layer_name = "Col"
nt.links.new(vc.outputs["Color"], pr.inputs["Base Color"])

# A floor the way a procedural wood floor is built (the BASKETBALL-COURT
# scene's): planks from a banded Wave through a Color Ramp, mixed with the
# grain of a stretched Noise — two textures on one Base Color, which is
# what a single-texture slot cannot carry.
nt, out = plane("MixedWood", 18)
pr = nt.nodes.new("ShaderNodeBsdfPrincipled")
nt.links.new(pr.outputs["BSDF"], out.inputs["Surface"])
coord = nt.nodes.new("ShaderNodeTexCoord")
planks = nt.nodes.new("ShaderNodeTexWave")
planks.wave_type = "BANDS"
planks.bands_direction = "Y"
planks.inputs["Scale"].default_value = 7.0
planks.inputs["Distortion"].default_value = 0.4
grain = nt.nodes.new("ShaderNodeTexNoise")
grain.inputs["Scale"].default_value = 3.0
grain.inputs["Detail"].default_value = 8.0
stretch = nt.nodes.new("ShaderNodeMapping")
stretch.inputs["Scale"].default_value = (1.0, 22.0, 1.0)
ramp = nt.nodes.new("ShaderNodeValToRGB")
ramp.color_ramp.elements[0].position = 0.30
ramp.color_ramp.elements[0].color = (0.46, 0.28, 0.13, 1.0)
ramp.color_ramp.elements[1].position = 0.78
ramp.color_ramp.elements[1].color = (0.68, 0.47, 0.26, 1.0)
mix = nt.nodes.new("ShaderNodeMix")
mix.data_type = "RGBA"
mix.inputs["Factor"].default_value = 0.35
nt.links.new(coord.outputs["Object"], stretch.inputs["Vector"])
nt.links.new(stretch.outputs["Vector"], grain.inputs["Vector"])
nt.links.new(coord.outputs["Object"], planks.inputs["Vector"])
nt.links.new(by_id(planks.outputs, "Fac"), by_id(ramp.inputs, "Fac"))
nt.links.new(ramp.outputs["Color"], mix.inputs[6])
nt.links.new(grain.outputs["Color"], mix.inputs[7])
nt.links.new(mix.outputs[2], pr.inputs["Base Color"])

hashed = bpy.data.node_groups.new("Hash", "ShaderNodeTree")
hashed.interface.new_socket(name="Value", in_out="OUTPUT", socket_type="NodeSocketFloat")
go = hashed.nodes.new("NodeGroupOutput")
wn = hashed.nodes.new("ShaderNodeTexWhiteNoise")
wn.noise_dimensions = "3D"
hashed.links.new(wn.outputs["Value"], go.inputs["Value"])
nt, out = plane("WhiteNoised", 17)
pr = nt.nodes.new("ShaderNodeBsdfPrincipled")
nt.links.new(pr.outputs["BSDF"], out.inputs["Surface"])
g = nt.nodes.new("ShaderNodeGroup")
g.node_tree = hashed
nt.links.new(g.outputs["Value"], pr.inputs["Base Color"])

path = os.path.join(HERE, "NODES.blend")
bpy.ops.wm.save_as_mainfile(filepath=path, compress=True)
print("saved", path)
