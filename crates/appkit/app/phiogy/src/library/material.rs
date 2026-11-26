//! The kit's materials in the library (appkit `asset/material.tsv`): each
//! drawn once as a ball in the film, the inspector's tile for it, and the
//! one a part of a model wears where the person chose one.

use std::rc::Rc;

use appkit::pui::core::orbit::{ROOM, shot};
use appkit::pui::core::{PCamera, PMaterial, PPiece};
use appkit::service::PFolder;
use appkit::theme::PTheme;

use super::{Library, three};

/// Where the kit keeps its materials, under its shared assets.
const TABLE: &str = "material.tsv";

/// The kit's materials, as its table has them.
pub fn read() -> Vec<Rc<PMaterial>> {
    let text = std::fs::read_to_string(std::path::Path::new(PFolder::shared()).join(TABLE)).unwrap_or_default();
    PMaterial::table(&text).into_iter().map(Rc::new).collect()
}

impl Library {
    /// The material of that name.
    pub fn material(&self, name: &str) -> Option<Rc<PMaterial>> {
        self.material.iter().find(|m| m.name == name).cloned()
    }

    /// The picture of a material's ball in the film, drawn the first time
    /// it is asked for: its name there.
    pub fn ball(&self, material: &PMaterial, theme: &dyn PTheme) -> String {
        let name = format!("material/{}", material.name);
        if self.shot.borrow().contains(&name) {
            return name;
        }
        let side = self.set.get("tile").parse().unwrap_or(192);
        let (triangle, normal) = PMaterial::ball(self.set.get("ball.step").parse().unwrap_or(64));
        let piece = [PPiece { triangle, normal, colour: material.colour, material: Some(Rc::new(material.clone())) }];
        let fov: f32 = self.set.get("camera.fov").parse().unwrap_or(30.0);
        let back = 0.5 * theme.size(ROOM).unwrap_or(1.0) / (fov.to_radians() / 2.0).tan();
        let camera = PCamera { eye: [0.0, back * 0.15, back], at: [0.0; 3], up: [0.0, 1.0, 0.0], fov, near: 0.01 };
        let sun = appkit::pui::core::PSun { way: three(&self.set, "sun.way").map(|v| -v), key: self.set.get("sun.key").parse().unwrap_or(1.0), sky: three(&self.set, "light.sky"), ground: three(&self.set, "light.ground"), shine: 0.0, gloss: 1.0 };
        let (wide, high, pixel) = shot(&piece, &camera, &sun, (side, side), theme);
        self.film.show(&name, wide, high, pixel);
        self.shot.borrow_mut().insert(name.clone());
        name
    }
}
