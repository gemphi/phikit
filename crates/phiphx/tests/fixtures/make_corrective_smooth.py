"""A bent tube under Corrective Smooth, once per setting the port tells
apart, and every vertex of each as Blender evaluates it — for
`tests/corrective_smooth_truth.rs`.

A two-bone armature ("root" 0→1 m up z, "tip" 1→2 m) bends a 0.15 m open
tube (8 sides, 13 rings, rigid weights split at z = 1) by 70° at the
joint, where a plain Armature pinches it. Every tube is its own object at
the origin with its own copy of the mesh:
- "Plain": Armature only — the pinch the modifier corrects.
- "Simple": Armature → Corrective Smooth at its defaults (Simple, factor
  0.5, 5 iterations, rest ORCO).
- "Length": Length Weight, factor 0.8, 12 iterations, scale 1.25.
- "Group": a "joint" group fading out from the knee, Pin Boundaries on.
- "Invert": the same group inverted.
- "Only": Only Smooth, 10 iterations.
- "Bind": rest BIND, bound with the tip bent −30° the other way, so the
  saved `bind_coords` are not the mesh.
- "First": Corrective Smooth (ORCO) before the Armature, the order
  `Hair.001` in the CW-POSES files has.

The pose is keyed at frames 1 and 2, so the import's skin path and the
saved pose agree.

Run: blender -b --python make_corrective_smooth.py -- corrective_smooth.blend corrective_smooth.tsv"""
import bpy, math, sys

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
arm = bpy.data.armatures.new("Arm")
rig = bpy.data.objects.new("Rig", arm)
sc.collection.objects.link(rig)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.mode_set(mode="EDIT")
root = arm.edit_bones.new("root"); root.head, root.tail = (0, 0, 0), (0, 0, 1)
tip = arm.edit_bones.new("tip"); tip.head, tip.tail = (0, 0, 1), (0, 0, 2); tip.parent = root
bpy.ops.object.mode_set(mode="OBJECT")
tip_pose = rig.pose.bones["tip"]
tip_pose.rotation_mode = "XYZ"

SIDES, RINGS = 8, 13


def tube(name):
    verts = [(0.15 * math.cos(2 * math.pi * k / SIDES), 0.15 * math.sin(2 * math.pi * k / SIDES), 2 * r / (RINGS - 1))
             for r in range(RINGS) for k in range(SIDES)]
    faces = [(r * SIDES + k, r * SIDES + (k + 1) % SIDES, (r + 1) * SIDES + (k + 1) % SIDES, (r + 1) * SIDES + k)
             for r in range(RINGS - 1) for k in range(SIDES)]
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, [], faces)
    ob = bpy.data.objects.new(name, me)
    sc.collection.objects.link(ob)
    root_g, tip_g = ob.vertex_groups.new(name="root"), ob.vertex_groups.new(name="tip")
    joint = ob.vertex_groups.new(name="joint")
    for v in me.vertices:
        z = v.co.z
        if z < 1 - 1e-6:
            root_g.add([v.index], 1.0, "REPLACE")
        elif z > 1 + 1e-6:
            tip_g.add([v.index], 1.0, "REPLACE")
        else:
            root_g.add([v.index], 0.5, "REPLACE")
            tip_g.add([v.index], 0.5, "REPLACE")
        joint.add([v.index], max(0.0, 1.0 - abs(z - 1.0) / 0.6), "REPLACE")
    return ob


def armature(ob):
    ob.modifiers.new("Armature", "ARMATURE").object = rig


def smooth(ob, **settings):
    md = ob.modifiers.new("CorrectiveSmooth", "CORRECTIVE_SMOOTH")
    for k, v in settings.items():
        setattr(md, k, v)
    return md


armature(tube("Plain"))
for name, settings in (
    ("Simple", {}),
    ("Length", {"smooth_type": "LENGTH_WEIGHTED", "factor": 0.8, "iterations": 12, "scale": 1.25}),
    ("Group", {"vertex_group": "joint", "use_pin_boundary": True}),
    ("Invert", {"vertex_group": "joint", "invert_vertex_group": True}),
    ("Only", {"use_only_smooth": True, "iterations": 10}),
):
    ob = tube(name)
    armature(ob)
    smooth(ob, **settings)
first = tube("First")
smooth(first)
armature(first)

bind = tube("Bind")
armature(bind)
smooth(bind, rest_source="BIND")
tip_pose.rotation_euler = (math.radians(-30), 0, 0)
bpy.context.view_layer.update()
with bpy.context.temp_override(object=bind, active_object=bind):
    bpy.ops.object.correctivesmooth_bind(modifier="CorrectiveSmooth")
assert bind.modifiers["CorrectiveSmooth"].is_bind, "bound"

tip_pose.rotation_euler = (math.radians(70), 0, 0)
for f in (1, 2):
    tip_pose.keyframe_insert("rotation_euler", frame=f)
sc.frame_set(1)

bpy.ops.wm.save_as_mainfile(filepath=out)
graph = bpy.context.evaluated_depsgraph_get()
with open(truth, "w") as f:
    for ob in sc.objects:
        if ob.type != "MESH":
            continue
        mesh = ob.evaluated_get(graph).to_mesh()
        f.write(f"{ob.name}\t{len(mesh.vertices)}\t" + " ".join(f"{c:.9g}" for v in mesh.vertices for c in v.co) + "\n")
