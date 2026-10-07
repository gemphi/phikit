"""Blender's own compositor over a known input, one node at a time.

Each case builds a fresh scene whose tree is

    Image (a 4x4 PNG of known pixels) -> <the node under test> -> Composite

renders it, and records the output pixels. The input image is written once
as a 32-bit float EXR so the values go in exactly and come back exactly —
a PNG would quantise them and the comparison would measure the codec.

Writes `composite.txt`: one line a case,

    <case> <r> <g> <b> <a>   (per pixel, 16 numbers for the 4x4)

and `COMPOSITE.blend`, a file carrying one chained tree for the reader to
read (Image -> Brightness/Contrast -> Hue Saturation Value -> Mix -> Composite).

Run: `python3 -I truth/blender_composite.py`
"""

import os
import sys

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "composite.txt")
FIXTURE = os.path.join(os.path.dirname(HERE), "fixtures", "COMPOSITE.blend")
TMP = os.path.join(HERE, "_comp_tmp")
os.makedirs(TMP, exist_ok=True)

W = H = 4
# Sixteen pixels spanning the interesting ground: black, white, over-white,
# a negative channel, saturated primaries, greys, and a range of alphas.
INPUT = [
    (0.0, 0.0, 0.0, 1.0), (1.0, 1.0, 1.0, 1.0), (0.5, 0.5, 0.5, 1.0), (0.25, 0.5, 0.75, 1.0),
    (1.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.0, 1.0), (0.0, 0.0, 1.0, 1.0), (0.8, 0.6, 0.2, 1.0),
    (2.0, 1.5, 0.1, 1.0), (0.05, 0.02, 0.01, 1.0), (0.9, 0.9, 0.1, 0.5), (0.3, 0.3, 0.3, 0.25),
    (-0.1, 0.4, 0.2, 1.0), (0.7, 0.1, 0.9, 0.75), (0.15, 0.85, 0.45, 1.0), (0.6, 0.6, 0.6, 0.0),
]


def write_input():
    """The input as a float EXR, exact."""
    img = bpy.data.images.new("CompInput", width=W, height=H, float_buffer=True, alpha=True)
    flat = []
    # Blender's rows run bottom-up; keep INPUT as row 0 first and flip here.
    for row in range(H):
        src = H - 1 - row
        for col in range(W):
            flat.extend(INPUT[src * W + col])
    img.pixels.foreach_set(flat)
    path = os.path.join(TMP, "input.exr")
    img.file_format = "OPEN_EXR"
    img.filepath_raw = path
    img.save()
    return path


def fresh(path):
    """A scene set up to render 4x4 with the compositor on and nothing else."""
    scene = bpy.context.scene
    scene.render.resolution_x = W
    scene.render.resolution_y = H
    scene.render.resolution_percentage = 100
    scene.render.use_compositing = True
    scene.render.use_sequencer = False
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = "OPEN_EXR"
    scene.render.image_settings.color_depth = "32"
    scene.render.image_settings.color_mode = "RGBA"
    scene.view_settings.view_transform = "Standard"
    scene.view_settings.look = "None"
    scene.render.engine = "BLENDER_WORKBENCH"
    tree = tree_of(scene)
    for node in list(tree.nodes):
        tree.nodes.remove(node)
    image = tree.nodes.new("CompositorNodeImage")
    image.image = bpy.data.images.load(path, check_existing=True)
    # 5.0 dropped the Composite node: the scene points at a node GROUP and
    # the group's own output is the result. Older Blenders have no group and
    # end the tree at a Composite node.
    try:
        composite = tree.nodes.new("CompositorNodeComposite")
    except RuntimeError:
        composite = tree.nodes.new("NodeGroupOutput")
    return scene, tree, image, composite


def tree_of(scene):
    """The scene's compositor tree, whichever Blender this is."""
    if hasattr(scene, "compositing_node_group"):
        if scene.compositing_node_group is None:
            group = bpy.data.node_groups.new("Compositing Nodes", "CompositorNodeTree")
            group.interface.new_socket("Image", in_out="INPUT", socket_type="NodeSocketColor")
            group.interface.new_socket("Image", in_out="OUTPUT", socket_type="NodeSocketColor")
            scene.compositing_node_group = group
        return scene.compositing_node_group
    scene.use_nodes = True
    return scene.node_tree


def rendered(scene, name):
    """Render and read the output EXR back, row 0 first."""
    path = os.path.join(TMP, f"{name}.exr")
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(path, check_existing=False)
    flat = [0.0] * (W * H * 4)
    img.pixels.foreach_get(flat)
    out = []
    for row in range(H):
        src = H - 1 - row
        for col in range(W):
            at = (src * W + col) * 4
            out.extend(flat[at : at + 4])
    bpy.data.images.remove(img)
    return out


