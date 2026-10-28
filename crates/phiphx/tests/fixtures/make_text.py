"""Text objects in Blender's builtin font (`bfont.pfb`), for the text-to-mesh
path (`blend/text`, `tests/text_truth.rs`):

- "Text Extruded": two lines ("Phi Text" / "Line 2"), centred, extruded 0.05
  with a 0.01 round bevel of resolution 2 and fill mode BOTH: the outlines
  swept by the bevel profile, front and back caps filled.
- "Text Flat": two lines ("Hello," / "Wörld 3!" with a tab), left aligned,
  size 0.8, character spacing 1.2, word spacing 1.5, line spacing 1.3,
  shear 0.2, offset (0.1, -0.05), resolution 6, under a Solidify of 0.05 —
  the flat fill then a modifier stack, as `Text.004` / `Text.005` of the
  CW-POSES files have.
- "Text Underlined": "Under line" with its first five characters
  underlined and small caps on the second word, aligned right and to the
  top: underline splines and small-caps scaling.

Run: blender -b --python make_text.py -- text.blend"""
import bpy, sys

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene


def text(name, body, loc):
    cu = bpy.data.curves.new(name, "FONT")
    cu.body = body
    ob = bpy.data.objects.new(name, cu)
    ob.location = loc
    sc.collection.objects.link(ob)
    return ob


ex = text("Text Extruded", "Phi Text\nLine 2", (0, 0, 0))
ex.data.align_x = "CENTER"
ex.data.extrude = 0.05
ex.data.bevel_depth = 0.01
ex.data.bevel_resolution = 2
ex.data.fill_mode = "BOTH"

flat = text("Text Flat", "Hello,\nWörld\t3!", (0, -3, 0))
cu = flat.data
cu.size = 0.8
cu.space_character = 1.2
cu.space_word = 1.5
cu.space_line = 1.3
cu.shear = 0.2
cu.offset_x = 0.1
cu.offset_y = -0.05
cu.resolution_u = 6
mod = flat.modifiers.new("Solidify", "SOLIDIFY")
mod.thickness = 0.05

ul = text("Text Underlined", "Under line", (0, -6, 0))
ul.data.align_x = "RIGHT"
ul.data.align_y = "TOP"
for i, c in enumerate(ul.data.body_format):
    c.use_underline = i < 5
    c.use_small_caps = i >= 6

bpy.ops.wm.save_as_mainfile(filepath=sys.argv[sys.argv.index("--") + 1])
