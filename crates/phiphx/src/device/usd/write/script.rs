//! USDA scene writer: a `BlenderScene` goes through the one scene writer.
//!
//! `BlenderScene` is the flat record a Blender-side script hands over; the
//! writer of record is [`UsdSceneWriter`](super::scene::UsdSceneWriter) over a
//! `SceneModel`, so this turns the record into one and lets it write.
use crate::device::blend::write::script::types::{BlenderLightType, BlenderScene, CameraData, CameraType, LightData, MaterialData, MeshObjectData};
use crate::scene::{SceneCamera, SceneCollection, SceneLight, SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
use crate::Result;
use std::sync::Arc;

impl BlenderScene {
    /// Export this scene as USDA through the scene writer: meshes with
    /// normals, UVs and materials, instances as shared prototypes, lights,
    /// the camera, and the collection tree. What the record does not hold
    /// (a world, an ortho camera's scale, parents) takes the scene's default.
    pub fn to_usda(&self) -> Result<String> {
        Ok(self.scene_model().to_usda())
    }

    /// The `SceneModel` this record describes.
    pub fn scene_model(&self) -> SceneModel {
        let mut model = SceneModel::default();
        model.report.scene = self.name.clone();
        model.render.resolution_x = self.resolution[0];
        model.render.resolution_y = self.resolution[1];
        model.objects = self.meshes.iter().map(|m| self.object(m)).collect();
        for inst in &self.instances {
            let Some(proto) = model.objects.iter().find(|o| o.name == inst.prototype_name).cloned() else { continue };
            for (k, t) in inst.transforms.iter().enumerate() {
                let mut copy = proto.clone();
                copy.name = format!("{}.inst{k}", proto.name);
                copy.matrix_world = glam::Mat4::from_cols_array(t).to_cols_array_2d();
                model.objects.push(copy);
            }
        }
        model.selection.render_disabled = self.meshes.iter().filter(|m| !m.visible).map(|m| m.name.clone()).collect();
        if let Some(cam) = &self.camera {
            model.render.camera = cam.name.clone();
            model.cameras.push(Self::camera(cam));
        }
        model.lights = self.lights.iter().map(Self::light).collect();
        model.collections = self.collection_tree();
        model
    }

    fn object(&self, m: &MeshObjectData) -> SceneObject {
        let tri = m.indices.len();
        let normals = match m.normals.len() {
            n if n == m.vertices.len() => SceneNormals::PerVertex(m.normals.clone()),
            n if n == tri => SceneNormals::PerFace(m.normals.clone()),
            _ => SceneNormals::PerFace(Vec::new()),
        };
        let mesh = SceneMesh { vertices: m.vertices.clone(), indices: m.indices.iter().flatten().copied().collect(), normals, corners: Default::default() };
        let mut o = SceneObject::identity(m.name.clone(), mesh);
        o.matrix_world = m.matrix_world;
        o.uvs = Arc::new(m.uvs.clone());
        o.materials = m.material_names.iter().map(|n| self.material(n)).collect();
        o
    }

    /// The record's material called `name`, or a grey one by that name.
    fn material(&self, name: &str) -> SceneMaterial {
        let Some(d) = self.materials.iter().find(|d| d.name == name) else { return SceneMaterial { name: name.into(), ..SceneMaterial::neutral() } };
        Self::material_of(d)
    }

    fn material_of(d: &MaterialData) -> SceneMaterial {
        let mut m = SceneMaterial { name: d.name.clone(), base_color: d.base_color, roughness: d.roughness, metallic: d.metallic, ..SceneMaterial::neutral() };
        m.viewport_color = d.base_color;
        m.bsdf.emission = d.emission;
        m.bsdf.emission_strength = d.emission_strength;
        m.bsdf.alpha = d.alpha;
        m.base_color_texture = d.base_color_texture.clone();
        m.normal_texture = d.normal_texture.clone().map(|t| (t, 1.0));
        m
    }

    /// `CameraData::fov` is the horizontal angle over Blender's 36 mm film.
    fn camera(c: &CameraData) -> SceneCamera {
        let kind = match c.camera_type {
            CameraType::Perspective => "PERSP",
            CameraType::Orthographic => "ORTHO",
            CameraType::Panoramic => "PANO",
        };
        SceneCamera {
            name: c.name.clone(),
            location: c.location,
            matrix_world: c.matrix_world,
            kind: kind.into(),
            focal_length_mm: 18.0 / (c.fov / 2.0).tan(),
            sensor_width: 36.0,
            sensor_height: 24.0,
            sensor_fit: "AUTO".into(),
            shift: [0.0; 2],
            ortho_scale: 6.0,
            clip_start: c.clip_start,
            clip_end: c.clip_end,
            display_size: 1.0,
            dof: Default::default(),
        }
    }

    /// A script reads `Light.energy` in the UI's watts, where the scene
    /// stores an area light's as 4/pi of that (`scene/extras.rs`).
    fn light(l: &LightData) -> SceneLight {
        let (kind, energy) = match l.light_type {
            BlenderLightType::Point => ("POINT", l.energy),
            BlenderLightType::Sun => ("SUN", l.energy),
            BlenderLightType::Spot => ("SPOT", l.energy),
            BlenderLightType::Area => ("AREA", l.energy * 4.0 / std::f32::consts::PI),
        };
        let [w, h] = l.area_size.unwrap_or([0.0; 2]);
        SceneLight {
            name: l.name.clone(),
            location: l.location,
            matrix_world: l.matrix_world,
            color: l.color,
            energy,
            kind: kind.into(),
            spot_size: l.spot_angle.unwrap_or(0.0),
            spot_blend: l.spot_blend.unwrap_or(0.0),
            size: w,
            size_y: h,
            shape: if (w - h).abs() < f32::EPSILON { "SQUARE" } else { "RECTANGLE" }.into(),
        }
    }

    /// The record's collections as a tree under one top collection: those
    /// no other collection lists as a child are its children.
    fn collection_tree(&self) -> Vec<SceneCollection> {
        let node = |c: &crate::device::blend::write::script::types::CollectionData| SceneCollection {
            name: c.name.clone(),
            object_indices: c.object_names.iter().filter_map(|n| self.meshes.iter().position(|m| &m.name == n)).collect(),
            extra_names: c.object_names.iter().filter(|n| self.lights.iter().any(|l| &l.name == *n) || self.camera.as_ref().is_some_and(|k| &k.name == *n)).cloned().collect(),
            ..Default::default()
        };
        let listed: std::collections::HashSet<&String> = self.collections.iter().flat_map(|c| &c.child_collection_names).collect();
        let build = |c: &crate::device::blend::write::script::types::CollectionData| -> SceneCollection {
            // Depth is bounded by the number of collections, so a cycle ends.
            fn go(s: &BlenderScene, c: &crate::device::blend::write::script::types::CollectionData, node: &dyn Fn(&crate::device::blend::write::script::types::CollectionData) -> SceneCollection, left: usize) -> SceneCollection {
                let mut out = node(c);
                if left > 0 {
                    out.children = c.child_collection_names.iter().filter_map(|n| s.collections.iter().find(|k| &k.name == n)).map(|k| go(s, k, node, left - 1)).collect();
                }
                out
            }
            go(self, c, &node, self.collections.len())
        };
        let top = SceneCollection { name: "Scene Collection".into(), children: self.collections.iter().filter(|c| !listed.contains(&c.name)).map(build).collect(), ..Default::default() };
        vec![top]
    }
}
