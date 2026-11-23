//! A stage into the world: every active prim a word, spelled by its
//! path, standing in its parent's word, with its pose from its
//! `xformOp`s, its type and its properties. A Reality Composer Pro
//! component (a child prim of type `RealityKitComponent` or
//! `RealityKitCustomComponent`) is not a thing of its own but data on
//! the thing it is in: one of its [`ComponentSet`], with its id
//! (`info:id`) and its properties, its structs flattened into them.

use crate::aspect::Aspect;
use crate::comp::pose::Pose;
use crate::comp::tree::adopt;
use crate::word::Word;
use crate::world::World;

use super::layer::{Prim, Property, Spec};
use super::stage::Stage;
use super::value::Value;

/// What a prim is: its USD type, and its properties.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Kind {
    pub kind: String,
    pub property: Vec<Property>,
}

impl Aspect for Kind {}

impl Kind {
    /// The value of one of its properties.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.property.iter().find(|p| p.name == name).map(|p| &p.value)
    }
}

/// A component authored on a prim: its id and its values.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Component {
    pub id: String,
    pub property: Vec<Property>,
}

impl Component {
    /// One of its values; a struct's are written `struct.name`.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.property.iter().find(|p| p.name == name).map(|p| &p.value)
    }
}

/// The components authored on a thing.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ComponentSet(pub Vec<Component>);

impl Aspect for ComponentSet {}

impl ComponentSet {
    /// The component of that id (`RealityKit.Collider`, say).
    pub fn find(&self, id: &str) -> Option<&Component> {
        self.0.iter().find(|c| c.id == id)
    }
}

/// The types that are components, not things.
const COMPONENT: [&str; 2] = ["RealityKitComponent", "RealityKitCustomComponent"];

/// Put a stage's prims into the world, under `parent` where one is
/// given. A prim's word is spelled by its path, after its parent's
/// spelling where it has a parent: two scenes' `/Root` under two parts
/// are two things. The words made, the root's first.
pub fn intake(world: &mut World, stage: &Stage, parent: Option<Word>) -> Vec<Word> {
    let above = parent.and_then(|p| world.lexicon.spelling(p)).unwrap_or_default().to_string();
    let mut made = Vec::new();
    for prim in &stage.prim {
        place(world, prim, &above, parent, &mut made);
    }
    made
}

/// One prim and what is under it.
fn place(world: &mut World, prim: &Prim, above: &str, parent: Option<Word>, made: &mut Vec<Word>) {
    let active = prim.meta("active").is_none_or(Value::truth);
    if prim.spec == Spec::Class || !active || COMPONENT.contains(&prim.kind.as_str()) {
        return;
    }
    let path = format!("{above}/{}", prim.name);
    let word = world.coin_bare(&path);
    made.push(word);
    world.bind(word, pose(prim));
    world.bind(word, Kind { kind: prim.kind.clone(), property: prim.property.clone() });
    let component: Vec<Component> = prim.child.iter().filter(|c| COMPONENT.contains(&c.kind.as_str())).map(component).collect();
    if !component.is_empty() {
        world.bind(word, ComponentSet(component));
    }
    parent.into_iter().for_each(|parent| adopt(world, word, parent));
    for child in &prim.child {
        place(world, child, &path, Some(word), made);
    }
}

/// A prim's pose from its transform ops.
fn pose(prim: &Prim) -> Pose {
    let get = |name: &str| prim.get(name).map(Value::numbers).unwrap_or_default();
    let three = |v: Vec<f32>, or: [f32; 3]| match v.as_slice() {
        [x, y, z] => [*x, *y, *z],
        [s] => [*s; 3],
        _ => or,
    };
    let orient = match get("xformOp:orient").as_slice() {
        [w, x, y, z] => [*w, *x, *y, *z],
        _ => [1.0, 0.0, 0.0, 0.0],
    };
    Pose { translate: three(get("xformOp:translate"), [0.0; 3]), orient, scale: three(get("xformOp:scale"), [1.0; 3]) }
}

/// A component prim as data: its id, its values, its structs' values
/// under their names.
fn component(prim: &Prim) -> Component {
    let id = prim.get("info:id").map_or(prim.name.as_str(), Value::text).to_string();
    let mut property = prim.property.clone();
    let mut stack: Vec<(String, &Prim)> = prim.child.iter().map(|c| (c.name.clone(), c)).collect();
    while let Some((name, child)) = stack.pop() {
        property.extend(child.property.iter().map(|p| Property { name: format!("{name}.{}", p.name), ..p.clone() }));
        stack.extend(child.child.iter().map(|c| (format!("{name}.{}", c.name), c)));
    }
    Component { id, property }
}