def case(lines, path, name, build):
    """One case: build the middle of the chain, render, record."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene, tree, image, composite = fresh(path)
    tail = build(tree, image)
    tree.links.new(tail, composite.inputs["Image"])
    out = rendered(scene, name)
    lines.append(name + " " + " ".join(f"{v:.7g}" for v in out))
    print(name, "ok", flush=True)


def new_node(tree, *idnames):
    """The first node type that exists under any of `idnames`.

    5.x renamed several compositor nodes to the shared shader spelling —
    `CompositorNodeGamma` is `ShaderNodeGamma` now, `CompositorNodeMixRGB`
    is `ShaderNodeMix` — and dropped the Composite node for a group output.
    """
    for name in idnames:
        try:
            return tree.nodes.new(name)
        except RuntimeError:
            continue
    raise RuntimeError(f"none of {idnames}")


def factor(node):
    """Mix's blend factor, under either of its two names."""
    return [s for s in node.inputs if s.name in ("Fac", "Factor") and s.enabled][0]


def color_ab(node):
    """Mix's two colour inputs: `ShaderNodeMix` has an A/B pair per type."""
    return [s for s in node.inputs if s.type == "RGBA" and s.enabled][:2]


def color_out(node):
    return [s for s in node.outputs if s.type == "RGBA"][0]


def socket_in(node, *names):
    """The first input socket that exists under any of `names`."""
    for n in names:
        if n in node.inputs:
            return node.inputs[n]
    raise KeyError(f"{node.bl_idname}: none of {names}")


