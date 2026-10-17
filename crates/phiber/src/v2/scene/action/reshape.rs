//! Keys the editors reshaped — `ACTION_OT_interpolation_type` /
//! `_easing_type` / `_handle_type` (the Dope Sheet's and Graph editor's Key ▸
//! Interpolation, Easing and Handle Type), kept in the same ordered list as
//! the inserts and moves (`ctx.scene.keyed`), so a reshape replays over the
//! file's curves in the order it was made, Undo steps it, and a save writes
//! the keys it changed.
//!
//! A reshape names an action, the slot an object is animated through, and the
//! frames of the keys it touched, and sets one of a key's shape properties:
//! its interpolation to the next key (`BezTriple.ipo`), the easing of an
//! easing interpolation (`BezTriple.easing`), or both handle types
//! (`BezTriple.h1 / h2`). Only the keys standing at those frames on the
//! slot's curves change; the rest of the curve is left as it was.

use crate::phiphx::blend::{
    EASE_AUTO, EASE_IN, EASE_IN_OUT, EASE_OUT, HANDLE_ALIGN, HANDLE_AUTO, HANDLE_AUTO_ANIM, HANDLE_FREE, HANDLE_VECT,
    IPO_BACK, IPO_BEZ, IPO_BOUNCE, IPO_CIRC, IPO_CONST, IPO_CUBIC, IPO_ELASTIC, IPO_EXPO, IPO_LIN, IPO_QUAD, IPO_QUART,
    IPO_QUINT, IPO_SINE,
};
use crate::phiphx::scene::SceneAction;

/// The shape change a reshape makes — exactly one is set by a menu pick, but
/// the three are independent so a future combined edit can set more at once.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BKeyShape {
    /// `BezTriple.ipo` — the interpolation to the next key.
    pub ipo: Option<u8>,
    /// `BezTriple.easing` — the easing of an easing interpolation.
    pub easing: Option<u8>,
    /// Both handle types (`BezTriple.h1` and `.h2`).
    pub handle: Option<u8>,
}

impl BKeyShape {
    pub fn is_empty(self) -> bool {
        self.ipo.is_none() && self.easing.is_none() && self.handle.is_none()
    }

    /// The interpolation a menu value names (`NODE_MT`'s `ACTION_OT_interpolation_type`).
    pub fn ipo_of(kind: &str) -> Option<u8> {
        Some(match kind {
            "const" => IPO_CONST,
            "lin" => IPO_LIN,
            "bez" => IPO_BEZ,
            "sine" => IPO_SINE,
            "quad" => IPO_QUAD,
            "cubic" => IPO_CUBIC,
            "quart" => IPO_QUART,
            "quint" => IPO_QUINT,
            "expo" => IPO_EXPO,
            "circ" => IPO_CIRC,
            "back" => IPO_BACK,
            "bounce" => IPO_BOUNCE,
            "elastic" => IPO_ELASTIC,
            _ => return None,
        })
    }

    pub fn easing_of(kind: &str) -> Option<u8> {
        Some(match kind {
            "auto" => EASE_AUTO,
            "in" => EASE_IN,
            "out" => EASE_OUT,
            "inout" => EASE_IN_OUT,
            _ => return None,
        })
    }

    pub fn handle_of(kind: &str) -> Option<u8> {
        Some(match kind {
            "free" => HANDLE_FREE,
            "aligned" => HANDLE_ALIGN,
            "vector" => HANDLE_VECT,
            "auto" => HANDLE_AUTO,
            "auto_clamped" => HANDLE_AUTO_ANIM,
            _ => return None,
        })
    }

    /// The shape a `<field>:<kind>` menu value names: `ipo:lin`, `easing:in`,
    /// `handle:vector`. `None` when the field or the kind is unknown.
    pub fn from_value(field: &str, kind: &str) -> Option<Self> {
        let shape = match field {
            "ipo" => Self { ipo: Some(Self::ipo_of(kind)?), ..Self::default() },
            "easing" => Self { easing: Some(Self::easing_of(kind)?), ..Self::default() },
            "handle" => Self { handle: Some(Self::handle_of(kind)?), ..Self::default() },
            _ => return None,
        };
        Some(shape)
    }
}

/// The keys at `frames` on `action`'s curves through `slot` (all of them when
/// `slot` is empty), reshaped by `shape`.
#[derive(Clone, Debug, PartialEq)]
pub struct BKeyReshape {
    pub action: String,
    pub slot: String,
    pub frames: Vec<f32>,
    pub shape: BKeyShape,
}

/// A reshape's head mark: `^action#slot#frames=ipo,easing,handle`, each field
/// a number or `-` for "leave it".
const MARK: char = '^';
const NONE: &str = "-";

impl BKeyReshape {
    /// Within half a thousandth: the columns are keyed to a thousandth.
    fn hit(&self, x: f32) -> bool {
        self.frames.iter().any(|f| (f - x).abs() < 5e-4)
    }

