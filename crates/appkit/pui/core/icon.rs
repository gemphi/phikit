//! An icon: a mark known by its name. It is drawn, takes no events and
//! holds nothing. The painter finds its picture among the app's assets, at
//! `PIcon::path`, and tints it with the theme's colour.

use super::{PDraw, PPaint, PPoint, PRect, PSize};

/// The theme's token an icon is tinted with.
pub const INK: &str = "icon.ink";

/// A mark known by its name.
#[derive(Debug, Clone, PartialEq)]
pub struct PIcon {
    pub name: String,
    pub size: PSize,
}

impl PIcon {
    pub fn new(name: &str, size: PSize) -> PIcon {
        PIcon { name: name.to_string(), size }
    }

    /// Where the icon's picture is among an app's assets.
    pub fn path(&self) -> String {
        format!("icon/{}.svg", self.name)
    }
}

impl PDraw for PIcon {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let ink = paint.color(INK);
        paint.icon(&self.name, PRect { origin, size: self.size }, ink);
    }
}
