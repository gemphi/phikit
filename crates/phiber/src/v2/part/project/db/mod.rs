//! The project's assets as a phiecs world — the Project Browser's database.
//!
//! Every datablock the open scene carries is a word, spelt `<kind>:<name>`
//! (`material:Skin`), with what the browser shows of it bound as an aspect
//! (`BAsset`); each kind is a word too, and two relations join them: an
//! asset is `in` its kind, and `uses` what it draws with — an object its
//! mesh and its materials, a material its image. The navigator counts a
//! kind by the words `in` it, the inspector lists what an asset uses and
//! what uses it, and the grid is the words of the kind picked.
//!
//! The world is built once per scene (keyed by the model's address) and
//! kept on the UI thread, which is the only one that browses.

use crate::phiphx::scene::SceneModel;
use phiecs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// A kind of asset, in the navigator's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BAssetKind {
    Scene,
    Object,
    Mesh,
    Material,
    Image,
    Camera,
    Light,
    Action,
    Collection,
}

impl BAssetKind {
    pub const ALL: [BAssetKind; 9] = [Self::Scene, Self::Object, Self::Mesh, Self::Material, Self::Image, Self::Camera, Self::Light, Self::Action, Self::Collection];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Scene => "scene",
            Self::Object => "object",
            Self::Mesh => "mesh",
            Self::Material => "material",
            Self::Image => "image",
            Self::Camera => "camera",
            Self::Light => "light",
            Self::Action => "action",
            Self::Collection => "collection",
        }
    }

    /// The navigator's folder name.
    pub fn label(self) -> &'static str {
        match self {
            Self::Scene => "Scenes",
            Self::Object => "Objects",
            Self::Mesh => "Meshes",
            Self::Material => "Materials",
            Self::Image => "Images",
            Self::Camera => "Cameras",
            Self::Light => "Lights",
            Self::Action => "Actions",
            Self::Collection => "Collections",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Scene => "scene",
            Self::Object => "object",
            Self::Mesh => "mesh",
            Self::Material => "material",
            Self::Image => "image",
            Self::Camera => "camera",
            Self::Light => "light",
            Self::Action => "action",
            Self::Collection => "collection",
        }
    }

    pub fn by_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.slug() == slug)
    }
}

/// What the browser shows of an asset.
#[derive(Clone, Debug, PartialEq)]
pub struct BAsset {
    pub kind: BAssetKind,
    pub name: String,
    /// Its line under the card: a mesh's size, an image's pixels.
    pub detail: String,
    /// The inspector's rows, label and value.
    pub info: Vec<(String, String)>,
    /// Its colour, where it has one (a material's base, a light's).
    pub colour: Option<[f32; 3]>,
}
impl Aspect for BAsset {}

pub struct BProjectDb {
    pub world: World,
    pub kinds: Vec<(BAssetKind, Word)>,
    pub within: Relation,
    pub uses: Relation,
}

impl BProjectDb {
    /// The words `in` `kind`, in the order they were coined.
    pub fn assets(&self, kind: BAssetKind) -> Vec<Word> {
        let Some(&(_, k)) = self.kinds.iter().find(|(c, _)| *c == kind) else { return Vec::new() };
        self.world.words().iter().copied().filter(|&w| self.world.holds(w, self.within, k)).collect()
    }

    pub fn asset(&self, word: Word) -> Option<&BAsset> {
        self.world.read::<BAsset>(word)
    }

    /// The word an asset is spelt as, if the project has it.
    pub fn find(&self, kind: BAssetKind, name: &str) -> Option<Word> {
        let spelling = format!("{}:{name}", kind.slug());
        self.world.words().iter().copied().find(|&w| self.world.spelling(w) == spelling)
    }

    /// What `word` uses.
    pub fn uses_of(&self, word: Word) -> Vec<Word> {
        self.world.calls_for(word, self.uses).into_iter().map(|(w, _)| w).collect()
    }

    /// What uses `word`.
    pub fn used_by(&self, word: Word) -> Vec<Word> {
        self.world.words().iter().copied().filter(|&w| self.world.holds(w, self.uses, word)).collect()
    }

