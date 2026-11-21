// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.cpp`
//! (`RenderScheduler::full_report`, lines 630-745). The report's words are
//! `data/report.tsv`.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::OnceLock;

use super::{RenderScheduler, TimeAverage};
use crate::device::DenoiserPass;
use crate::util::{Clock, row_list};

/// The report's text for `key`.
fn text(key: &str) -> &'static str {
    static TABLE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let read = || row_list(include_str!("../../../data/report.tsv")).filter_map(|row| Some((*row.first()?, *row.get(1)?))).collect();
    TABLE.get_or_init(read).get(key).copied().unwrap_or("")
}

/// A section's heading.
fn heading(key: &str) -> String {
    format!("\n{}:\n", text(key))
}

/// One indented `name: value` line.
fn field(key: &str, value: impl Display) -> String {
    format!("  {}: {}\n", text(key), value)
}

/// `string_from_bool`.
fn flag(value: bool) -> &'static str {
    text(match value {
        true => "true",
        false => "false",
    })
}

/// One row of the time table: wall time and average.
fn time_row(key: &str, time: &TimeAverage) -> String {
    format!("  {:>20} {:>20.6} {:>20.6}\n", text(key), time.wall(), time.average())
}

impl<C: Clock> RenderScheduler<C> {
    /// `full_report`: the settings and the times of the render, on many
    /// lines.
    pub fn full_report(&self) -> String {
        let (time, adaptive, denoise) = (&self.task_time, &self.adaptive_sampling, &self.denoise_param);
        let wall_time = self.state.end_render_time - self.state.start_render_time;
        let rebalanced = self.state.rebalance_request_count != 0;
        let mode = match (self.headless, self.background) {
            (true, _) => "mode.headless",
            (false, true) => "mode.background",
            (false, false) => "mode.interactive",
        };
        let mut line = vec![format!("\n{}\n\n", text("title"))];
        line.push(format!("{}: {}\n", text("mode"), text(mode)));
        line.push(format!("{}: {}x{}\n", text("resolution"), self.buffer_param.width, self.buffer_param.height));

        line.push(heading("adaptive"));
        line.push(field("use", flag(adaptive.enabled)));
        if adaptive.enabled {
            line.push(field("adaptive.step", adaptive.adaptive_step));
            line.push(field("adaptive.min", adaptive.min_sample));
            line.push(field("adaptive.threshold", format!("{:.6}", adaptive.threshold)));
        }

        line.push(heading("denoiser"));
        line.push(field("use", flag(denoise.enabled)));
        if denoise.enabled {
            let mut pass = vec![text("denoiser.color")];
            pass.extend(DenoiserPass::label_list(denoise.pass_mask));
            line.push(field("denoiser.type", denoise.denoiser.label()));
            line.push(field("denoiser.start", denoise.start_sample));
            line.push(field("denoiser.pass", pass.join(", ")));
        }

        if rebalanced {
            line.push(heading("rebalance"));
            line.push(field("rebalance.request", self.state.rebalance_request_count));
            line.push(field("rebalance.change", self.state.rebalance_change_count));
        }

        line.push(heading("time"));
        line.push(format!("  {:>20} {:>20} {:>20}\n", "", text("time.wall"), text("time.average")));
        line.push(time_row("time.path_trace", &time.path_trace));
        if adaptive.enabled {
            line.push(time_row("time.adaptive", &time.adaptive_filter));
        }
        if denoise.enabled {
            line.push(time_row("time.denoise", &time.denoise));
        }
        line.push(time_row("time.display", &time.display_update));
        if rebalanced {
            line.push(time_row("time.rebalance", &time.rebalance));
        }
        let total = time.path_trace.wall() + time.adaptive_filter.wall() + time.denoise.wall() + time.display_update.wall();
        line.push(format!("\n  {}: {:.6}\n", text("total"), total));

        let count = self.rendered_sample_count();
        let rendered = text("rendered").replace("{sample}", &count.to_string());
        line.push(format!("\n{}\n", rendered.replace("{second}", &format!("{wall_time:.6}"))));
        // With adaptive sampling a sample covers a varying number of
        // pixels, so a time for each means nothing.
        if !adaptive.enabled {
            let per_sample = format!("{:.6}", wall_time / f64::from(count));
            line.push(format!("{}\n", text("per_sample").replace("{second}", &per_sample)));
        }
        line.concat()
    }
}
