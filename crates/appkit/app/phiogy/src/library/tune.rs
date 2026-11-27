//! A part's material as the person tuned it in the inspector, as Reality
//! Composer Pro's physically based material is tuned: its roughness, its
//! metal, how often its pattern repeats, how deep its relief, and its
//! second colour (`data/control.tsv`, kept at look/<item>/<part>/<name>).
//! A part given only controls wears a plain material in its own colour.

use std::rc::Rc;

use appkit::pui::core::PMaterial;
use appkit::pui::state::PState;

use super::attribute::{Attribute, paint};
use super::{Item, Library};
use crate::data;

impl Library {
    /// The material a part wears: the one chosen, or a plain one where
    /// only its controls were set, tuned as they say; none where neither.
    pub fn worn(&self, item: &Item, part: usize, attribute: &Attribute) -> Option<Rc<PMaterial>> {
        let chosen = self.material(&attribute.material(&item.key, part));
        let set = |name: &str| attribute.get(&format!("look/{}/{part}/{name}", item.key));
        let named = control_names();
        let touched = named.iter().any(|name| !set(name).is_empty()) || !set("second").is_empty();
        let own = || self.model(&item.path).part.get(part).map(|p| PMaterial::plain(p.piece.colour));
        let mut material = match (chosen, touched) {
            (Some(chosen), _) => (*chosen).clone(),
            (None, true) => own()?,
            (None, false) => return None,
        };
        let number = |name: &str| set(name).parse::<f32>().ok();
        material.rough = number("rough").unwrap_or(material.rough);
        material.metal = number("metal").unwrap_or(material.metal);
        material.scale *= number("tiling").unwrap_or(1.0);
        material.depth *= number("relief").unwrap_or(1.0);
        material.second = paint(&set("second")).unwrap_or(material.second);
        Some(Rc::new(material))
    }
}

/// The controls' names, as the table has them.
pub(super) fn control_names() -> Vec<String> {
    data::row(data::CONTROL).into_iter().filter_map(|cell| cell.first().map(|name| name.to_string())).collect()
}

/// The person's choices for the library's forms, a control that was not
/// set standing where the part has it: its material's own, or the table's
/// number.
pub struct Tune {
    pub attribute: Rc<Attribute>,
    pub library: Rc<Library>,
}

impl PState for Tune {
    fn get(&self, path: &str) -> String {
        let kept = self.attribute.get(path);
        let mut step = path.rsplitn(3, '/');
        let (Some(name), Some(part), Some(rest)) = (step.next(), step.next(), step.next()) else { return kept };
        let row = data::row(data::CONTROL).into_iter().find(|cell| cell.first() == Some(&name));
        let (Some(row), true) = (row, kept.is_empty()) else { return kept };
        let item = rest.strip_prefix("look/").and_then(|key| self.library.get(key));
        let worn = item.zip(part.parse::<usize>().ok()).and_then(|(item, part)| self.library.worn(&item, part, &self.attribute));
        match (row.get(4).copied(), worn) {
            (Some("own"), Some(material)) => format!("{:.2}", match name {
                "metal" => material.metal,
                _ => material.rough,
            }),
            (Some("own"), None) => String::new(),
            (without, _) => without.unwrap_or_default().to_string(),
        }
    }

    fn set(&self, path: &str, value: &str) {
        self.attribute.set(path, value);
    }
}