    pub fn apply(&self, actions: &mut [SceneAction]) {
        let Some(action) = actions.iter_mut().find(|a| a.name == self.action) else { return };
        let run = match self.slot.is_empty() {
            true => 0..action.curves.len(),
            false => match action.slots.iter().find(|s| s.identifier == self.slot) {
                Some(slot) => slot.curves.clone(),
                None => return,
            },
        };
        for curve in &mut action.curves[run] {
            let mut touched = false;
            for key in curve.keys.iter_mut().filter(|k| self.hit(k.co[0])) {
                if let Some(ipo) = self.shape.ipo {
                    key.ipo = ipo;
                }
                if let Some(easing) = self.shape.easing {
                    key.easing = easing;
                }
                if let Some(handle) = self.shape.handle {
                    key.h1_type = handle;
                    key.h2_type = handle;
                }
                touched = true;
            }
            // A handle-type change moves the handles it governs; `recalc` is
            // Blender's `BKE_fcurve_handles_recalc` after the edit.
            if touched && self.shape.handle.is_some() {
                curve.recalc_handles();
            }
        }
    }

    fn num(v: Option<u8>) -> String {
        v.map_or(NONE.to_string(), |n| n.to_string())
    }

    fn opt(part: &str) -> Option<u8> {
        (part != NONE).then(|| part.parse().ok()).flatten()
    }

    pub(super) fn parse(entry: &str) -> Option<Self> {
        let (head, shape) = entry.strip_prefix(MARK)?.rsplit_once('=')?;
        let mut head = head.splitn(3, '#');
        let (action, slot) = (head.next()?.to_string(), head.next()?.to_string());
        let frames = head.next()?.split(',').filter_map(|f| f.parse().ok()).collect();
        let mut field = shape.split(',');
        let shape = BKeyShape {
            ipo: Self::opt(field.next()?),
            easing: Self::opt(field.next()?),
            handle: Self::opt(field.next()?),
        };
        Some(Self { action, slot, frames, shape })
    }

    pub(super) fn encode(&self) -> String {
        let frames: Vec<String> = self.frames.iter().map(f32::to_string).collect();
        format!(
            "{MARK}{}#{}#{}={},{},{}",
            self.action,
            self.slot,
            frames.join(","),
            Self::num(self.shape.ipo),
            Self::num(self.shape.easing),
            Self::num(self.shape.handle),
        )
    }

    pub(super) fn is_mark(entry: &str) -> bool {
        entry.starts_with(MARK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::{BFCurve, BKey, BlendActionSlot};
    use crate::phiphx::scene::SceneAction;

    fn curve_at(frames: &[f32]) -> BFCurve {
        let keys = frames.iter().map(|&x| BKey::new([x - 1.0, 0.0], [x, 0.0], [x + 1.0, 0.0], IPO_BEZ)).collect();
        BFCurve::new("location", 0, keys)
    }

    fn action_with(frames: &[f32]) -> Vec<SceneAction> {
        vec![SceneAction {
            name: "Act".into(),
            slots: vec![BlendActionSlot { handle: 1, identifier: "OBCube".into(), curves: 0..1 }],
            curves: vec![curve_at(frames)],
            ..Default::default()
        }]
    }

    fn reshape(frames: Vec<f32>, shape: BKeyShape) -> BKeyReshape {
        BKeyReshape { action: "Act".into(), slot: "OBCube".into(), frames, shape }
    }

    #[test]
    fn interpolation_is_set_only_on_the_named_keys() {
        let mut actions = action_with(&[1.0, 5.0, 9.0]);
        reshape(vec![5.0], BKeyShape { ipo: Some(IPO_LIN), ..Default::default() }).apply(&mut actions);
        let keys = &actions[0].curves[0].keys;
        assert_eq!(keys[0].ipo, IPO_BEZ, "the untouched key keeps Bézier");
        assert_eq!(keys[1].ipo, IPO_LIN, "the named key is now linear");
        assert_eq!(keys[2].ipo, IPO_BEZ);
    }

    #[test]
    fn easing_and_handle_set_their_own_fields() {
        let mut actions = action_with(&[1.0]);
        reshape(vec![1.0], BKeyShape { easing: Some(EASE_IN), ..Default::default() }).apply(&mut actions);
        assert_eq!(actions[0].curves[0].keys[0].easing, EASE_IN);
        reshape(vec![1.0], BKeyShape { handle: Some(HANDLE_VECT), ..Default::default() }).apply(&mut actions);
        let key = actions[0].curves[0].keys[0];
        assert_eq!((key.h1_type, key.h2_type), (HANDLE_VECT, HANDLE_VECT), "both handles take the type");
    }

    #[test]
    fn a_reshape_round_trips_through_its_encoding() {
        let r = reshape(vec![1.0, 5.0], BKeyShape { ipo: Some(IPO_SINE), easing: Some(EASE_OUT), handle: None });
        let text = r.encode();
        assert!(BKeyReshape::is_mark(&text));
        assert_eq!(BKeyReshape::parse(&text), Some(r));
    }

    #[test]
    fn a_menu_value_names_its_shape() {
        assert_eq!(BKeyShape::from_value("ipo", "lin"), Some(BKeyShape { ipo: Some(IPO_LIN), ..Default::default() }));
        assert_eq!(BKeyShape::from_value("easing", "inout"), Some(BKeyShape { easing: Some(EASE_IN_OUT), ..Default::default() }));
        assert_eq!(BKeyShape::from_value("handle", "vector"), Some(BKeyShape { handle: Some(HANDLE_VECT), ..Default::default() }));
        assert_eq!(BKeyShape::from_value("ipo", "nonsense"), None);
        assert_eq!(BKeyShape::from_value("what", "lin"), None);
    }
}
