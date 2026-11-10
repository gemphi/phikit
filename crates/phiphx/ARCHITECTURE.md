# phiphx — architecture

Solid boxes are in the crate today. Dashed boxes are to build, each named
with the reference it is read from (`reference/<repo>`). Blender stays the
specification for everything a `.blend` shows; the references add what a
`.blend` does not hold.

## The modules and what they use

```mermaid
graph LR
    subgraph today["in the crate today"]
        m_phx["phx<br/>Phx: open · peek · detect"]
        m_import["import<br/>asset router · gltf · usd · ply · phx"]
        m_blend["blend<br/>file · object · mesh · modifier · rig · anim ·<br/>shade · physics · curve · text · asset · write"]
        m_scene["scene<br/>SceneModel: objects · materials · textures ·<br/>rigs · motion · progressive"]
        m_comp["comp<br/>compositor: tree · node · eval"]
        m_sky["sky<br/>atmosphere · spectrum"]
        m_export["export<br/>blend · gltf · usd · phx · scene · ply · las · pcd · xyz"]
        m_cloud["cloud<br/>PointCloud, sampling"]
        m_geom["geom<br/>mesh · bvh · bounds · vertex"]
        m_format["format<br/>output formats, materials"]
        m_uv["uv<br/>island · lscm · pack"]
        m_spatial["spatial<br/>cards, glyphs, implicit shapes"]
        m_project["project"]
    end
    subgraph build["to build"]
        m_char["character<br/>the contract checked at open,<br/>a rig as asset + pose for phiecs<br/>momentum"]
        m_bound["bound splats<br/>Gaussians that follow a rig or a cage<br/>D3GA · goliath"]
        m_body["parametric body<br/>identity · pose · correctives · levels of detail<br/>MHR"]
        m_recon["reconstruction in<br/>cameras, depth and points from images<br/>vggt · map-anything · sam-3d-objects"]
        m_skinout["rig out<br/>skins and joints in the glTF writer<br/>momentum io/gltf"]
    end
    m_phx --> m_blend
    m_phx --> m_import
    m_phx --> m_scene
    m_import --> m_blend
    m_import --> m_export
    m_blend --> m_scene
    m_scene --> m_blend
    m_blend --> m_comp
    m_comp --> m_blend
    m_scene --> m_comp
    m_blend --> m_sky
    m_scene --> m_sky
    m_blend --> m_export
    m_cloud --> m_geom
    m_geom --> m_cloud
    m_cloud --> m_export
    m_cloud --> m_format
    m_project --> m_import
    m_char -.-> m_scene
    m_char -.-> m_blend
    m_bound -.-> m_char
    m_body -.-> m_char
    m_body -.-> m_import
    m_recon -.-> m_cloud
    m_recon -.-> m_import
    m_skinout -.-> m_export
    m_skinout -.-> m_char
    classDef plan stroke-dasharray: 6 4;
    class m_char,m_bound,m_body,m_recon,m_skinout plan;
```

## A file to a frame, and where it is handed on

```mermaid
graph TB
    f_file[".blend"]
    f_parse["blend/file<br/>header · sdna · blocks · zstd"]
    f_read["blend/object · mesh · modifier · rig · anim · shade"]
    f_model["scene::SceneModel"]
    f_rigs["scene/rig.rs: SceneRigs<br/>bones_at · meshes_at · skinned_at"]
    f_time["scene/types_ext.rs: SceneTimeline"]
    subgraph out["who takes it"]
        o_ray["phiray<br/>Scene, then Tracer::repose a frame"]
        o_ber["phiber<br/>views, Project Browser"]
        o_ecs["phiecs<br/>a word an object: Pose · Skeleton · JointPose · Asset"]
        o_files["export<br/>phx · gltf · usd · point clouds"]
    end
    f_file --> f_parse --> f_read --> f_model
    f_model --> f_rigs
    f_model --> f_time
    f_model --> o_ray
    f_rigs --> o_ray
    f_model --> o_ber
    f_model --> o_files
    f_rigs -.-> o_ecs
    f_time -.-> o_ecs
    classDef plan stroke-dasharray: 6 4;
    class o_ecs plan;
```

## Pages, one subject each

- [character](docs/character.md): Blender's rig beside momentum's, the contract, what drives each reference's deformation

Folder notes: [scene](src/scene/README.md) · [blend](src/blend/README.md) ·
[import](src/import/README.md) · [export](src/export/README.md) ·
[cloud](src/cloud/README.md) · [geom](src/geom/README.md) ·
[format](src/format/README.md) · [spatial](src/spatial/README.md)

Folders and order of work for the new lanes: [PLAN.md](PLAN.md) ·
What each part must do: [SPECS.md](SPECS.md) ·
The references: [reference/README.md](reference/README.md) ·
Blender parity, what is left to do: [TODO.md](TODO.md)
