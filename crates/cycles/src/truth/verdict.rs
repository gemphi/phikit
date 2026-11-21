// SPDX-FileCopyrightText: 2018-2023 Blender Authors
// SPDX-License-Identifier: Apache-2.0
//! Ported from `tests/python/modules/render_report.py` (`diff_output`,
//! lines 125-210; `test.error = "VERIFY"` for a failure or a missing
//! reference): a scene in, a verdict out, and the diff images next to it.
use std::path::{Path, PathBuf};

use super::{Allowance, Comparison, Image, TestScene, TruthError, compare, diff_image};

/// A function from a scene to the image the engine made of it.
pub trait Renderer {
    fn render(&mut self, scene: &TestScene) -> Result<Image, TruthError>;
}

/// How a test ended.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// Within the allowance.
    Pass,
    /// Past it (`VERIFY` in Blender's report).
    Fail,
    /// There is no reference to compare with (`VERIFY` with no update).
    NoReference,
    /// The engine made no image: the message.
    RenderFailed(String),
}

/// What the judge says of one scene.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub category: String,
    pub name: String,
    pub allowance: Allowance,
    pub outcome: Outcome,
    pub comparison: Option<Comparison>,
    /// The colour diff, `|ref - render| * 16`, written when the scene was
    /// compared (`diff_color_img`), and the alpha one when both have alpha.
    pub diff_color: Option<PathBuf>,
    pub diff_alpha: Option<PathBuf>,
}

impl Verdict {
    pub fn passed(&self) -> bool {
        self.outcome == Outcome::Pass
    }
}

/// Render `scene` with `renderer` and judge it against its reference under
/// the allowance of its category. The diff images go to `out`.
pub fn judge(scene: &TestScene, renderer: &mut impl Renderer, out: &Path) -> Verdict {
    let allowance = Allowance::of_category(&scene.category, false);
    let mut verdict = Verdict { category: scene.category.clone(), name: scene.name.clone(), allowance, outcome: Outcome::NoReference, comparison: None, diff_color: None, diff_alpha: None };
    if !scene.reference.is_file() {
        return verdict;
    }
    let result = Image::read(&scene.reference).and_then(|reference| renderer.render(scene).map(|render| (reference, render)));
    let (reference, render) = match result {
        Ok(pair) => pair,
        Err(error) => {
            verdict.outcome = Outcome::RenderFailed(error.to_string());
            return verdict;
        }
    };
    match compare(&reference, &render, allowance) {
        Ok(comparison) => {
            verdict.outcome = if comparison.failed { Outcome::Fail } else { Outcome::Pass };
            verdict.comparison = Some(comparison);
        }
        Err(error) => {
            verdict.outcome = Outcome::RenderFailed(error.to_string());
            return verdict;
        }
    }
    let stem = format!("{}_{}", scene.category, scene.name);
    let write = |channel: &[usize], suffix: &str| -> Option<PathBuf> {
        let path = out.join(format!("{stem}.{suffix}.png"));
        diff_image(&reference, &render, channel).and_then(|image| image.write_png(&path)).ok().map(|()| path)
    };
    verdict.diff_color = write(&[0, 1, 2], "diff");
    if reference.channel_count == 4 && render.channel_count == 4 {
        verdict.diff_alpha = write(&[3], "diff_alpha");
    }
    verdict
}
