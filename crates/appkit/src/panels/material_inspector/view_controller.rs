//! # NSMaterialInspectorViewController — Material Properties Panel
//!
//! Material inspector view controller for editing material properties.
//!
//! ## Apple Documentation
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//!
//! ## See Also
//! - [NSStackView](https://developer.apple.com/documentation/appkit/nsstackview)
//! - [NSColorWell](https://developer.apple.com/documentation/appkit/nscolorwell)
use crate::prelude::*;
use crate::theme::{keys};

pub struct NSMaterialInspectorViewController {
    pub view: NSView,
}

impl NSMaterialInspectorViewController {
    pub fn new(rect: NSRect) -> Self {
        let mut view = NSView::new(rect).with_name("MaterialInspectorPanel");
        view.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );
        view.set_background_color(Some(crate::theme::color(keys::CLR_BG_PANEL)));
        Self { view }
    }
}
impl NSMaterialInspectorViewController {
    pub fn set_material(&mut self, _material: &dyn std::any::Any) {
        // Display material properties
    }

    pub fn clear_material(&mut self) {
        // Clear the inspector
    }
}
