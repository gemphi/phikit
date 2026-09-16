//! Spatial BUIKit Components Constructed From 3D Spatial Mesh Objects

use crate::phiphx::spatial::mesh_converter::PhiQuiMeshType;
use crate::phiphx::spatial::spatial_object::SpatialObject3D;
use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

#[derive(IntoElement)]
pub struct BSpatialButton {
    pub label: String,
    pub mesh: SpatialObject3D,
    pub selected: bool,
}

impl BSpatialButton {
    pub fn new(label: impl Into<String>, mesh_type: PhiQuiMeshType) -> Self {
        let label = label.into();
        let mesh = SpatialObject3D::new(format!("Btn_{}", label), mesh_type);
        Self {
            label,
            mesh,
            selected: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl RenderOnce for BSpatialButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let bg = if self.selected {
            gpui::rgb(0xa855f7)
        } else {
            gpui::rgb(0x1f1b2e)
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .px(px(12.0))
            .py(px(6.0))
            .rounded(px(6.0))
            .bg(bg)
            .border_1()
            .border_color(gpui::rgb(0x342d4a))
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(gpui::rgb(0xffffff))
                    .child(self.label),
            )
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(gpui::rgb(0x9ca3af))
                    .child(format!("[{}]", self.mesh.mesh_type.name())),
            )
    }
}

#[derive(IntoElement)]
pub struct BSpatialCard {
    pub title: String,
    pub mesh: SpatialObject3D,
}

impl BSpatialCard {
    pub fn new(title: impl Into<String>, mesh_type: PhiQuiMeshType) -> Self {
        let title = title.into();
        let mesh = SpatialObject3D::new(format!("Card_{}", title), mesh_type);
        Self { title, mesh }
    }
}

impl RenderOnce for BSpatialCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .p(px(14.0))
            .gap(px(6.0))
            .rounded(px(8.0))
            .bg(gpui::rgb(0x191624))
            .border_1()
            .border_color(gpui::rgb(0x2b253e))
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(gpui::rgb(0xf3f4f6))
                    .child(self.title),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(gpui::rgb(0xa855f7))
                    .child(format!(
                        "3D Mesh Object: {} ({} vertices, {} faces)",
                        self.mesh.mesh_type.name(),
                        self.mesh.vertices.len(),
                        self.mesh.indices.len() / 3
                    )),
            )
    }
}
