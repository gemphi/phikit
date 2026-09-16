//! 3D Spatial Card Panel GPUI Element for PhiQui

use crate::phiphx::spatial::mesh_converter::PhiQuiMeshType;
use crate::phiphx::spatial::spatial_animator::SpatialTransform3D;
use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

#[derive(IntoElement)]
pub struct SpatialCard {
    pub title: String,
    pub transform: SpatialTransform3D,
    pub mesh_type: PhiQuiMeshType,
    pub is_selected: bool,
}

impl SpatialCard {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            transform: SpatialTransform3D::default(),
            mesh_type: PhiQuiMeshType::Cube,
            is_selected: false,
        }
    }

    pub fn transform(mut self, transform: SpatialTransform3D) -> Self {
        self.transform = transform;
        self
    }

    pub fn mesh_type(mut self, mesh_type: PhiQuiMeshType) -> Self {
        self.mesh_type = mesh_type;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }
}

impl RenderOnce for SpatialCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let border_color = if self.is_selected {
            gpui::rgb(0xa855f7)
        } else {
            gpui::rgb(0x2b253e)
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .p(px(12.0))
            .rounded(px(8.0))
            .bg(gpui::rgb(0x1f1b2e))
            .border_1()
            .border_color(border_color)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(div().text_color(gpui::rgb(0xf3f4f6)).child(self.title))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(gpui::rgb(0x9ca3af))
                            .child(format!("Mesh: {}", self.mesh_type.name())),
                    ),
            )
    }
}
