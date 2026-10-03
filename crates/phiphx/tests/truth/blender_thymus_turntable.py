# Makes `truth/thymus_turntable.txt` (Blender 5.0): THYMUS's turntable
# camera `turn_cam_02` — parented to the `Path` Bézier circle, whose
# evaluation time a Generator drives, and aimed by Track To — its world
# matrix at five frames. For `blend/curve/tests.rs`. Run with the bpy module:
#
#   python3 blender_thymus_turntable.py ../fixtures/THYMUS-BOLDROID-WAVE-003.blend thymus_turntable.txt
import bpy, sys

blend, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.open_mainfile(filepath=blend)
scene = bpy.context.scene
with open(truth, "w") as f:
    for frame in (1, 37, 60, 125, 140):
        scene.frame_set(frame)
        graph = bpy.context.evaluated_depsgraph_get()
        m = scene.objects["turn_cam_02"].evaluated_get(graph).matrix_world
        f.write(f"{frame}\tOB\tturn_cam_02\t" + " ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4)) + "\n")
