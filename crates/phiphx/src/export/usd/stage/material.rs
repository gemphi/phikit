//! `SceneMaterial` as a `UsdPreviewSurface` material.
//!
//! What the scene carries of the Principled BSDF maps one to one: base
//! colour, roughness, metallic, IOR, emission (colour x strength), alpha,
//! coat. What it does not (transmission, subsurface, sheen, anisotropy,
//! procedural textures, the node graph) UsdPreviewSurface has no input for
//! and is left out; an image only reaches the layer by the path the scene
//! holds, and a packed image is not extracted.
use super::name::{UsdName, UsdNameBook};
use super::out::UsdOut;
use super::text::UsdText;
use super::texture::UsdTexture;
use crate::scene::{SceneMaterial, SceneModel};
use std::collections::HashMap;

/// The materials of a scene by name, each under `/World/Material`.
pub(super) struct MaterialBook<'a> {
    item: Vec<(String, &'a SceneMaterial)>,
    path: HashMap<&'a str, String>,
}

impl<'a> MaterialBook<'a> {
    /// Where the materials live; the Scope is written by [`Self::write`].
    pub const ROOT: &'static str = "/World/Material";

    /// Every material of every object, the first of a name winning (Blender
    /// keeps material names unique, so a repeat is the same material).
    pub fn of(scene: &'a SceneModel, names: &mut UsdNameBook) -> Self {
        let mut book = Self { item: Vec::new(), path: HashMap::new() };
        for m in scene.objects.iter().flat_map(|o| o.materials.iter()) {
            if book.path.contains_key(m.name.as_str()) {
                continue;
            }
            let id = names.claim(Self::ROOT, &m.name);
            book.path.insert(&m.name, format!("{}/{id}", Self::ROOT));
            book.item.push((id, m));
        }
        book
    }

    /// The prim path of the material called `name`.
    pub fn path(&self, name: &str) -> Option<&str> {
        self.path.get(name).map(String::as_str)
    }

    /// The `Scope` holding every material; nothing when there are none.
    pub fn write(&self, out: &mut UsdOut) {
        if self.item.is_empty() {
            return;
        }
        out.open("Scope", "Material", &[]);
        for (id, m) in &self.item {
            Self::write_one(out, id, m, &format!("{}/{id}", Self::ROOT));
        }
        out.close();
    }

    fn write_one(out: &mut UsdOut, id: &str, m: &SceneMaterial, at: &str) {
        let texture = Self::texture(m);
        out.open("Material", id, &UsdName::meta(&m.name, id));
        out.line("token inputs:frame:stPrimvarName = \"st\"");
        out.line(&format!("token outputs:surface.connect = <{at}/Surface.outputs:surface>"));
        out.open("Shader", "Surface", &[]);
        out.line("uniform token info:id = \"UsdPreviewSurface\"");
        let feeds = |input: &str| texture.iter().find(|t| t.input == input).map(|t| format!("<{at}/{}.outputs:{}>", t.name, t.channel));
        let [r, g, b, _] = m.base_color;
        Self::input(out, "color3f inputs:diffuseColor", UsdText::vec3([r, g, b]), feeds("color3f inputs:diffuseColor"));
        Self::input(out, "float inputs:roughness", UsdText::float(m.roughness), feeds("float inputs:roughness"));
        Self::input(out, "float inputs:metallic", UsdText::float(m.metallic), feeds("float inputs:metallic"));
        let glow = m.bsdf.emission.map(|c| c * m.bsdf.emission_strength);
        Self::input(out, "color3f inputs:emissiveColor", UsdText::vec3(glow), feeds("color3f inputs:emissiveColor"));
        out.line(&format!("float inputs:ior = {}", UsdText::float(m.bsdf.ior)));
        out.line(&format!("float inputs:opacity = {}", UsdText::float(m.bsdf.alpha)));
        if m.bsdf.coat > 0.0 {
            out.line(&format!("float inputs:clearcoat = {}", UsdText::float(m.bsdf.coat)));
            out.line(&format!("float inputs:clearcoatRoughness = {}", UsdText::float(m.bsdf.coat_roughness)));
        }
        if let Some(link) = feeds("normal3f inputs:normal") {
            out.line(&format!("normal3f inputs:normal.connect = {link}"));
        }
        out.line("token outputs:surface");
        out.close();
        if !texture.is_empty() {
            UsdTexture::write_reader(out, at);
            for t in &texture {
                t.write(out, at);
            }
        }
        out.close();
    }

    /// An input as a value, or connected to its image when it has one.
    fn input(out: &mut UsdOut, decl: &str, value: String, link: Option<String>) {
        match link {
            Some(link) => out.line(&format!("{decl}.connect = {link}")),
            None => out.line(&format!("{decl} = {value}")),
        }
    }

    fn key(s: &Option<crate::scene::SceneMap>) -> Option<&str> {
        s.as_ref().map(|k| k.key.as_str())
    }

    /// The images of `m` that reach a UsdPreviewSurface input.
    fn texture(m: &SceneMaterial) -> Vec<UsdTexture<'_>> {
        let slot = [
            ("Base", m.base_color_texture.as_deref(), true, "color3f inputs:diffuseColor", "rgb"),
            ("Rough", Self::key(&m.maps.rough), false, "float inputs:roughness", "r"),
            ("Metal", Self::key(&m.maps.metal), false, "float inputs:metallic", "r"),
            ("Emit", Self::key(&m.maps.emit), true, "color3f inputs:emissiveColor", "rgb"),
            ("Normal", m.normal_texture.as_ref().map(|(k, _)| k.as_str()), false, "normal3f inputs:normal", "rgb"),
        ];
        slot.into_iter().filter_map(|(name, file, srgb, input, channel)| Some(UsdTexture { name, file: file.filter(|f| !f.is_empty())?, srgb, input, channel })).collect()
    }
}
