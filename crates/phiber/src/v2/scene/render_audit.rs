//! What a render is actually given, setting by setting — the trace log's
//! answer to "is this control working, or only drawn?".
//!
//! At every render's start the journal gets one line per Render-tab
//! property (`part::props::render::specs`): the value in force, where it
//! came from (the file or an edit, or the default when nothing set it),
//! Cycles' own default (`intern/cycles/blender/addon/properties.py`), and
//! whether Phiray reads it. A control that is drawn but not read is listed
//! as **ignored**, so a render that differs from Blender's names the
//! settings that could explain it instead of leaving them assumed.

use crate::v2::part::props::prop::{BDefault, BPropSpec};
use crate::v2::state::{BState, ButsKey};

/// The Render-tab keys Phiray reads, and where (`render_set.rs` unless
/// said). Everything else on the tab is drawn and saved but not rendered.
pub const APPLIED: &[&str] = &[
    "engine",
    "cycles.samples",
    "cycles.preview_samples",
    "cycles.use_adaptive_sampling",
    "cycles.adaptive_threshold",
    "cycles.adaptive_min_samples",
    "cycles.time_limit",
    "cycles.use_denoising",
    "cycles.use_preview_denoising",
    "cycles.preview_denoising_start_sample",
    "cycles.diffuse_bounces",
    // `rendered/world.rs`'s `lit`, onto `Camera::depth`.
    "cycles.max_bounces",
    "cycles.glossy_bounces",
    "cycles.transmission_bounces",
    "cycles.transparent_max_bounces",
    // `rendered/world.rs`'s `lit`, onto `Camera::limit`.
    "cycles.sample_clamp_direct",
    "cycles.sample_clamp_indirect",
    // `rendered/world.rs`'s `lit`, onto `Camera::integrator`.
    "cycles.blur_glossy",
    "cycles.caustics_reflective",
    "cycles.caustics_refractive",
    "cycles.pixel_filter_type",
    "cycles.filter_width",
    // The card Phiray draws on (`part::pref::device`).
    "cycles.device",
    "render.film_transparent",
    // `BColorManagement`.
    "view_settings.view_transform",
    "view_settings.look",
    "view_settings.exposure",
    "view_settings.gamma",
];

/// Render-tab keys that have nothing to act on in Phiray, and why — kept
/// apart from the ignored ones, which Phiray *could* honour and does not
/// yet. A GPU tracer has no CPU threads to count; it keeps the scene between
/// renders whatever Persistent Data says; it has one denoiser of its own;
/// and it draws no volumes.
pub const NOT_APPLICABLE: &[(&str, &str)] = &[
    ("render.threads_mode", "a GPU render has no CPU threads"),
    ("render.threads", "a GPU render has no CPU threads"),
    ("render.use_persistent_data", "the scene is always kept between renders"),
    ("cycles.denoiser", "Phiray has one denoiser, its own"),
    ("cycles.volume_bounces", "Phiray draws no volumes"),
    ("cycles.volume_step_rate", "Phiray draws no volumes"),
    ("cycles.volume_preview_step_rate", "Phiray draws no volumes"),
    ("cycles.volume_max_steps", "Phiray draws no volumes"),
];

/// Whether Phiray reads a Render-tab key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BAudited {
    Applied,
    /// Nothing in Phiray for it to act on, and why.
    NotApplicable(&'static str),
    Ignored,
}

impl BAudited {
    pub fn of(key: &str) -> Self {
        match (APPLIED.contains(&key), NOT_APPLICABLE.iter().find(|(k, _)| *k == key)) {
            (true, _) => Self::Applied,
            (false, Some((_, why))) => Self::NotApplicable(why),
            (false, None) => Self::Ignored,
        }
    }

    fn word(self) -> String {
        match self {
            Self::Applied => "applied".to_string(),
            Self::NotApplicable(why) => format!("n/a — {why}"),
            Self::Ignored => "IGNORED by Phiray".to_string(),
        }
    }
}

pub struct BRenderAudit;

impl BRenderAudit {
    /// One line per property, in panel order.
    pub fn lines(s: &BState) -> Vec<String> {
        crate::v2::part::props::render::specs().into_iter().map(|spec| Self::line(s, spec)).collect()
    }

    fn line(s: &BState, spec: &BPropSpec) -> String {
        let default = spec.default.map_or_else(|| "(the control's own)".to_string(), Self::shown);
        let (value, from) = match s.get(ButsKey::Value(spec.key)) {
            Some(v) => (v.to_string(), "set"),
            None => (spec.default.map_or_else(|| "-".to_string(), Self::shown), "default"),
        };
        let used = BAudited::of(spec.key).word();
        format!("  {:<40} {:>12} ({from}) | Cycles default {default} | {used}", spec.key, value)
    }