def main():
    path = write_input()
    lines = []

    def passthrough(tree, image):
        return image.outputs["Image"]

    case(lines, path, "passthrough", passthrough)

    def bright(tree, image):
        n = tree.nodes.new("CompositorNodeBrightContrast")
        socket_in(n, "Bright", "Brightness").default_value = 15.0
        socket_in(n, "Contrast").default_value = 40.0
        tree.links.new(image.outputs["Image"], socket_in(n, "Image", "Color"))
        return n.outputs[0]

    case(lines, path, "bright_contrast", bright)

    def bright_down(tree, image):
        n = tree.nodes.new("CompositorNodeBrightContrast")
        socket_in(n, "Bright", "Brightness").default_value = -10.0
        socket_in(n, "Contrast").default_value = -60.0
        tree.links.new(image.outputs["Image"], socket_in(n, "Image", "Color"))
        return n.outputs[0]

    case(lines, path, "bright_contrast_down", bright_down)

    def exposure(tree, image):
        n = tree.nodes.new("CompositorNodeExposure")
        socket_in(n, "Exposure").default_value = -1.5
        tree.links.new(image.outputs["Image"], socket_in(n, "Image", "Color"))
        return n.outputs[0]

    case(lines, path, "exposure", exposure)

    def gamma(tree, image):
        n = new_node(tree, "CompositorNodeGamma", "ShaderNodeGamma")
        socket_in(n, "Gamma").default_value = 2.2
        tree.links.new(image.outputs["Image"], socket_in(n, "Image", "Color"))
        return n.outputs[0]

    case(lines, path, "gamma", gamma)

    def invert(tree, image):
        n = tree.nodes.new("CompositorNodeInvert")
        socket_in(n, "Fac", "Factor").default_value = 0.75
        tree.links.new(image.outputs["Image"], socket_in(n, "Color", "Image"))
        return n.outputs[0]

    case(lines, path, "invert", invert)

    def huesat(tree, image):
        n = tree.nodes.new("CompositorNodeHueSat")
        socket_in(n, "Hue").default_value = 0.65
        socket_in(n, "Saturation").default_value = 1.4
        socket_in(n, "Value").default_value = 0.8
        socket_in(n, "Fac", "Factor").default_value = 1.0
        tree.links.new(image.outputs["Image"], socket_in(n, "Image", "Color"))
        return n.outputs[0]

    case(lines, path, "hue_sat", huesat)

    def posterize(tree, image):
        n = tree.nodes.new("CompositorNodePosterize")
        socket_in(n, "Steps").default_value = 5.0
        tree.links.new(image.outputs["Image"], socket_in(n, "Image", "Color"))
        return n.outputs[0]

    case(lines, path, "posterize", posterize)

    def rgb_to_bw(tree, image):
        n = tree.nodes.new("CompositorNodeRGBToBW")
        tree.links.new(image.outputs["Image"], socket_in(n, "Image", "Color"))
        return n.outputs[0]

    case(lines, path, "rgb_to_bw", rgb_to_bw)

    def set_alpha(tree, image):
        n = tree.nodes.new("CompositorNodeSetAlpha")
        socket_in(n, "Alpha").default_value = 0.4
        tree.links.new(image.outputs["Image"], socket_in(n, "Image", "Color"))
        return n.outputs[0]

    case(lines, path, "set_alpha_apply", set_alpha)

    def premul(tree, image):
        n = tree.nodes.new("CompositorNodePremulKey")
        tree.links.new(image.outputs["Image"], socket_in(n, "Image", "Color"))
        return n.outputs[0]

    case(lines, path, "premultiply", premul)

    def mixed(mode, fac):
        def build(tree, image):
            n = new_node(tree, "CompositorNodeMixRGB", "ShaderNodeMix")
            # `ShaderNodeMix` carries one A/B pair per data type and
            # switches which are available; the colour pair is the one a
            # compositor Mix used to be.
            if hasattr(n, "data_type"):
                n.data_type = "RGBA"
            n.blend_type = mode
            factor(n).default_value = fac
            a, b = color_ab(n)
            tree.links.new(image.outputs["Image"], a)
            b.default_value = (0.3, 0.7, 0.45, 1.0)
            return color_out(n)

        return build

    for mode in [
        "MIX", "ADD", "MULTIPLY", "SUBTRACT", "SCREEN", "DIVIDE", "DIFFERENCE",
        "DARKEN", "LIGHTEN", "OVERLAY", "DODGE", "BURN", "HUE", "SATURATION",
        "VALUE", "COLOR", "SOFT_LIGHT", "LINEAR_LIGHT", "EXCLUSION",
    ]:
        case(lines, path, f"mix_{mode.lower()}", mixed(mode, 0.65))

    def over(tree, image):
        n = tree.nodes.new("CompositorNodeAlphaOver")
        factor(n).default_value = 1.0
        back, front = color_ab(n)
        tree.links.new(image.outputs["Image"], back)
        front.default_value = (0.2, 0.9, 0.4, 0.6)
        return color_out(n)

    case(lines, path, "alpha_over", over)

    # The fixture's own chain, rendered: Image -> Brightness/Contrast ->
    # Hue Saturation Value -> Mix(Screen) -> output. The reader reads this
    # same tree out of `COMPOSITE.blend` and has to come to these pixels.
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene, tree, image, composite = fresh(path)
    chain(tree, image, composite)
    lines.append("chain " + " ".join(f"{v:.7g}" for v in rendered(scene, "chain")))
    print("chain ok", flush=True)

    with open(OUT, "w", encoding="utf-8") as fh:
        fh.write("# input (row 0 first, RGBA)\n")
        fh.write("input " + " ".join(f"{v:.7g}" for p in INPUT for v in p) + "\n")
        for line in lines:
            fh.write(line + "\n")
    print("wrote", OUT, len(lines), "cases")

    # And the fixture: the same chain, saved.
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene, tree, image, composite = fresh(path)
    chain(tree, image, composite)
    bpy.ops.wm.save_as_mainfile(filepath=FIXTURE)
    print("wrote", FIXTURE)


def chain(tree, image, composite):
    """Image -> Brightness/Contrast -> Hue Saturation Value -> Mix -> out."""
    bc = tree.nodes.new("CompositorNodeBrightContrast")
    socket_in(bc, "Bright", "Brightness").default_value = 12.0
    socket_in(bc, "Contrast").default_value = 25.0
    hs = tree.nodes.new("CompositorNodeHueSat")
    socket_in(hs, "Hue").default_value = 0.55
    socket_in(hs, "Saturation").default_value = 1.2
    mx = new_node(tree, "CompositorNodeMixRGB", "ShaderNodeMix")
    if hasattr(mx, "data_type"):
        mx.data_type = "RGBA"
    mx.blend_type = "SCREEN"
    factor(mx).default_value = 0.4
    mix_a, mix_b = color_ab(mx)
    mix_b.default_value = (0.1, 0.2, 0.35, 1.0)
    tree.links.new(image.outputs["Image"], socket_in(bc, "Image", "Color"))
    tree.links.new(bc.outputs[0], socket_in(hs, "Image", "Color"))
    tree.links.new(hs.outputs[0], mix_a)
    tree.links.new(color_out(mx), composite.inputs["Image"])
    for node, (x, y) in zip([image, bc, hs, mx, composite], [(-600, 0), (-380, 0), (-160, 0), (60, 0), (300, 0)]):
        node.location = (x, y)


if __name__ == "__main__":
    sys.exit(main())
