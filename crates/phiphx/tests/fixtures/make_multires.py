"""Meshes under a Multiresolution modifier with displacement sculpted in, and
every vertex of each as Blender evaluates it — for `tests/multires_truth.rs`
(`modifier/multires.rs`).

Each object is subdivided by the modifier itself
(`multires_subdivide`, Catmull-Clark) and then reshaped
(`multires_reshape`) onto a copy of its own top level pushed about by a
smooth vector field — so its `MDisps` grids hold displacements with
tangential parts as well as normal ones, as a sculpt leaves them.

- "Cube": a 2 m cube, three levels, viewport at 3 — eight valence-3
  extraordinary corners (OpenSubdiv's Gregory end caps).
- "Cube Low" and "Cube One": two more objects on the same mesh at viewport
  levels 2 and 1, below its totlvl of 3, so the grids are read at a stride.
- "Prism": a hexagonal prism, its caps six-sided faces (PTEX face per
  corner), two levels.
- "Sheet": an open 3 × 3 grid of quads, two levels — boundary vertices and
  smooth one-face corners. "Sheet Keep" the same with Keep Corners.

Run: blender -b --python make_multires.py -- multires.blend multires.txt
"""
import bpy, math, sys

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene


def field(co, k):
    x, y, z = co
    return (
        x + k * math.sin(2.1 * y + 0.3) * math.cos(1.3 * z),
        y + k * math.sin(1.7 * z + 0.9) * math.cos(1.1 * x),
        z + k * math.sin(1.9 * x + 0.5) * math.cos(1.6 * y),
    )


def build(name, verts, faces, levels, k, boundary="ALL"):
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, [], faces)
    ob = bpy.data.objects.new(name, me)
    sc.collection.objects.link(ob)
    md = ob.modifiers.new("Multires", "MULTIRES")
    md.boundary_smooth = boundary
    with bpy.context.temp_override(object=ob, active_object=ob):
        for _ in range(levels):
            bpy.ops.object.multires_subdivide(modifier="Multires", mode="CATMULL_CLARK")
    # The top level as Blender evaluates it, pushed by the field.
    graph = bpy.context.evaluated_depsgraph_get()
    top = bpy.data.meshes.new_from_object(ob.evaluated_get(graph))
    for v in top.vertices:
        v.co = field(v.co, k)
    target = bpy.data.objects.new(name + " target", top)
    sc.collection.objects.link(target)
    with bpy.context.temp_override(object=ob, active_object=ob, selected_objects=[ob, target],
                                   selected_editable_objects=[ob, target]):
        bpy.ops.object.multires_reshape(modifier="Multires")
    bpy.data.objects.remove(target)
    bpy.data.meshes.remove(top)
    return ob


CUBE = [(-1, -1, -1), (1, -1, -1), (1, 1, -1), (-1, 1, -1), (-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1)]
CUBE_FACES = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (2, 3, 7, 6), (0, 4, 7, 3), (1, 2, 6, 5)]
cube = build("Cube", CUBE, CUBE_FACES, 3, 0.15)
for name, lvl in (("Cube Low", 2), ("Cube One", 1)):
    twin = cube.copy()
    twin.name = name
    sc.collection.objects.link(twin)
    twin.modifiers["Multires"].levels = lvl

ring = [(math.cos(a * math.pi / 3), math.sin(a * math.pi / 3)) for a in range(6)]
prism_verts = [(x + 4, y, -1) for x, y in ring] + [(x + 4, y, 1) for x, y in ring]
prism_faces = [tuple(range(5, -1, -1)), tuple(range(6, 12))] + [(i, (i + 1) % 6, (i + 1) % 6 + 6, i + 6) for i in range(6)]
build("Prism", prism_verts, prism_faces, 2, 0.12)

grid = [(x - 1.5, y - 1.5 + 4, 0.2 * math.sin(x + y)) for y in range(4) for x in range(4)]
grid_faces = [(y * 4 + x, y * 4 + x + 1, y * 4 + x + 5, y * 4 + x + 4) for y in range(3) for x in range(3)]
build("Sheet", grid, grid_faces, 2, 0.12)
build("Sheet Keep", [(x + 5, y, z) for x, y, z in grid], grid_faces, 2, 0.12, boundary="PRESERVE_CORNERS")

bpy.ops.wm.save_as_mainfile(filepath=out)
graph = bpy.context.evaluated_depsgraph_get()
with open(truth, "w") as f:
    for ob in sorted(sc.objects, key=lambda o: o.name):
        md = ob.modifiers["Multires"]
        mesh = ob.evaluated_get(graph).to_mesh()
        f.write(f"{ob.name}\t{md.levels}/{md.total_levels}\t{len(mesh.vertices)}\t{len(mesh.polygons)}\t"
                + " ".join(f"{c:.6f}" for v in mesh.vertices for c in v.co) + "\n")
