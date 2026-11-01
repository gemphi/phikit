mod color_palette;
mod registration;
mod token_resolution;
mod variant_loading;

pub use color_palette::{rgb, rgba};
pub use registration::{
    cache_generation, get_color, get_geometry, get_registry, notify_theme_changed, ThemeRegistry,
};
pub use token_resolution::{CustomTheme, Geometry, Palette, ThemeProvider};

#[cfg(test)]
mod tests;
