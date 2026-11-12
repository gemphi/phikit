// SPDX-FileCopyrightText: 2018-2023 Blender Authors
// SPDX-License-Identifier: Apache-2.0
//! Gate 0, the judge: find a test scene and the image Blender made of it,
//! render it through the session, compare by Blender's own rule, and say
//! pass or fail. Ported from `tests/python/cycles_render_tests.py` and
//! `tests/python/modules/render_report.py`; PLAN.md names the folder
//! `truth/`, which has no Cycles directory of its own.
//!
//! | File | Ported from |
//! |---|---|
//! | `compare.rs` | `render_report.py:125-205` (`oiiotool --diff`) |
//! | `allowance.rs` | `render_report.py:287-288`, `cycles_render_tests.py:337-357` |
//! | `scene.rs` | `render_report.py` (`test_get_images`) |
//! | `image.rs` | what `oiiotool` reads and writes |
//! | `verdict.rs` | `render_report.py` (`diff_output`, the report's `VERIFY`) |
//! | `render.rs` | `session.cpp` (`run_main_render_loop`) |
mod allowance;
mod compare;
mod display;
mod engine;
mod error;
mod image;
mod render;
mod scene;
pub(crate) mod verdict;
pub use allowance::Allowance;
pub use compare::{Comparison, compare, diff_image};
pub use error::TruthError;
pub use image::Image;
pub use display::{ViewSetting, ViewTransform};
pub use engine::{RenderReport, SceneRenderer};
pub use render::{SessionRenderer, display, display_view, srgb_encode};
pub use scene::{TEST_ROOT, TestScene, find_root};
pub use verdict::{Outcome, Renderer, Verdict, judge};
