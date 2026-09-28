# Blender truth for `space/*/set.rs`: every factory workspace's areas — type,
# mode, position and size in the 1920 × 1080 startup window, the regions each
# shows — so phiber's space specs are checked against `startup.blend` rather
# than remembered.
#
#   python3 blender_workspaces.py OUT.txt        (bpy module)
#
# One line per area: workspace, area type, mode, x, y, w, h, then the visible
# regions as type:WxH, then the 3D viewport's shading and perspective or the
# Image editor's image.
import bpy, sys

out = sys.argv[1]
bpy.ops.wm.read_homefile(app_template="")
with open(out, "w") as f:
    for ws in bpy.data.workspaces:
        screen = ws.screens[0]
        for a in sorted(screen.areas, key=lambda a: (-a.y, a.x)):
            sp = a.spaces.active
            mode = getattr(sp, "mode", "") or ""
            if a.type == "DOPESHEET_EDITOR":
                mode = sp.mode
            regions = " ".join("%s:%dx%d" % (r.type, r.width, r.height) for r in a.regions
                               if r.width > 1 and r.height > 1 and r.type != "WINDOW")
            extra = ""
            if a.type == "VIEW_3D":
                extra = "%s %s" % (sp.shading.type, sp.region_3d.view_perspective)
            elif a.type == "IMAGE_EDITOR":
                extra = sp.image.name if sp.image else "-"
            f.write("%s\t%s\t%s\t%d\t%d\t%d\t%d\t%s\t%s\n" % (
                ws.name, a.type, mode, a.x, a.y, a.width, a.height, regions, extra))
