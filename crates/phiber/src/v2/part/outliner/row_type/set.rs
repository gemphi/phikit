//! Row icons per Blender ID type (the common ones).
//!
//! Blender ref: `bl_ui/space_outliner.py` and
//! `editors/space_outliner/`.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BOutlineRowType {
    SceneCollection,
    Collection,
    Object,
    Mesh,
    Curve,
    Surface,
    Metaball,
    Text,
    Lattice,
    GreasePencil,
    Camera,
    Light,
    LightProbe,
    Speaker,
    Armature,
    Bone,
    Material,
    Texture,
    Image,
    Shader,
    Modifier,
    Constraint,
    ParticleSystem,
    Action,
    Nla,
    Driver,
    ShapeKey,
    World,
    WorkSpace,
    Library,
    Node,
    Brush,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOutlineRowSpec {
    pub kind: BOutlineRowType,
    pub icon: &'static str,
    pub label: &'static str,
}

impl BSpec for BOutlineRowSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

const fn r(kind: BOutlineRowType, icon: &'static str, label: &'static str) -> BOutlineRowSpec {
    BOutlineRowSpec { kind, icon, label }
}

pub static ROW_TYPES: &[BOutlineRowSpec] = &[
    r(
        BOutlineRowType::SceneCollection,
        "scene-collection",
        "Scene Collection",
    ),
    r(BOutlineRowType::Collection, "collection", "Collection"),
    r(BOutlineRowType::Object, "object", "Object"),
    r(BOutlineRowType::Mesh, "mesh", "Mesh"),
    r(BOutlineRowType::Curve, "curve", "Curve"),
    r(BOutlineRowType::Surface, "surface", "Surface"),
    r(BOutlineRowType::Metaball, "metaball", "Metaball"),
    r(BOutlineRowType::Text, "text", "Text"),
    r(BOutlineRowType::Lattice, "lattice", "Lattice"),
    r(
        BOutlineRowType::GreasePencil,
        "grease-pencil",
        "Grease Pencil",
    ),
    r(BOutlineRowType::Camera, "camera", "Camera"),
    r(BOutlineRowType::Light, "light", "Light"),
    r(BOutlineRowType::LightProbe, "light-probe", "Light Probe"),
    r(BOutlineRowType::Speaker, "speaker", "Speaker"),
    r(BOutlineRowType::Armature, "armature", "Armature"),
    r(BOutlineRowType::Bone, "bone", "Bone"),
    r(BOutlineRowType::Material, "material", "Material"),
    r(BOutlineRowType::Texture, "texture", "Texture"),
    r(BOutlineRowType::Image, "image", "Image"),
    r(BOutlineRowType::Shader, "shader", "Shader"),
    r(BOutlineRowType::Modifier, "modifier", "Modifier"),
    r(BOutlineRowType::Constraint, "constraint", "Constraint"),
    r(
        BOutlineRowType::ParticleSystem,
        "particles",
        "Particle System",
    ),
    r(BOutlineRowType::Action, "action", "Action"),
    r(BOutlineRowType::Nla, "nla", "NLA Strip"),
    r(BOutlineRowType::Driver, "driver", "Driver"),
    r(BOutlineRowType::ShapeKey, "shape-key", "Shape Key"),
    r(BOutlineRowType::World, "world", "World"),
    r(BOutlineRowType::WorkSpace, "workspace", "Workspace"),
    r(BOutlineRowType::Library, "library", "Library"),
    r(BOutlineRowType::Node, "node", "Node"),
    r(BOutlineRowType::Brush, "brush", "Brush"),
];
