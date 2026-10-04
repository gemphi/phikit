//! An object's two Shadow switches as the Properties editor flips them —
//! Viewport Display ▸ Shadow (`Object.display.show_shadows`, the negative
//! of `dtx`'s `OB_DRAW_NO_SHADOW_CAST`) and Visibility ▸ Ray Visibility ▸
//! Shadow (`Object.visible_shadow`, the negative of `visibility_flag`'s
//! `OB_HIDE_SHADOW`). One key over the file (`ctx.scene.shadow_cast`,
//! `name=display,ray|…`), stepped by Undo like the other scene edits; the
//! viewport build reads the switch the shading mode wants
//! (`BlendShadowCast::casts_in`) and a save writes both bits back.

use super::BObject3D;
use crate::phiphx::blend::BlendShadowCast;
use crate::v2::state::{BKey as _, BPath, BState, ButsKey, CtxKey, SceneKey, rule::BRule, val::{BValue, BValueKind}};

/// The panel's two checkboxes, and the keys they are drawn from:
/// `OBJECT_PT_display`'s and `OBJECT_PT_visibility`'s.
pub const DISPLAY_FIELD: &str = "show_shadows";
pub const RAY_FIELD: &str = "visible_shadow";

/// Every object the panel switched, by its name now.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BObjectShadowList {
    pub list: Vec<(String, BlendShadowCast)>,
}

impl BObjectShadowList {
    pub fn now(s: &BState) -> Self {
        Self::parse(s.get(SceneKey::ShadowCast).unwrap_or(""))
    }

    /// The list as `ctx.scene.shadow_cast` spells it.
    pub fn parse(raw: &str) -> Self {
        let one = |t: &str| {
            let (d, r) = t.split_once(',')?;
            Some(BlendShadowCast { display: d == "1", ray: r == "1" })
        };
        let list = raw.split('|').filter_map(|e| e.split_once('=').and_then(|(n, t)| one(t).map(|t| (n.to_string(), t)))).collect();
        Self { list }
    }

    pub fn encode(&self) -> String {
        self.list.iter().map(|(n, t)| format!("{n}={},{}", u8::from(t.display), u8::from(t.ray))).collect::<Vec<_>>().join("|")
    }

    pub fn write(&self, s: &mut BState) {
        s.set(SceneKey::ShadowCast, self.encode());
    }

    pub fn get(&self, name: &str) -> Option<BlendShadowCast> {
        self.list.iter().find(|(n, _)| n == name).map(|(_, t)| *t)
    }

    pub fn set(&mut self, name: &str, t: BlendShadowCast) {
        self.list.retain(|(n, _)| n != name);
        self.list.push((name.to_string(), t));
    }

    pub fn forget(&mut self, name: &str) {
        self.list.retain(|(n, _)| n != name);
    }

    /// The panel's two checkboxes as the state holds them, both or none.
    pub fn from_panel(s: &BState) -> Option<BlendShadowCast> {
        let read = |field: &'static str| match s.value(&ButsKey::Value(field).path(), BValueKind::Bool) {
            Some(BValue::Bool(v)) => Some(v),
            _ => None,
        };
        Some(BlendShadowCast { display: read(DISPLAY_FIELD)?, ray: read(RAY_FIELD)? })
    }

    /// The switched objects take their switches over what the file said.
    pub fn apply(&self, objects: &mut [BObject3D]) {
        if self.list.is_empty() {
            return;
        }
        for object in objects.iter_mut() {
            if let BObject3D::Loaded { name, shadow, .. } = object {
                if let Some(t) = self.get(name) {
                    *shadow = t;
                }
            }
        }
    }

    /// One number for the object list's memo.
    pub fn fingerprint(s: &BState) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        s.get(SceneKey::ShadowCast).hash(&mut h);
        h.finish()
    }
}

/// A Shadow checkbox clicked: the active object takes the panel's two.
pub struct BObjectShadowRule;

impl BRule for BObjectShadowRule {
    fn wants(&self, path: &BPath) -> bool {
        matches!(path.segs.as_ref(), ["buts", field] if *field == DISPLAY_FIELD || *field == RAY_FIELD)
    }
    fn after(&self, _: &BPath, _: &str, s: &mut BState) {
        let (Some(active), Some(t)) = (s.get(CtxKey::Object).map(str::to_string), BObjectShadowList::from_panel(s)) else { return };
        let mut all = BObjectShadowList::now(s);
        all.set(&active, t);
        all.write(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_round_trips_and_a_switched_object_takes_its_switches() {
        let mut s = BState::new();
        let mut all = BObjectShadowList::default();
        all.set("Cube", BlendShadowCast { display: false, ray: true });
        all.set("Lamp Post", BlendShadowCast { display: true, ray: false });
        all.write(&mut s);
        assert_eq!(s.get(SceneKey::ShadowCast), Some("Cube=0,1|Lamp Post=1,0"));
        let again = BObjectShadowList::now(&s);
        assert_eq!(again, all);
        assert_eq!(again.get("Cube"), Some(BlendShadowCast { display: false, ray: true }));
        assert_eq!(again.get("Sphere"), None);
        let object = |name: &str| crate::phiphx::scene::SceneObject {
            name: name.to_string(),
            mesh: crate::phiphx::scene::SceneMesh {
                vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                indices: vec![0, 1, 2],
                normals: crate::phiphx::scene::SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
                corners: Default::default(),
            }
            .into(),
            location: [0.0; 3],
            scale: [1.0; 3],
            matrix_world: crate::phiphx::scene::IDENTITY_M4,
            materials: Vec::new(),
            material_indices: Default::default(),
            uvs: Default::default(),
            color: [1.0; 4],
            shadow: Default::default(),
        };
        let model = crate::phiphx::scene::SceneModel { objects: vec![object("Cube"), object("Sphere")], ..Default::default() };
        let mut objects = super::super::current_scene(&s, Some(&model));
        again.apply(&mut objects);
        let shadow = |o: &BObject3D| match o {
            BObject3D::Loaded { shadow, .. } => *shadow,
            BObject3D::Primitive { .. } => unreachable!(),
        };
        assert_eq!(shadow(&objects[0]), BlendShadowCast { display: false, ray: true });
        assert_eq!(shadow(&objects[1]), BlendShadowCast::default(), "an unswitched object keeps the file's");
    }

    #[test]
    fn the_rule_takes_the_panels_checkboxes_for_the_active_object() {
        let mut s = BState::new();
        s.set(CtxKey::Object, "Cube");
        s.set(ButsKey::Value(DISPLAY_FIELD), BValue::Bool(true).encode());
        assert!(BObjectShadowList::from_panel(&s).is_none(), "one checkbox alone is not a pair");
        s.set(ButsKey::Value(RAY_FIELD), BValue::Bool(false).encode());
        let rule = BObjectShadowRule;
        assert!(rule.wants(&ButsKey::Value(RAY_FIELD).path()) && !rule.wants(&ButsKey::Value("show_name").path()));
        rule.after(&ButsKey::Value(RAY_FIELD).path(), "false", &mut s);
        assert_eq!(BObjectShadowList::now(&s).get("Cube"), Some(BlendShadowCast { display: true, ray: false }));
    }
}
