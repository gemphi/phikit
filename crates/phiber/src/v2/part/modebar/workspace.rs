//! A workspace's object mode — `WorkSpace.object_mode`: switching to a
//! workspace puts the active object in its mode when the object can take
//! it (`ED_workspace_change` → `ED_object_mode_set`). Blender 5.0's
//! factory workspaces (checked with bpy): Modeling and UV Editing in Edit
//! Mode, Sculpting in Sculpt Mode, Texture Paint in Texture Paint,
//! Animation in Pose Mode, the rest in Object Mode. A mode the object
//! cannot take — Pose Mode on a mesh — leaves it as it was.

use crate::v2::space::BSpaceType;
use crate::v2::state::val::V_MODE_OBJECT;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, rule::BRule};

/// One workspace's mode.
pub struct BWorkspaceMode;

impl BWorkspaceMode {
    /// The mode `space` puts the active object in, as the mode menu names it.
    pub fn of(space: BSpaceType) -> &'static str {
        match space {
            BSpaceType::Modeling | BSpaceType::UvEditing => "Edit Mode",
            BSpaceType::Sculpting => "Sculpt Mode",
            BSpaceType::TexturePaint => "Texture Paint",
            BSpaceType::Animation => "Pose Mode",
            _ => V_MODE_OBJECT,
        }
    }

    /// Can an object of type `kind` (`Object.type`) be in `mode`
    /// (`ED_object_mode_compat_test`)?
    pub fn takes(kind: &str, mode: &str) -> bool {
        match mode {
            "Edit Mode" => matches!(kind, "MESH" | "CURVE" | "SURFACE" | "FONT" | "META" | "LATTICE" | "ARMATURE" | "CURVES" | "GREASEPENCIL"),
            "Sculpt Mode" | "Texture Paint" => kind == "MESH",
            "Pose Mode" => kind == "ARMATURE",
            _ => true,
        }
    }

    /// The active object's type: a loaded object's, a camera's, light's or
    /// empty's, else a mesh (the default scene's and every added object).
    fn kind(s: &BState, object: &str) -> String {
        crate::v2::scene::source::with_loaded(|model| {
            let Some(m) = model else { return "MESH".to_string() };
            let file = crate::v2::scene::edit::BSceneEdit::now(s).renamed.iter().find(|(new, _)| new == object).map_or(object.to_string(), |(_, old)| old.clone());
            if let Some(i) = m.objects.iter().position(|o| o.name == file) {
                return m.object_types.get(i).cloned().unwrap_or_else(|| "MESH".to_string());
            }
            let extra = [(m.cameras.iter().any(|c| c.name == file), "CAMERA"), (m.lights.iter().any(|l| l.name == file), "LIGHT"), (m.empties.iter().any(|e| e.name == file), "EMPTY")];
            extra.into_iter().find(|(is, _)| *is).map_or("MESH", |(_, k)| k).to_string()
        })
    }
}

/// `ctx.space` → the active object in the workspace's mode.
pub struct BWorkspaceModeRule;

impl BRule for BWorkspaceModeRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == CtxKey::Space.path()
    }
    fn after(&self, _: &BPath, value: &str, state: &mut BState) {
        let Some(space) = BSpaceType::from_slug(value) else { return };
        let mode = BWorkspaceMode::of(space);
        let object = state.get(CtxKey::Object).filter(|o| !o.is_empty()).map(str::to_string);
        let takes = match &object {
            Some(object) => BWorkspaceMode::takes(&BWorkspaceMode::kind(state, object), mode),
            None => mode == V_MODE_OBJECT,
        };
        if takes {
            state.set(CtxKey::Mode, mode);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspaces_carry_blenders_modes_and_objects_take_only_their_own() {
        assert_eq!(BWorkspaceMode::of(BSpaceType::UvEditing), "Edit Mode");
        assert_eq!(BWorkspaceMode::of(BSpaceType::Layout), V_MODE_OBJECT);
        assert!(BWorkspaceMode::takes("MESH", "Sculpt Mode") && !BWorkspaceMode::takes("CAMERA", "Edit Mode"));
        assert!(!BWorkspaceMode::takes("MESH", "Pose Mode"), "a mesh stays as it was in Animation");
    }
}
