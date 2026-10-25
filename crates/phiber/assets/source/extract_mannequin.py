"""Pull the demo's player out of the production mannequin: the female
mannequin body, the shirt and the shorts from its outfits, and a plain
armature of the Rigify rig's 71 deform bones — no controls, no
constraints, nothing an evaluator has to solve — into
`mannequin-player.blend`, which `make_demo.py` builds the teams
from. It lives here in `assets/source`, outside `assets/blend`, which the
browser build embeds whole.

    python extract_mannequin.py mannequin \\
        ../production/Mannequins-Female--Pose-05.blend mannequin-player.blend
    python extract_mannequin.py epic01 \\
        Female-Epic-01-One_Piece-posing.blend epic01-player.blend

Everything is taken at the rig's rest pose and in world space, so the
meshes and the bones agree wherever the source had them posed or placed.
The mannequin's pieces are smoothed by one applied level of Subdivision;
the character's keep their own topology (four times lighter) and carry a
`smooth` flag for the scene to subdivide them at load, before the Armature.
"""

import os
import sys

import bpy

PRESET, SRC, OUT = sys.argv[-3], sys.argv[-2], sys.argv[-1]
# What to take from each source: its generated Rigify rig, and each mesh
# under the name the demo knows it by. `body` is the one the kit is cut from.
PRESETS = {
    # Mannequins-Female--Pose-05: the dress-form mannequin and its outfit.
    "mannequin": ("rig.012", {"Female-Mannequin-Body-05.001": "Body", "Shirt-Base.001": "Jersey", "Shorts.003": "Shorts"}),
    # Female-Epic-01: the full character — body, face and hair.
    "epic01": ("rig.", {
        "Full-body ": "Body",  # sic: the name ends in a space
        "Eye.L": "Eye.L", "Eye.R": "Eye.R", "Eyebrows": "Eyebrows",
        "Eyelashes": "Eyelashes", "Nose": "Nose", "Hair-base": "Hair", "Pony-Tail": "Ponytail",
        # Not "Hair-framing-face.", "Face-Hair" or "Hair-Strands": the
        # posing file sweeps them back with its own face-hair rig, and at
        # rest they hang across her face.
    }),
}
RIG, PIECES = PRESETS[PRESET]
DROP = {".select_vert", ".select_edge", ".select_poly", "uv_seam", "sharp_edge"}
# Subdivision applied into the extracted meshes: the mannequin's one level
# is baked in; the character keeps its own topology and is smoothed at load.
APPLY = {"mannequin": 1, "epic01": 0}[PRESET]

try:
    bpy.ops.wm.open_mainfile(filepath=os.path.abspath(SRC))
except RuntimeError as why:  # the file's broken shape-key pointers are reported, not fatal
    print("open reported:", str(why).splitlines()[0])

rig = bpy.data.objects[RIG]
rig.data.pose_position = "REST"

# The deform bones in world space at rest, each parented to its nearest
# deforming ancestor. Rigify hangs the face's deform bones off `ORG-face`,
# whose chain (`ORG-spine.006` …) has no deform bone in it: an ORG- or MCH-
# ancestor stands for its DEF- twin, or the face would stay put while the
# head turns.
deforming = {b.name for b in rig.data.bones if b.use_deform}


def deform_parent(b):
    a = b.parent
    while a is not None:
        if a.use_deform:
            return a.name
        twin = "DEF-" + a.name.split("-", 1)[1] if a.name.startswith(("ORG-", "MCH-")) else None
        if twin in deforming and twin != b.name:
            return twin
        a = a.parent
    return None


bones = {}
m = rig.matrix_world
for b in rig.data.bones:
    if b.use_deform:
        bones[b.name] = (m @ b.head_local, m @ b.tail_local, (m.to_3x3() @ b.z_axis).normalized(), deform_parent(b))
# What is still a root above the neck (the ponytail's and the hair
# strands' chains hang off their own controls) rides with the head.
neck = bones["DEF-spine.004"][0].z
for name, (head, tail, z, parent) in list(bones.items()):
    if parent is None and name != "DEF-spine" and head.z > neck:
        bones[name] = (head, tail, z, "DEF-spine.006")
