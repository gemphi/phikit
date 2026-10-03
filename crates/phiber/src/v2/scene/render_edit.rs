//! What the Render and Output tabs changed since the file was opened, for
//! Ctrl S: the engine, the Cycles settings the panel draws, and where a
//! render goes and in what format. Seeded from the file (`app/seed.rs`),
//! read by the tracer (`render_set.rs`) and the saves (`output_set.rs`),
//! these were the last panel values a save left behind.
//!
//! A Cycles value is an ID property on `scene["cycles"]`, and Blender
//! writes only the ones that differ from the RNA default; so a panel value
//! is an edit when it differs from the file's value, or, where the file has
//! none, from the property's default (`BPropSpec::default`). A value the
//! panel never set is not an edit. Enumerations (the denoiser, the pixel
//! filter) are strings, which the writer does not add yet; the device is
//! phiber's own (`pref/device.rs`), not the file's.

use crate::phiphx::blend::{BIdValue, BlendOutput};
use crate::phiphx::scene::SceneModel;
use crate::v2::part::props::prop::{BDefault, BPropKind, BPropSpec};
use crate::v2::state::key::ButsKey;
use crate::v2::state::val::{BValue, BValueKind};
use crate::v2::state::BState;

/// Blender stores an add-on's enumeration as the item's number
/// (`EnumProperty`), so the panel's identifier and the file's value meet
/// here: the Cycles denoiser (OptiX 2, OpenImageDenoise 4) and pixel filter
/// (Box 0, Gaussian 1, Blackman-Harris 2), from `cycles/blender/addon/
/// properties.py`. The device is phiber's own and not written.
pub struct BCyclesEnum;

impl BCyclesEnum {
    const TABLE: &'static [(&'static str, &'static [(&'static str, i32)])] = &[
        ("denoiser", &[("OPTIX", 2), ("OPENIMAGEDENOISE", 4)]),
        ("pixel_filter_type", &[("BOX", 0), ("GAUSSIAN", 1), ("BLACKMAN_HARRIS", 2)]),
    ];

    /// The file's number for the panel's `identifier` of `name`.
    pub fn value(name: &str, identifier: &str) -> Option<i32> {
        Self::TABLE.iter().find(|(n, _)| *n == name)?.1.iter().find(|(id, _)| *id == identifier).map(|(_, v)| *v)
    }

    /// The panel's identifier for the file's `value` of `name`.
    pub fn name(name: &str, value: i32) -> Option<&'static str> {
        Self::TABLE.iter().find(|(n, _)| *n == name)?.1.iter().find(|(_, v)| *v == value).map(|(id, _)| *id)
    }
}

/// The Render and Output edits a save writes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BRenderEdits {
    pub engine: Option<String>,
    pub film_transparent: Option<bool>,
    pub cycles: Vec<(String, BIdValue)>,
    pub output: Option<BlendOutput>,
}

impl BRenderEdits {
    /// `edits` with these three fields set from here.
    pub fn over(self, edits: crate::phiphx::blend::write::BlendEdits) -> crate::phiphx::blend::write::BlendEdits {
        crate::phiphx::blend::write::BlendEdits { engine: self.engine, film_transparent: self.film_transparent, cycles: self.cycles, output: self.output, ..edits }
    }

    /// The panel's values that differ from `model`'s file.
    pub fn of(s: &BState, model: &SceneModel) -> Self {
        let engine = s.get(ButsKey::Value("engine")).filter(|e| !e.is_empty() && *e != model.render.engine).map(str::to_string);
        let film_transparent = s.get(ButsKey::Value("render.film_transparent")).and_then(|v| BValue::parse(BValueKind::Bool, v)).and_then(|v| match v {
            BValue::Bool(b) if b != model.render.film_transparent => Some(b),
            _ => None,
        });
        let cycles = crate::v2::part::props::render::specs()
            .into_iter()
            .filter_map(|spec| Some((spec.key.strip_prefix("cycles.")?, spec)))
            .filter_map(|(name, spec)| Self::cycles_edit(s, model, name, spec))
            .collect();
        // The panel holds the path, format, depth and channels; the rest of
        // the struct stays the file's.
        let file = &model.render.output;
        let panel = BlendOutput { quality: file.quality, compress: file.compress, exr_codec: file.exr_codec, ..crate::v2::scene::output_set(s) };
        let output = Some(panel).filter(|o| *o != *file);
        Self { engine, film_transparent, cycles, output }
    }

