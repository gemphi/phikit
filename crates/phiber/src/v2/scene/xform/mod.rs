//! G, R and S in the 3D Viewport — Blender's `transform.translate`,
//! `transform.rotate` and `transform.resize` on the selected objects: the
//! pointer drives the change from where it first stood (`set.rs`), X / Y / Z
//! hold it to a world axis and Shift X / Y / Z to a plane (`constraint.rs`),
//! a number typed drives it instead (`typed.rs`), a click or Return keeps
//! it, Escape or the right button puts everything back (`rule.rs`). A child
//! of an object that moves too is left to follow its parent, as Blender
//! skips it.
//!
//! The transform under way keeps what it started from in
//! `ctx.view3d.transform.modal` —
//! `<mode>#<constraint>#<from x,y>#<at x,y>#<typed>#<objects>#<before>`,
//! each object `name=<its own transform>;<the parent part of its world
//! matrix>` and `<before>` the edits list as it was — so every move is
//! measured from the start and a cancel restores exactly. Its moves land
//! in the Transform panel's own key (`scene::transform`): the object
//! moves, its children follow, a save writes it, and the whole transform
//! is one undo step (`app/handle.rs` holds the history while it runs).

pub mod constraint;
pub mod rule;
pub mod set;
pub mod typed;

pub use constraint::BConstraint;
pub use rule::BTransformOpRule;
pub use set::{BPointerTravel, BTransformMode};
pub use typed::BNumInput;

use super::edit::BSceneEdit;
use super::links::BSceneTree;
use super::pick::BPickView;
use super::select::BSelection;
use super::transform::{BTransform, BTransforms};
use super::BObject3D;
use crate::v2::state::{BKey as _, BState, SceneKey, View3dKey, log};
use crate::v2::view::prim::BAction;
use glam::{Mat4, Vec3};

/// One object the transform moves: its own transform when it began, and
/// the part of its world matrix that is its parent's (`world · local⁻¹`).
#[derive(Clone, Debug, PartialEq)]
pub struct BTransformEntry {
    pub name: String,
    pub start: BTransform,
    pub parent: Mat4,
}

impl BTransformEntry {
    fn world(&self) -> Mat4 {
        self.parent * self.start.matrix()
    }

    /// Its own transform once the world has changed by `delta`: a move
    /// keeps its rotation and scale exactly, a turn its scale, a scale its
    /// rotation (`transform_convert_object.cc`, `ElementRotation` et al.).
    fn after(&self, mode: BTransformMode, delta: Mat4) -> BTransform {
        let moved = BTransform::of_matrix(self.parent.inverse() * delta * self.world());
        match mode {
            BTransformMode::Translate => BTransform { location: moved.location, ..self.start },
            BTransformMode::Rotate => BTransform { scale: self.start.scale, ..moved },
            BTransformMode::Resize => BTransform { rotation: self.start.rotation, ..moved },
        }
    }
}

/// A transform under way.
#[derive(Clone, Debug, PartialEq)]
pub struct BTransformModal {
    pub mode: BTransformMode,
    /// What it is held to.
    pub constraint: BConstraint,
    /// Where the pointer first stood on the canvas; `None` until it moves.
    pub from: Option<(f32, f32)>,
    /// Where the pointer is now.
    pub at: Option<(f32, f32)>,
    /// A number typed, which drives it while there is one.
    pub typed: BNumInput,
    pub list: Vec<BTransformEntry>,
    /// `ctx.scene.transformed` from before it began.
    pub before: String,
}

