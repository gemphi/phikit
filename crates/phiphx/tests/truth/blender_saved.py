# Blender truth for `tests/blender_opens_saves.rs`: open every `.blend` in
# DIR (the files phiphx saved) and write what Blender reads of the edits,
# in the test's line format.
#
#   python3 blender_saved.py DIR OUT.txt        (bpy as a module)
#   blender -b --python blender_saved.py -- DIR OUT.txt
import bpy, os, sys

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
folder, out = args[0], args[1]
lines = ["# Blender %s reading phiphx's saves (blender_saved.py)" % bpy.app.version_string]
for name in sorted(os.listdir(folder)):
    if not name.endswith(".blend"):
        continue
    case = name[:-6]
    bpy.ops.wm.open_mainfile(filepath=os.path.join(folder, name), load_ui=False)
    scene = bpy.context.scene
    layer = bpy.context.view_layer
    lines.append("%s frame %d %d %d" % (case, scene.frame_current, scene.frame_start, scene.frame_end))
    active = layer.objects.active
    lines.append("%s active %s" % (case, active.name if active else ""))
    for ob in sorted(scene.objects, key=lambda o: o.name):
        mesh = " %dv %df" % (len(ob.data.vertices), len(ob.data.polygons)) if ob.type == "MESH" else ""
        ad = ob.animation_data
        playing = ad and any(not t.mute and len(t.strips) for t in ad.nla_tracks)
        if ad and (ad.action or playing):
            at = "animated"
        else:
            import math
            rot = tuple(math.degrees(a) + 0.0 for a in ob.rotation_euler)
            # A child of an animated object goes where its parent's action takes it.
            up, carried = ob.parent, False
            while up:
                carried = carried or bool(up.animation_data and up.animation_data.action)
                up = up.parent
            world = "animated" if carried else "%.3f %.3f %.3f" % tuple(ob.matrix_world.translation)
            at = "%.3f %.3f %.3f rot %.1f %.1f %.1f scale %.3f %.3f %.3f world %s" % (
                tuple(ob.location) + rot + tuple(ob.scale) + (world,))
        parent = ob.parent.name if ob.parent else "-"
        mats = ",".join(s.material.name for s in ob.material_slots if s.material) or "-"
        lines.append("%s object %s %s%s at %s parent %s mats %s render %d viewport %d hidden %d selected %d" % (
            case, ob.name, ob.type, mesh, at, parent, mats, int(not ob.hide_render), int(not ob.hide_viewport),
            int(ob.hide_get()), int(ob.select_get())))
    def walk(lc, parent):
        c = lc.collection
        path = c.name if not parent else parent + "/" + c.name
        names = ",".join(sorted(o.name for o in c.objects))
        lines.append("%s collection %s [%s] exclude %d hide %d render %d" % (
            case, path, names, int(lc.exclude), int(lc.hide_viewport), int(not c.hide_render)))
        for k in lc.children:
            walk(k, path)
    walk(layer.layer_collection, "")
    for a in sorted(bpy.data.actions, key=lambda a: a.name):
        rng = "%.2f-%.2f" % (a.frame_start, a.frame_end) if a.use_frame_range else "-"
        lines.append("%s action %s slots [%s] range %s cyclic %d" % (
            case, a.name, ",".join(s.identifier for s in a.slots), rng, int(a.use_cyclic)))
    for ob in sorted(scene.objects, key=lambda o: o.name):
        ad = ob.animation_data
        if ad and ad.action:
            lines.append("%s anim %s %s %s" % (case, ob.name, ad.action.name, ad.action_slot.identifier if ad.action_slot else "-"))
    # Where the keys are inserted: every curve's keys and handles, and
    # whether they are the handles Blender computes for those keys.
    if case == "actions":
        close = lambda a, b: all(abs(x - y) < 1e-4 for x, y in zip(a, b))
        for a in sorted(bpy.data.actions, key=lambda a: a.name):
            for layer in a.layers:
                for strip in layer.strips:
                    for s in a.slots:
                        bag = strip.channelbag(s)
                        for fc in (bag.fcurves if bag else []):
                            pts = fc.keyframe_points
                            keys = " ".join("%.3f,%.3f" % tuple(p.co) for p in pts)
                            stored = [tuple(p.handle_left) + tuple(p.handle_right) for p in pts]
                            handles = " ".join("%.3f,%.3f/%.3f,%.3f" % h for h in stored)
                            fc.update()
                            again = [tuple(p.handle_left) + tuple(p.handle_right) for p in pts]
                            verdict = "blenders" if all(close(x, y) for x, y in zip(stored, again)) else "not-blenders"
                            lines.append("%s curve %s %s %s[%d] keys %s handles %s %s" % (
                                case, a.name, s.identifier, fc.data_path, fc.array_index, keys, handles, verdict))
    # The NLA as laid, and where it moves its object.
    if case in ("actions", "nla_cube"):
        extend = {"HOLD": 0, "HOLD_FORWARD": 1, "NOTHING": 2}
        blend = {"REPLACE": 0, "ADD": 1, "SUBTRACT": 2, "MULTIPLY": 3, "COMBINE": 4}
        for ob in sorted(scene.objects, key=lambda o: o.name):
            ad = ob.animation_data
            if not ad or not len(ad.nla_tracks):
                continue
            for t in ad.nla_tracks:
                strips = " ".join("%s:%s:%s:%.2f-%.2f:%.2f-%.2f:sync%d:cyclic%d:sel%d:act%d:mute%d:extend%d:blend%d:influence%.2f" % (
                    st.name, st.action.name if st.action else "", st.action_slot.identifier if st.action_slot else "",
                    st.frame_start, st.frame_end, st.action_frame_start, st.action_frame_end,
                    int(st.use_sync_length), int(st.use_animated_time_cyclic), int(st.select), int(st.active), int(st.mute),
                    extend[st.extrapolation], blend[st.blend_type], st.influence) for st in t.strips)
                lines.append("%s nla %s track %s mute %d lock %d sel %d act %d [%s]" % (
                    case, ob.name, t.name, int(t.mute), int(t.lock), int(t.select), int(t.active), strips))
            # An object disabled in viewports is not evaluated.
            if ob.hide_viewport:
                continue
            for frame in (1, 5, 12, 24, 30, 42):
                scene.frame_set(frame)
                mw = ob.matrix_world
                lines.append("%s at %s %d %.3f %.3f %.3f y %.3f %.3f %.3f" % ((case, ob.name, frame) + tuple(mw.translation) + tuple(mw.col[1][:3])))
        scene.frame_set(42)
    for m in sorted(scene.timeline_markers, key=lambda m: (m.frame, m.name)):
        lines.append("%s marker %d %s" % (case, m.frame, m.name))
    ts = scene.tool_settings
    lines.append("%s playback %s scrub %d lock %d subframe %d autokey %d" % (
        case, scene.sync_mode, int(scene.use_audio_scrub), int(scene.lock_frame_selection_to_range),
        int(scene.show_subframe), int(ts.use_keyframe_insert_auto)))
    r, c, im = scene.render, scene.cycles, scene.render.image_settings
    lines.append("%s render %s film %d samples %d preview %d bounces %d denoise %d %s filter %s path %s format %s depth %s mode %s" % (
        case, r.engine, int(r.film_transparent), c.samples, c.preview_samples, c.diffuse_bounces, int(c.use_denoising),
        c.denoiser, c.pixel_filter_type,
        r.filepath, im.file_format, im.color_depth, im.color_mode))
with open(out, "w") as f:
    f.write("\n".join(lines) + "\n")
