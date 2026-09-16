//! 3D Spatial UI Elements Constructed Directly & Recursively From Geometry Mesh Objects

use crate::phiphx::spatial::mesh_converter::PhiQuiMeshType;
use crate::phiphx::spatial::spatial_object::SpatialObject3D;
use gpui::{AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

#[derive(IntoElement)]
pub struct SpatialMeshElement {
    pub label: String,
    pub mesh_object: SpatialObject3D,
    pub is_active: bool,
    pub children: Vec<AnyElement>,
}

impl SpatialMeshElement {
    pub fn new(label: impl Into<String>, mesh_type: PhiQuiMeshType) -> Self {
        let label = label.into();
        let mesh_object = SpatialObject3D::new(format!("Mesh_{}", label), mesh_type);
        Self {
            label,
            mesh_object,
            is_active: false,
            children: Vec::new(),
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Add a child spatial element recursively
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for SpatialMeshElement {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let active_bg = if self.is_active {
            gpui::rgb(0x262138)
        } else {
            gpui::rgb(0x191624)
        };

        let mut node = div()
            .flex()
            .flex_col()
            .p(px(10.0))
            .gap(px(6.0))
            .rounded(px(6.0))
            .bg(active_bg)
            .border_1()
            .border_color(gpui::rgb(0x342d4a))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(gpui::rgb(0xf3f4f6))
                            .child(self.label),
                    )
                    .child(
                        div()
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .bg(gpui::rgb(0xa855f7))
                            .text_size(px(11.0))
                            .text_color(gpui::rgb(0xffffff))
                            .child(format!(
                                "3D Mesh: {} ({} verts)",
                                self.mesh_object.mesh_type.name(),
                                self.mesh_object.vertices.len()
                            )),
                    ),
            );

        if !self.children.is_empty() {
            let mut container = div().flex().flex_col().gap(px(4.0)).pl(px(12.0));
            for child in self.children {
                container = container.child(child);
            }
            node = node.child(container);
        }

        node
    }
}