impl BTransformModal {
    pub fn now(s: &BState) -> Option<Self> {
        let raw = s.get(View3dKey::Transform).filter(|raw| !raw.is_empty())?;
        let mut parts = raw.splitn(7, '#');
        let mode = BTransformMode::from_slug(parts.next()?)?;
        let constraint = BConstraint::parse(parts.next()?);
        let point = |raw: &str| raw.split_once(',').and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?)));
        let (from, at) = (point(parts.next()?), point(parts.next()?));
        let typed = BNumInput(parts.next()?.to_string());
        let list = parts.next()?.split('|').filter_map(Self::parse_entry).collect();
        Some(Self { mode, constraint, from, at, typed, list, before: parts.next().unwrap_or("").to_string() })
    }

    pub fn active(s: &BState) -> bool {
        Self::now(s).is_some()
    }

    fn parse_entry(raw: &str) -> Option<BTransformEntry> {
        let (name, rest) = raw.split_once('=')?;
        let (start, parent) = rest.split_once(';')?;
        let m: Vec<f32> = parent.split(',').filter_map(|v| v.parse().ok()).collect();
        let parent = (m.len() == 16).then(|| Mat4::from_cols_slice(&m))?;
        Some(BTransformEntry { name: name.to_string(), start: BTransform::parse(start)?, parent })
    }

    /// The key's value, as [`Self::now`] reads it back.
    pub fn encode(&self) -> String {
        let point = |p: Option<(f32, f32)>| p.map(|(x, y)| format!("{x},{y}")).unwrap_or_default();
        let list: Vec<String> = self
            .list
            .iter()
            .map(|e| format!("{}={};{}", e.name, e.start.encode(), e.parent.to_cols_array().map(|v| v.to_string()).join(",")))
            .collect();
        let (mode, constraint) = (self.mode.slug(), self.constraint.encode());
        format!("{mode}#{constraint}#{}#{}#{}#{}#{}", point(self.from), point(self.at), self.typed.0, list.join("|"), self.before)
    }

    /// The median of the objects' origins — Blender's default pivot.
    pub fn pivot(&self) -> Vec3 {
        let sum: Vec3 = self.list.iter().map(|e| e.world().w_axis.truncate()).sum();
        sum / self.list.len().max(1) as f32
    }

    /// G / R / S: the selected objects of the scene shown — meshes, cameras,
    /// lights and empties — less those whose parent moves with them. With
    /// none, it says so and nothing starts.
    pub fn start(s: &mut BState, mode: BTransformMode) {
        let selection = BSelection::now(s);
        let edits = BTransforms::now(s);
        let edit = BSceneEdit::now(s);
        let scene = super::scene_objects(s);
        let list: Vec<BTransformEntry> = super::source::with_loaded(|model| {
            let Some(model) = model else { return Vec::new() };
            // Where each object stands now: the file's meshes, cameras,
            // lights and empties with the edits in, and the session's own.
            let old = BTransforms::posed(s, model, super::motion::shown_frame(s));
            let mut worlds = edits.worlds(s, model, &old);
            for object in scene.iter() {
                if let BObject3D::Loaded { name, matrix_world, .. } = object {
                    worlds.insert(name.clone(), Mat4::from_cols_array_2d(matrix_world));
                }
            }
            let tree = BSceneTree::now(s, model);
            let parent_moves = |name: &str| selection.list.iter().any(|other| other != name && tree.descends_from(name, other));
            selection
                .list
                .iter()
                .filter(|name| !parent_moves(name))
                .filter_map(|name| {
                    let world = *worlds.get(name)?;
                    let local = BTransforms::file_local(&edit, model, name).unwrap_or(world);
                    let start = edits.get(name).unwrap_or_else(|| BTransform::of_matrix(local));
                    Some(BTransformEntry { name: name.clone(), start, parent: world * start.matrix().inverse() })
                })
                .collect()
        });
        if list.is_empty() {
            log::push("No objects selected to transform".to_string());
            return;
        }
        let before = s.get(SceneKey::Transformed).unwrap_or("").to_string();
        let modal = Self { mode, constraint: BConstraint::Free, from: None, at: None, typed: BNumInput::default(), list, before };
        s.set(View3dKey::Transform, modal.encode());
    }

    /// The edits list with the objects changed by `delta` in world space.
    fn edits(&self, delta: Mat4) -> BTransforms {
        let mut edits = BTransforms::parse(&self.before);
        for entry in &self.list {
            edits.set(&entry.name, entry.after(self.mode, delta));
        }
        edits
    }

    /// Where it stands now, in `view` on a canvas of `size`: the number
    /// typed when there is one, else the pointer's travel — `None` before
    /// the pointer has moved.
    pub fn applied(&self, view: &BPickView, size: (f32, f32)) -> Option<BTransforms> {
        if let Some(value) = self.typed.value() {
            return Some(self.edits(self.mode.typed(self.pivot(), self.constraint, value, view.axes().2)));
        }
        let (from, to) = self.from.zip(self.at)?;
        Some(self.edits(self.mode.delta(view, self.pivot(), self.constraint, BPointerTravel { from, to, size })))
    }

    /// The pointer moved to `at` on a canvas of `size`, in `view`: it is
    /// kept (the first move is where the travel starts), and — unless a
    /// number drives it — the objects follow.
    pub fn moved(&self, view: &BPickView, at: (f32, f32), size: (f32, f32)) -> Vec<BAction> {
        let next = Self { from: self.from.or(Some(at)), at: Some(at), ..self.clone() };
        let mut out = vec![BAction::Set(View3dKey::Transform.path(), next.encode())];
        if next.typed.is_empty() {
            if let Some(edits) = next.applied(view, size) {
                out.push(BAction::Set(SceneKey::Transformed.path(), edits.encode()));
            }
        }
        out
    }

    /// The read-out Blender shows in the header while it runs: how far,
    /// how much turned, how much scaled — the number being typed when one
    /// is — and what it is held to.
    pub fn readout(&self, s: &BState) -> String {
        let Some(first) = self.list.first() else { return String::new() };
        let now = BTransforms::now(s).get(&first.name).unwrap_or(first.start);
        let shown = match (self.mode, self.typed.is_empty()) {
            (_, false) => format!("[{}|]", self.typed.0),
            (BTransformMode::Translate, true) => {
                let d = first.parent.transform_vector3(Vec3::from(now.location) - Vec3::from(first.start.location));
                format!("{:.4} m ({:.4}, {:.4}, {:.4})", d.length(), d.x, d.y, d.z)
            }
            (BTransformMode::Rotate, true) => {
                let turn = now.matrix().to_scale_rotation_translation().1 * first.start.matrix().to_scale_rotation_translation().1.inverse();
                format!("{:.2}°", turn.to_axis_angle().1.to_degrees())
            }
            (BTransformMode::Resize, true) => {
                let r = Vec3::from(now.scale) / Vec3::from(first.start.scale).max(Vec3::splat(1e-6));
                format!("{:.4} {:.4} {:.4}", r.x, r.y, r.z)
            }
        };
        let name = match self.mode {
            BTransformMode::Translate => "D",
            BTransformMode::Rotate => "Rot",
            BTransformMode::Resize => "Scale",
        };
        format!("{name}: {shown}{}", self.constraint.label())
    }

    /// For R and S, the pivot and the pointer: Blender's dashed guide line.
    pub fn guide(&self) -> Option<(Vec3, (f32, f32))> {
        (self.mode != BTransformMode::Translate).then_some(()).and(self.at).map(|at| (self.pivot(), at))
    }
}
