//! Custom host canvas view — host-specific content.

pub mod canvas;
pub use canvas::CustomCanvas;

use super::super::{BState, BView};
use crate::store::hosts::BHostType;

pub struct CustomView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> CustomView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        use crate::bui::state::BKey;
        use crate::store::parts::custom::def::BContent;

        // Viewport camera comes from runtime state (orbit/zoom), defaults match Blender.
        let cam = |k: &BKey, d: f32| {
            self.state
                .get(k)
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(d)
        };
        let camera = (
            cam(&BKey::CamYaw, 45.0),
            cam(&BKey::CamPitch, 30.0),
            cam(&BKey::CamZoom, 1.0),
        );

        fn node(c: &BContent, camera: (f32, f32, f32)) -> BView {
            match c {
                BContent::Empty => BView::empty(),
                BContent::Spatial {
                    name,
                    objects,
                    vertices,
                } => BView::spatial(None, *objects, *vertices, *name)
                    .with_camera(camera.0, camera.1, camera.2),
                BContent::Label { text, icon } => match icon {
                    Some(i) => BView::label_icon(*text, *i),
                    None => BView::label(*text),
                },
                BContent::PropRow { label, value } => BView::prop_row(*label, *value),
                BContent::Item {
                    label,
                    icon,
                    active,
                } => {
                    let v = BView::item(*label).with_icon(*icon);
                    if *active { v.with_active(true) } else { v }
                }
                BContent::Section {
                    label,
                    icon,
                    expanded,
                    items,
                } => BView::section(
                    *label,
                    icon.map(|i| i.to_string()),
                    *expanded,
                    items.iter().map(|c| node(c, camera)).collect(),
                ),
                BContent::Separator => BView::separator(),
                BContent::Scroll(children) => {
                    BView::scroll(children.iter().map(|c| node(c, camera)).collect())
                }
                BContent::Column(children) => BView::column(
                    Some(6.0),
                    children.iter().map(|c| node(c, camera)).collect(),
                ),
                BContent::Row(children) => BView::row(
                    Some(16.0),
                    children.iter().map(|c| node(c, camera)).collect(),
                ),
            }
        }

        // Pure loop over the store's static content for this host.
        let content = crate::store::parts::custom::set::content_for(self.host);
        match content.len() {
            0 => BView::empty(),
            1 => node(&content[0], camera),
            _ => BView::column(Some(6.0), content.iter().map(|c| node(c, camera)).collect()),
        }
    }
}
