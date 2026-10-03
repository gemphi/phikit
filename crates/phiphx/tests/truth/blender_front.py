import bpy, sys, math
try: bpy.ops.wm.open_mainfile(filepath=sys.argv[-2], load_ui=False)
except RuntimeError: pass
out=sys.argv[-1]; sc=bpy.context.scene
# bounds of visible meshes (evaluated), like the phiber example
import numpy as np
dg=bpy.context.evaluated_depsgraph_get(); lo=np.array([1e9]*3); hi=-lo
for o in sc.objects:
    if o.type!='MESH' or not o.visible_get(): continue
    ev=o.evaluated_get(dg); me=ev.to_mesh(); n=len(me.vertices)
    if n: a=np.empty(n*3); me.vertices.foreach_get("co",a); a=a.reshape(-1,3)@np.array(ev.matrix_world)[:3,:3].T+np.array(ev.matrix_world)[:3,3]; lo=np.minimum(lo,a.min(0)); hi=np.maximum(hi,a.max(0))
    ev.to_mesh_clear()
height=hi[2]-lo[2]; cz=(lo[2]+hi[2])/2; W,H=720,1080; aspect=W/H
cam=bpy.data.cameras.new("front"); cam.type='ORTHO'; cam.ortho_scale=height*1.1/aspect**2
ob=bpy.data.objects.new("front",cam); sc.collection.objects.link(ob)
ob.location=(0,-10,cz); ob.rotation_euler=(math.pi/2,0,0); sc.camera=ob
sc.render.engine='BLENDER_WORKBENCH'; sc.render.resolution_x=W; sc.render.resolution_y=H; sc.render.resolution_percentage=100
sh=sc.display.shading; sh.light='STUDIO'; sh.color_type='MATERIAL'; sh.show_cavity=False; sh.show_shadows=False; sh.show_object_outline=False
sc.display.render_aa='OFF'
sc.view_settings.view_transform='Standard'
w=sc.world or bpy.data.worlds.new("w"); sc.world=w; w.use_nodes=False; w.color=(0.048,0.048,0.048)  # 0.24 display ≈ 0.048 linear
sc.render.film_transparent=False
sc.render.image_settings.file_format='PNG'; sc.render.filepath=out
bpy.ops.render.render(write_still=True)
print("wrote",out,"height",height,"ortho",cam.ortho_scale)
