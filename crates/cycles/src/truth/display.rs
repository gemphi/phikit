// SPDX-License-Identifier: Apache-2.0
//! The colour management between the film and the PNG Blender wrote: the
//! scene's view settings (`Scene.view_settings`, `scene.display_settings`)
//! as OpenColorIO applies them for a render saved as 8-bit sRGB. Blender's
//! `Standard` view transform is the display's sRGB curve; the exposure
//! (stops) multiplies the scene-linear value first and the gamma follows
//! the curve (`out = value^(1/gamma)`), as OCIO's `ExposureContrast` and
//! `Gamma` transforms do (`ocio_impl.cc`, `OCIOImpl::createDisplayProcessor`).
//!
//! `Raw` is no curve. `Filmic`, `AgX`, a look, and `Khronos PBR Neutral`
//! are tables of OCIO configuration that are not ported: they draw with the
//! Standard curve and say so in [`ViewSettings::approximated`].
use super::render::srgb_encode;

/// `view_settings.view_transform`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewTransform {
    Standard,
    Raw,
    /// A transform of the OCIO configuration that is not ported, by name.
    Other(String),
}

/// What turns the film into the picture.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewSettings {
    pub transform: ViewTransform,
    /// A look other than `None`.
    pub look: Option<String>,
    pub exposure: f32,
    pub gamma: f32,
    /// `film.exposure` (Cycles' own, `cycles.film_exposure`), which scales
    /// the pass as it is read.
    pub film_exposure: f32,
    /// The buffer is read as Cycles keeps it (see display_view).
    pub cycles_buffer: bool,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self { transform: ViewTransform::Standard, look: None, exposure: 0.0, gamma: 1.0, film_exposure: 1.0, cycles_buffer: false }
    }
}

impl ViewSettings {
    /// The view settings of a scene's names, as phiphx reads them.
    pub fn of(view_transform: &str, look: &str, exposure: f32, gamma: f32, film_exposure: f32) -> ViewSettings {
        let transform = match view_transform {
            // `Default` is what a file that holds no view transform reads as:
            // the display's first view, which is Standard.
            "Standard" | "Default" | "" => ViewTransform::Standard,
            "Raw" => ViewTransform::Raw,
            other => ViewTransform::Other(other.to_string()),
        };
        let look = match look {
            "None" | "" => None,
            other => Some(other.to_string()),
        };
        ViewSettings { transform, look, exposure, gamma: if gamma > 0.0 { gamma } else { 1.0 }, film_exposure, cycles_buffer: true }
    }

    /// Whether the picture is not what Blender's: a transform or a look the
    /// port does not have.
    pub fn approximated(&self) -> bool {
        matches!(self.transform, ViewTransform::Other(_)) || self.look.is_some()
    }

    /// One scene-linear channel (already divided by the sample count) as
    /// the display shows it, 0 to 1.
    pub fn apply(&self, linear: f32) -> f32 {
        let scaled = linear * self.film_exposure * 2f32.powf(self.exposure);
        let curved = match self.transform {
            ViewTransform::Raw => scaled.clamp(0.0, 1.0),
            _ => srgb_encode(scaled),
        };
        match self.gamma == 1.0 {
            true => curved,
            false => curved.powf(1.0 / self.gamma),
        }
    }
}
