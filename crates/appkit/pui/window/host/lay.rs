//! Where the parts of a host go inside the rectangle its area has.
//!
//! The regions are stacked from the top. A region the theme gives a
//! height (`region.header`) takes it; the others share what is left. In a
//! region the panes stand left, centre, right: a side pane takes the
//! theme's `pane.side` when there is a centre beside it, and the centre
//! what is left; without a centre the panes share the width. The parts of
//! a pane are stacked in it.

use super::{PHost, PLevel};
use crate::pui::core::PRect;
use crate::pui::window::{PRegion, PSide};
use crate::theme::PTheme;

/// Where one part of a host is shown.
#[derive(Debug, Clone, PartialEq)]
pub struct PSlot {
    pub host: String,
    pub region: String,
    pub part: String,
    /// Its rectangle, in the coordinates the host's own was given in.
    pub rect: PRect,
}

impl PHost {
    /// Where each part the host declares is shown, inside `rect`.
    pub fn lay(&self, rect: PRect, theme: &dyn PTheme) -> Vec<PSlot> {
        let fixed = |region: &PRegion| theme.size(&format!("region.{}", region.name));
        let taken: f32 = self.region.iter().filter_map(fixed).sum();
        let loose = self.region.iter().filter(|region| fixed(region).is_none()).count().max(1);
        let share = (rect.size.height - taken).max(0.0) / loose as f32;
        let mut top = rect.origin.y;
        let mut out = Vec::new();
        for region in &self.region {
            let height = fixed(region).unwrap_or(share);
            let band = PRect::new(rect.origin.x, top, rect.size.width, height);
            out.extend(self.row(region, band, theme));
            top += height;
        }
        out
    }

    /// The parts of one region, in its band.
    fn row(&self, region: &PRegion, band: PRect, theme: &dyn PTheme) -> Vec<PSlot> {
        let count = region.pane.len().max(1) as f32;
        let centred = region.pane.iter().any(|pane| pane.side == PSide::Center);
        let side = match (centred, theme.size("pane.side")) {
            (true, Some(width)) => width,
            _ => band.size.width / count,
        };
        let edge = region.pane.iter().filter(|pane| pane.side != PSide::Center).count() as f32 * side;
        let mut out = Vec::new();
        for pane in &region.pane {
            let (left, width) = match pane.side {
                PSide::Left => (band.origin.x, side),
                PSide::Right => (band.origin.x + band.size.width - side, side),
                PSide::Center => match region.pane.iter().any(|pane| pane.side == PSide::Left) {
                    true => (band.origin.x + side, band.size.width - edge),
                    false => (band.origin.x, band.size.width - edge),
                },
            };
            let height = band.size.height / pane.part.len().max(1) as f32;
            let slot = |(index, part): (usize, &String)| PSlot {
                host: self.slug().to_string(),
                region: region.name.clone(),
                part: part.clone(),
                rect: PRect::new(left, band.origin.y + index as f32 * height, width, height),
            };
            out.extend(pane.part.iter().enumerate().map(slot));
        }
        out
    }
}
