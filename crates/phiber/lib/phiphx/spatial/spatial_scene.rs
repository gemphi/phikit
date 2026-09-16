//! 3D Spatial Scene Container for PhiQui GPUI Engine

use crate::phiphx::spatial::mesh_converter::PhiQuiMeshType;
use crate::phiphx::spatial::spatial_bui::{BSpatialButton, BSpatialCard};
use crate::phiphx::spatial::spatial_card::SpatialCard;
use crate::phiphx::spatial::spatial_element::SpatialMeshElement;
use crate::phiphx::spatial::spatial_object::SpatialObject3D;
use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

#[derive(IntoElement)]
pub struct SpatialScene {
    pub active_mesh: PhiQuiMeshType,
    pub objects: Vec<SpatialObject3D>,
}

impl SpatialScene {
    pub fn new() -> Self {
        let objects = vec![
            SpatialObject3D::new("Primary Cube Object", PhiQuiMeshType::Cube),
            SpatialObject3D::new("Secondary Torus Mesh", PhiQuiMeshType::Torus),
            SpatialObject3D::new("Suzanne Monkey Mesh", PhiQuiMeshType::Suzanne),
        ];

        Self {
            active_mesh: PhiQuiMeshType::Cube,
            objects,
        }
    }

    pub fn active_mesh(mut self, mesh: PhiQuiMeshType) -> Self {
        self.active_mesh = mesh;
        self
    }
}

impl RenderOnce for SpatialScene {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let active_obj = SpatialObject3D::new("Active 3D Object", self.active_mesh);

        div()
            .flex_1()
            .flex()
            .flex_col()
            .p(px(16.0))
            .gap(px(12.0))
            .bg(gpui::rgb(0x13111c))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(18.0))
                            .text_color(gpui::rgb(0xa855f7))
                            .child("PhiQui 3D Spatial Viewport Scene"),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(gpui::rgb(0x9ca3af))
                            .child(format!(
                                "3D Mesh: {} (Verts: {}, Faces: {})",
                                active_obj.mesh_type.name(),
                                active_obj.vertices.len(),
                                active_obj.indices.len() / 3
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.0))
                    .child(
                        BSpatialButton::new("Render 3D Mesh", PhiQuiMeshType::Cube).selected(true),
                    )
                    .child(BSpatialButton::new("Morph Topology", PhiQuiMeshType::Torus))
                    .child(BSpatialButton::new(
                        "Monkey Suzanne",
                        PhiQuiMeshType::Suzanne,
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(BSpatialCard::new(
                        "3D Mesh Inspector Panel",
                        self.active_mesh,
                    ))
                    .child(
                        SpatialMeshElement::new("Root Scene Node", PhiQuiMeshType::Cube)
                            .active(true)
                            .child(
                                SpatialMeshElement::new(
                                    "Shader Material Child",
                                    PhiQuiMeshType::Torus,
                                )
                                .child(SpatialMeshElement::new(
                                    "Suzanne Modifier Geometry",
                                    PhiQuiMeshType::Suzanne,
                                )),
                            ),
                    )
                    .child(
                        SpatialCard::new("Outliner Hierarchy").mesh_type(PhiQuiMeshType::Suzanne),
                    ),
            )
    }
}