roots = [n for n, (*_, p) in bones.items() if p is None]
print("root deform bones:", roots)


def rest_mesh(name, role):
    """`name` as the rig at rest deforms it, one Subdivision level, in world
    space, its vertex groups kept and its materials dropped."""
    src = bpy.data.objects[name]
    smooth = any(mod.type == "SUBSURF" for mod in src.modifiers)
    for mod in src.modifiers:
        if mod.type == "SUBSURF":
            mod.levels = APPLY
            mod.show_viewport = APPLY > 0
        elif mod.type != "ARMATURE":
            mod.show_viewport = False
    if APPLY and not smooth:
        sub = src.modifiers.new("Smooth", "SUBSURF")
        sub.levels = APPLY
        src.modifiers.move(len(src.modifiers) - 1, 0)  # before the armature
    bpy.context.view_layer.update()
    depsgraph = bpy.context.evaluated_depsgraph_get()
    mesh = bpy.data.meshes.new_from_object(src.evaluated_get(depsgraph), preserve_all_data_layers=True, depsgraph=depsgraph)
    mesh.transform(src.matrix_world)
    # The body comes out with its faces wound inward; anything offset along
    # its normals (the vest) would go under the skin. Turn the winding out
    # when the trunk's faces mostly look inward.
    from mathutils import Vector
    trunk = [p for p in mesh.polygons if 0.9 < p.center.z < 1.5 and abs(p.center.x) < 0.25]
    inward = sum(1 for p in trunk if p.normal.dot(Vector((p.center.x, p.center.y, 0.0))) < 0)
    if trunk and inward > len(trunk) / 2:
        mesh.flip_normals()
        print(role, "normals turned outward")
    mesh.materials.clear()
    # Sculpting and selection state, colour layers and seams: none of it
    # draws, all of it weighs.
    for name in [a.name for a in mesh.attributes if a.name in DROP or a.name.startswith(".sculpt")]:
        mesh.attributes.remove(mesh.attributes[name])
    for layer in list(mesh.color_attributes):
        if layer.name in mesh.attributes:
            mesh.attributes.remove(mesh.attributes[layer.name])
    mesh.name = f"Player {role}"
    # The group names come with the mesh (they live on it since Blender 3.0).
    ob = bpy.data.objects.new(f"Player {role}", mesh)
    # Kept at its own resolution, the piece says whether its source was
    # smoothed, for the scene to put a Subdivision back ahead of the
    # Armature — evaluated once at load, as the production file has it.
    ob["smooth"] = int(smooth and not APPLY)
    for p in mesh.polygons:
        p.use_smooth = True
    print(role, len(mesh.vertices), "vertices")
    return ob


pieces = [rest_mesh(name, role) for name, role in PIECES.items()]

arm = bpy.data.armatures.new("Player Rig")
player = bpy.data.objects.new("Player Rig", arm)
scene = bpy.data.scenes.new("Player")
scene.collection.objects.link(player)
bpy.context.window.scene = scene if bpy.context.window else scene
bpy.context.view_layer.objects.active = player
bpy.ops.object.mode_set(mode="EDIT")
edit = {}
for name, (head, tail, z, _) in bones.items():
    e = arm.edit_bones.new(name)
    e.head, e.tail = head, tail
    e.align_roll(z)
    edit[name] = e
for name, (*_, parent) in bones.items():
    if parent:
        edit[name].parent = edit[parent]
bpy.ops.object.mode_set(mode="OBJECT")
for ob in pieces:
    scene.collection.objects.link(ob)
    ob.parent = player
    ob.modifiers.new("Armature", "ARMATURE").object = player

bpy.data.libraries.write(os.path.abspath(OUT), {scene, player, *pieces}, fake_user=True, compress=True)
print("wrote", OUT, os.path.getsize(OUT), "bytes,", len(bones), "bones")
