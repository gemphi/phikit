//! IconView — pure icon specification and GPUI Svg renderer.

use gpui::prelude::*;
use gpui::{Hsla, Svg, px, svg};

/// Standardized icon sizes across the editor UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconSize {
    Xs, // 12px
    Sm, // 14px
    Md, // 16px
    Lg, // 20px
    Xl, // 24px
}

impl IconSize {
    pub fn pixels(self) -> f32 {
        match self {
            IconSize::Xs => 12.0,
            IconSize::Sm => 14.0,
            IconSize::Md => 16.0,
            IconSize::Lg => 20.0,
            IconSize::Xl => 24.0,
        }
    }
}

/// Pure icon view specification.
#[derive(Clone, Debug, PartialEq)]
pub struct IconView {
    pub name: String,
    pub size: IconSize,
    pub color: Option<Hsla>,
}

impl IconView {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: IconSize::Md,
            color: None,
        }
    }

    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Convert key name to asset path: "mouse-pointer" -> "icons/mouse_pointer.svg".
    pub fn asset_path(&self) -> String {
        let clean = self.name.replace('-', "_").replace(' ', "_").to_lowercase();
        format!("icons/{}.svg", clean)
    }

    /// Render this IconView to a GPUI `Svg` element.
    pub fn into_element(&self, default_color: Hsla) -> Svg {
        let color = self.color.unwrap_or(default_color);
        svg()
            .path(self.asset_path())
            .size(px(self.size.pixels()))
            .flex_shrink_0()
            .text_color(color)
    }

    /// Helper to render an icon directly.
    pub fn render(name: &str, size: IconSize, color: Hsla) -> Svg {
        Self::new(name).size(size).color(color).into_element(color)
    }
}

/// Helper function to build a GPUI `Svg` element directly from an icon string key.
pub fn render_icon(icon_name: &str, size: IconSize, color: Hsla) -> Svg {
    IconView::render(icon_name, size, color)
}
