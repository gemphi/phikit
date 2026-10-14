"""How long Blender itself takes to open a `.blend`, and to evaluate it.

`docs/frame-cost.md` has phiber's open times and no Blender column, so the
project's claim -- the same scene, read faster -- has never actually been
measured against the thing it claims to beat. This is that column.

The comparison is only fair if both sides are asked for the same work, and
they are not the same by default:

* **Read** is `wm.open_mainfile`: the file parsed and its datablocks built.
  Blender stops there. Nothing is evaluated, no modifier has run, no mesh
  has been subdivided, and `object.data` is still the *pre-modifier* mesh.
* **Evaluate** is what `Phx::open` also does: the depsgraph updated and
  every mesh object's evaluated geometry realised, which is what runs the
  modifier stack and gives the vertices a viewport would draw. Blender does
  this lazily, per object, when something asks -- so it is asked here, for
  every object, which is the state phiber's model is in when `open`
  returns.

`to_mesh()` is the cheapest way to force it that does not also allocate a
new mesh datablock (`evaluated_get(dg).to_mesh()` hands back the evaluated
mesh itself, owned by the object). Its vertex total is printed so the two
sides can be checked for having read the same scene -- a faster reader that
read less is not faster.

**Runs matter, on both sides.** The first open in a fresh process pays for
warm-up that has nothing to do with the file -- allocator arenas, thread
pools, first-touch page faults. Blender pays it on its first file and not
on the rest, which is why `-n` repeats each file and this reports the
**min**: the fairest figure to put beside the Rust side's, which repeats for
the same reason (`examples/bench_open.rs`).

Writes one line per file, tab separated:

    file, bytes, objects, vertices, min read ms, min evaluate ms, min total ms

Run (Blender 5.x, or the `bpy` module at the same version):

    blender --background --factory-startup --python blender_open_time.py \
        -- [-n RUNS] FILE...
"""

import os
import sys
import time

import bpy

# The timer: `perf_counter` rather than `process_time`, because the thing
# being compared is how long the user waits, and both sides thread.
clock = time.perf_counter


def once(path):
    """Open `path`, evaluate it, and report what each half cost."""
    started = clock()
    bpy.ops.wm.open_mainfile(filepath=path)
    read = clock() - started

    started = clock()
    scene = bpy.context.scene
    graph = bpy.context.evaluated_depsgraph_get()
    graph.update()
    vertices = 0
    objects = 0
    for obj in scene.objects:
        objects += 1
        if obj.type not in ("MESH", "CURVE", "SURFACE", "FONT", "META"):
            continue
        evaluated = obj.evaluated_get(graph)
        mesh = evaluated.to_mesh()
        if mesh is not None:
            vertices += len(mesh.vertices)
        evaluated.to_mesh_clear()
    evaluate = clock() - started

    return objects, vertices, read, evaluate


def main():
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    runs = 1
    path_list = []
    while argv:
        word = argv.pop(0)
        if word == "-n" and argv:
            runs = max(1, int(argv.pop(0)))
        else:
            path_list.append(word)
    if not path_list:
        print("# no files given", file=sys.stderr)
        return
    print(f"# file\tbytes\tobjects\tvertices\tread ms\tevaluate ms\ttotal ms  ({runs} run(s) each, min)")
    for path in path_list:
        if not os.path.exists(path):
            print(f"# {path}: no such file", file=sys.stderr)
            continue
        size = os.path.getsize(path)
        best = None
        for _ in range(runs):
            try:
                objects, vertices, read, evaluate = once(path)
            except (RuntimeError, SystemError) as why:
                print(f"# {path}: {type(why).__name__}: {why}", file=sys.stderr)
                best = None
                break
            if best is None or read + evaluate < best[2] + best[3]:
                best = (objects, vertices, read, evaluate)
        if best is None:
            continue
        objects, vertices, read, evaluate = best
        print(
            f"{os.path.basename(path)}\t{size}\t{objects}\t{vertices}"
            f"\t{read * 1000:.1f}\t{evaluate * 1000:.1f}\t{(read + evaluate) * 1000:.1f}"
        )
        sys.stdout.flush()


main()
