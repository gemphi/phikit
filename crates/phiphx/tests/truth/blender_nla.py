# Blender truth for `blend/anim/nla.rs`: every animated object's NLA — the
# active action and its extrapolation, then each track (last first, as the
# editor lists them) with its strips.
#
#   python3 blender_nla.py FILE.blend OUT.txt      (bpy module)
#
# Lines: `object <name> <action or -> <extrapolation> <tracks>` then
# `track <name> mute=<0/1> solo=<0/1>` and
# `strip <name> <type> <start> <end> <actstart> <actend> <repeat> <scale> <extrapolation> sel=<0/1> mute=<0/1> <action or ->`.
import bpy, sys

bpy.ops.wm.open_mainfile(filepath=sys.argv[1])
with open(sys.argv[2], "w") as f:
    for ob in bpy.data.objects:
        ad = ob.animation_data
        if not ad:
            continue
        f.write("object\t%s\t%s\t%s\t%d\n" % (ob.name, ad.action.name if ad.action else "-", ad.action_extrapolation, len(ad.nla_tracks)))
        for t in reversed(ad.nla_tracks):
            f.write("track\t%s\tmute=%d\tsolo=%d\n" % (t.name, t.mute, t.is_solo))
            for s in t.strips:
                f.write("strip\t%s\t%s\t%g\t%g\t%g\t%g\t%g\t%g\t%s\tsel=%d\tmute=%d\t%s\n" % (
                    s.name, s.type, s.frame_start, s.frame_end, s.action_frame_start, s.action_frame_end,
                    s.repeat, s.scale, s.extrapolation, s.select, s.mute, s.action.name if s.action else "-"))