    fn shown(d: BDefault) -> String {
        match d {
            BDefault::Bool(b) => b.to_string(),
            BDefault::Int(i) => i.to_string(),
            BDefault::Float(f) => format!("{f}"),
            BDefault::Enum(e) => e.to_string(),
        }
    }

    /// The same as key/value pairs for the render's Status event
    /// (`state::event`), split the way the question is asked: the settings
    /// Phiray reads, then the ones it does not. A value read from the file
    /// or an edit says `(set)`; one left at its default says nothing more —
    /// `cycles.samples` → `256 (set)`, `cycles.glossy_bounces` → `4`.
    /// Settings that have nothing to act on in Phiray are not among the
    /// unread: they go with the read ones, their value saying why.
    pub fn pairs(s: &BState) -> (Vec<(String, String)>, Vec<(String, String)>) {
        let (mut read, mut ignored) = (Vec::new(), Vec::new());
        for spec in crate::v2::part::props::render::specs() {
            let value = match s.get(ButsKey::Value(spec.key)) {
                Some(v) => format!("{v} (set)"),
                None => spec.default.map_or_else(|| "-".to_string(), Self::shown),
            };
            match BAudited::of(spec.key) {
                BAudited::Applied => read.push((spec.key.to_string(), value)),
                BAudited::NotApplicable(why) => read.push((spec.key.to_string(), format!("{value} — n/a, {why}"))),
                BAudited::Ignored => ignored.push((spec.key.to_string(), value)),
            }
        }
        (read, ignored)
    }

    /// The block, into the session journal.
    pub fn note(s: &BState) {
        let lines = Self::lines(s);
        let ignored = lines.iter().filter(|l| l.ends_with("IGNORED by Phiray")).count();
        crate::v2::app::journal::BJournal::note(format!("Render settings — {} on the Render tab, {ignored} not read by Phiray:\n{}", lines.len(), lines.join("\n")));
    }

    /// The Status event a render reports to (`state::event`): what was
    /// asked for, the settings it was given and, when the cast had to say
    /// something, that. The render's own thread fills in how far it got
    /// and how it ended (`render_job::report_to`).
    pub fn open_event(s: &BState, frame: Option<f32>, size: (u32, u32), samples: u32, objects: usize, cast_note: Option<&str>) -> u64 {
        use crate::v2::state::event::BEvents;
        let event = BEvents::open(
            None,
            "Render Image",
            vec![
                ("Frame".into(), frame.map_or_else(|| "-".to_string(), |f| f.to_string())),
                ("Size".into(), format!("{} × {}", size.0, size.1)),
                ("Samples".into(), samples.to_string()),
                ("Objects".into(), objects.to_string()),
                ("Status".into(), "Rendering".into()),
            ],
        );
        let (read, ignored) = Self::pairs(s);
        BEvents::open(Some(event), "Settings read", read);
        BEvents::open(Some(event), "Settings not read by Phiray", ignored);
        if let Some(note) = cast_note {
            BEvents::open(Some(event), "Render cast", vec![("Note".into(), note.to_string())]);
        }
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_applied_key_is_a_render_tab_property() {
        for key in APPLIED.iter().chain(NOT_APPLICABLE.iter().map(|(k, _)| k)) {
            assert!(crate::v2::part::props::render::spec_of(key).is_some(), "{key} is not on the Render tab");
        }
    }

    #[test]
    fn a_file_value_is_marked_set_and_an_unread_control_is_called_ignored() {
        let mut s = BState::new();
        s.set(ButsKey::Value("cycles.samples"), "256");
        let lines = BRenderAudit::lines(&s);
        let samples = lines.iter().find(|l| l.contains("cycles.samples ")).expect("samples");
        assert!(samples.contains("256 (set)") && samples.contains("Cycles default 4096") && samples.ends_with("applied"), "{samples}");
        let blur = lines.iter().find(|l| l.contains("render.motion_blur_shutter")).expect("motion blur");
        assert!(blur.contains("0.5 (default)") && blur.ends_with("IGNORED by Phiray"), "{blur}");
        let threads = lines.iter().find(|l| l.contains("render.threads ")).expect("threads");
        assert!(threads.ends_with("n/a — a GPU render has no CPU threads"), "{threads}");
        // The event's pairs say the same, split by whether Phiray reads it.
        let (read, ignored) = BRenderAudit::pairs(&s);
        assert!(read.contains(&("cycles.samples".to_string(), "256 (set)".to_string())), "{read:?}");
        assert!(ignored.contains(&("render.motion_blur_shutter".to_string(), "0.5".to_string())), "{ignored:?}");
        assert!(!ignored.iter().any(|(k, _)| k == "render.threads"), "nothing to act on is not 'not read'");
    }
}