    /// `name`'s panel value, when it differs from the file's (or, with none
    /// there, the property's default).
    fn cycles_edit(s: &BState, model: &SceneModel, name: &str, spec: &BPropSpec) -> Option<(String, BIdValue)> {
        let kind = match spec.kind {
            BPropKind::Bool => BValueKind::Bool,
            BPropKind::Int { .. } => BValueKind::Int,
            BPropKind::Float { .. } => BValueKind::Float,
            BPropKind::Enum(_) => return Self::enum_edit(s, model, name, spec),
            _ => return None,
        };
        let now = BValue::parse(kind, s.get(ButsKey::Value(spec.key))?)?;
        let file = model.render.cycles.get(name);
        let same = match (&now, file) {
            (BValue::Bool(b), Some(v)) => v.as_bool() == Some(*b),
            (BValue::Int(i), Some(v)) => v.as_int().map(i64::from) == Some(*i),
            (BValue::Float(f), Some(v)) => v.as_float().is_some_and(|x| (x as f32 - *f).abs() < 1e-6),
            (BValue::Bool(b), None) => spec.default == Some(BDefault::Bool(*b)) || (spec.default.is_none() && !*b),
            (BValue::Int(i), None) => spec.default == Some(BDefault::Int(*i)) || (spec.default.is_none() && *i == 0),
            (BValue::Float(f), None) => matches!(spec.default, Some(BDefault::Float(d)) if (d - *f).abs() < 1e-6) || (spec.default.is_none() && *f == 0.0),
            _ => true,
        };
        if same {
            return None;
        }
        let value = match now {
            BValue::Bool(b) => BIdValue::Bool(b),
            BValue::Int(i) => BIdValue::Int(i as i32),
            BValue::Float(f) => BIdValue::Float(f64::from(f)),
            _ => return None,
        };
        Some((name.to_string(), value))
    }

    /// An enumeration's panel value as the file's number, when it differs
    /// from the file's (or, with none there, the default's).
    fn enum_edit(s: &BState, model: &SceneModel, name: &str, spec: &BPropSpec) -> Option<(String, BIdValue)> {
        let picked = s.get(ButsKey::Value(spec.key)).filter(|v| !v.is_empty())?;
        let value = BCyclesEnum::value(name, picked)?;
        let file = match model.render.cycles.get(name).and_then(|v| v.as_int()) {
            Some(v) => Some(v),
            None => match spec.default {
                Some(BDefault::Enum(d)) => BCyclesEnum::value(name, d),
                _ => None,
            },
        };
        (file != Some(value)).then(|| (name.to_string(), BIdValue::Int(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> SceneModel {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/blend/startup-5.blend");
        let scene = crate::phiphx::blend::BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
        SceneModel::from_blend_owned(scene, &path.to_string_lossy())
    }

    #[test]
    fn only_what_the_panel_changed_is_an_edit() {
        let model = model();
        let mut s = BState::new();
        crate::v2::app::seed::seed_render_props(&mut s, &model);
        crate::v2::app::seed::seed_output_props(&mut s, &model);
        assert_eq!(BRenderEdits::of(&s, &model), BRenderEdits::default(), "as seeded, nothing to write");
        // Max Samples typed, Denoise unticked (its default is on, so the
        // file carries nothing for it), the engine picked, the path typed.
        s.set(ButsKey::Value("cycles.samples"), BValue::Int(48).encode());
        s.set(ButsKey::Value("cycles.use_denoising"), BValue::Bool(false).encode());
        s.set(ButsKey::Value("cycles.diffuse_bounces"), BValue::Int(4).encode());
        s.set(ButsKey::Value("engine"), "CYCLES");
        s.set(ButsKey::Value("filepath"), "//out/frame_");
        s.set(ButsKey::Value("render.film_transparent"), BValue::Bool(true).encode());
        s.set(ButsKey::Value("cycles.pixel_filter_type"), BValue::Enum("GAUSSIAN".into()).encode());
        s.set(ButsKey::Value("cycles.denoiser"), BValue::Enum("OPENIMAGEDENOISE".into()).encode());
        let edits = BRenderEdits::of(&s, &model);
        assert_eq!(edits.engine.as_deref(), Some("CYCLES"));
        assert_eq!(edits.film_transparent, Some(true));
        assert_eq!(edits.cycles, [("samples".to_string(), BIdValue::Int(48)), ("use_denoising".to_string(), BIdValue::Bool(false)), ("pixel_filter_type".to_string(), BIdValue::Int(1))], "Diffuse at its default of 4 and the denoiser at its default are not edits; Gaussian is Blender's 1");
        assert_eq!(edits.output.as_ref().map(|o| o.path.as_str()), Some("//out/frame_"));
        assert_eq!(edits.output.as_ref().map(|o| o.format), Some(model.render.output.format), "the rest of Output stays the file's");
    }
}