    pub fn of(model: &SceneModel) -> BProjectDb {
        let mut world = World::new();
        world.register::<BAsset>();
        let within = world.declare("in").expect("a relation");
        let uses = world.declare("uses").expect("a relation");
        let kinds: Vec<(BAssetKind, Word)> = BAssetKind::ALL.iter().map(|&k| (k, world.coin_bare(&format!("kind:{}", k.slug())))).collect();
        let kind_word = |k: BAssetKind| kinds.iter().find(|(c, _)| *c == k).map(|(_, w)| *w).expect("every kind");
        let mut add = |world: &mut World, asset: BAsset| -> Word {
            let word = world.coin_bare(&format!("{}:{}", asset.kind.slug(), asset.name));
            if world.read::<BAsset>(word).is_none() {
                world.relate(word, within, kind_word(asset.kind));
                world.bind(word, asset);
            }
            word
        };
        let scene = if model.report.scene.is_empty() { "Scene".to_string() } else { model.report.scene.clone() };
        let r = &model.report;
        add(&mut world, BAsset { kind: BAssetKind::Scene, name: scene, detail: format!("{} objects", model.objects.len()), info: vec![("Objects".into(), r.objects.to_string()), ("Vertices".into(), r.verts.to_string()), ("Triangles".into(), r.tris.to_string()), ("Frames".into(), format!("{}–{}", model.timeline.frame_start, model.timeline.frame_end)), ("Version".into(), r.file_version.clone())], colour: None });
        // Meshes by their shared data: one mesh drawn by a thousand fans is
        // one mesh.
        let mut meshes: Vec<(*const crate::phiphx::scene::SceneMesh, Word)> = Vec::new();
        for o in &model.objects {
            let at = std::sync::Arc::as_ptr(&o.mesh);
            let mesh = match meshes.iter().find(|(p, _)| *p == at) {
                Some((_, w)) => *w,
                None => {
                    let (v, t) = (o.mesh.vertices.len(), o.mesh.indices.len() / 3);
                    let w = add(&mut world, BAsset { kind: BAssetKind::Mesh, name: o.name.clone(), detail: format!("{t} tris"), info: vec![("Vertices".into(), v.to_string()), ("Triangles".into(), t.to_string())], colour: None });
                    meshes.push((at, w));
                    w
                }
            };
            let [x, y, z] = o.location;
            let object = add(&mut world, BAsset { kind: BAssetKind::Object, name: o.name.clone(), detail: format!("{} slot{}", o.materials.len(), if o.materials.len() == 1 { "" } else { "s" }), info: vec![("Location".into(), format!("{x:.2}, {y:.2}, {z:.2}"))], colour: None });
            world.relate(object, uses, mesh);
            for m in &o.materials {
                let [r, g, b, _] = m.base_color;
                let material = add(&mut world, BAsset { kind: BAssetKind::Material, name: m.name.clone(), detail: format!("Roughness {:.2}", m.roughness), info: vec![("Base".into(), format!("{r:.2}, {g:.2}, {b:.2}")), ("Roughness".into(), format!("{:.2}", m.roughness)), ("Metallic".into(), format!("{:.2}", m.metallic)), ("Emission".into(), format!("{:.1}", m.bsdf.emission_strength))], colour: Some([r, g, b]) });
                world.relate(object, uses, material);
                if let Some(t) = &m.texture {
                    let name = m.base_color_texture.clone().unwrap_or_else(|| m.name.clone());
                    let image = add(&mut world, BAsset { kind: BAssetKind::Image, name, detail: format!("{} × {}", t.width, t.height), info: vec![("Size".into(), format!("{} × {} px", t.width, t.height))], colour: None });
                    world.relate(material, uses, image);
                }
            }
        }
        for c in &model.cameras {
            add(&mut world, BAsset { kind: BAssetKind::Camera, name: c.name.clone(), detail: format!("{:.0} mm", c.focal_length_mm), info: vec![("Focal length".into(), format!("{:.0} mm", c.focal_length_mm)), ("Clip".into(), format!("{:.2} – {:.0} m", c.clip_start, c.clip_end))], colour: None });
        }
        for l in &model.lights {
            add(&mut world, BAsset { kind: BAssetKind::Light, name: l.name.clone(), detail: l.kind.to_lowercase(), info: vec![("Type".into(), l.kind.clone()), ("Energy".into(), format!("{:.1}", l.energy))], colour: Some(l.color) });
        }
        for a in &model.actions {
            add(&mut world, BAsset { kind: BAssetKind::Action, name: a.name.clone(), detail: format!("{} curves", a.fcurve_count), info: vec![("Frames".into(), format!("{:.0}–{:.0}", a.frame_start, a.frame_end)), ("Curves".into(), a.fcurve_count.to_string()), ("Users".into(), a.users.join(", "))], colour: None });
        }
        fn walk(world: &mut World, add: &mut dyn FnMut(&mut World, BAsset) -> Word, c: &crate::phiphx::scene::SceneCollection) {
            add(world, BAsset { kind: BAssetKind::Collection, name: c.name.clone(), detail: format!("{} objects", c.object_indices.len()), info: vec![("Objects".into(), c.object_indices.len().to_string()), ("Children".into(), c.children.len().to_string())], colour: None });
            for child in &c.children {
                walk(world, add, child);
            }
        }
        for c in &model.collections {
            walk(&mut world, &mut add, c);
        }
        BProjectDb { world, kinds, within, uses }
    }
}

thread_local! {
    static DB: RefCell<Option<(usize, Rc<BProjectDb>)>> = const { RefCell::new(None) };
}

/// The open scene's database, built on first ask and kept while the scene
/// is the same one.
pub fn current() -> Option<Rc<BProjectDb>> {
    let model = crate::v2::scene::source::loaded_shared()?;
    let key = std::sync::Arc::as_ptr(&model) as usize;
    DB.with(|cell| {
        if let Some((_, db)) = cell.borrow().as_ref().filter(|(k, _)| *k == key) {
            return Some(db.clone());
        }
        let db = Rc::new(BProjectDb::of(&model));
        *cell.borrow_mut() = Some((key, db.clone()));
        Some(db)
    })
}

#[cfg(test)]
mod tests;
