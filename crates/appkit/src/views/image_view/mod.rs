//! # NSImageView — Image Display View
//!
//! A view that displays an image with various scaling options.
//!
//! ## Apple Documentation
//! - [NSImageView](https://developer.apple.com/documentation/appkit/nsimageview)
//! - [NSImage](https://developer.apple.com/documentation/appkit/nsimage)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let image = NSImage::load("icon.png");
//! let mut image_view = NSImageView::image_view_with_image(image);
//! image_view.set_scaling(NSImageScaling::ProportionallyUpOrDown);
//! ```
//!
//! ## See Also
//! - [NSImage](https://developer.apple.com/documentation/appkit/nsimage)
//! - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)

use crate::graphics::image::NSImage;
use crate::{NSColor, NSPoint, NSRect, NSSize, NSView, NSViewDelegate};
use std::sync::{Arc, Mutex};

/// **NSImageScaling** — How the image is scaled to fit the view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSImageScaling {
    ProportionallyDown = 0,
    AxesIndependently = 1,
    None = 2,
    ProportionallyUpOrDown = 3,
}

/// **NSImageView** — A view that displays an image
#[derive(Clone)]
pub struct NSImageView {
    view: NSView,
    inner: Arc<Mutex<NSImageViewState>>,
}

struct NSImageViewState {
    image: Option<Arc<NSImage>>,
    scaling: NSImageScaling,
    animates: bool,
}

impl NSImageView {
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(NSImageViewState {
            image: None,
            scaling: NSImageScaling::ProportionallyUpOrDown,
            animates: false,
        }));

        let mut view = NSView::new(frame);
        view.set_delegate(NSImageViewDelegate {
            state: state.clone(),
        });

        Self { view, inner: state }
    }

    /// **imageViewWithImage** — Factory method
    pub fn image_view_with_image(image: Arc<NSImage>) -> Self {
        let mut iv = Self::new(NSRect::new(
            0.0,
            0.0,
            image.size().width,
            image.size().height,
        ));
        iv.set_image(Some(image));
        iv
    }

    pub fn set_image(&mut self, image: Option<Arc<NSImage>>) {
        self.inner.lock().unwrap().image = image;
        self.view.set_needs_display();
    }

    pub fn image(&self) -> Option<Arc<NSImage>> {
        self.inner.lock().unwrap().image.clone()
    }

    pub fn set_image_scaling(&mut self, scaling: NSImageScaling) {
        self.inner.lock().unwrap().scaling = scaling;
        self.view.set_needs_display();
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }
    pub fn view_clone(&self) -> NSView {
        self.view.clone()
    }
}

struct NSImageViewDelegate {
    state: Arc<Mutex<NSImageViewState>>,
}

impl NSViewDelegate for NSImageViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let bounds = view.bounds();

        if let Some(ref image) = state.image {
            // Calculate target rect based on scaling
            let img_size = image.size();
            let target_rect = match state.scaling {
                NSImageScaling::AxesIndependently => bounds,
                NSImageScaling::None => NSRect::new(
                    bounds.min_x(),
                    bounds.min_y(),
                    img_size.width,
                    img_size.height,
                ),
                NSImageScaling::ProportionallyUpOrDown | NSImageScaling::ProportionallyDown => {
                    let w_ratio = bounds.width() / img_size.width;
                    let h_ratio = bounds.height() / img_size.height;
                    let ratio = w_ratio.min(h_ratio);

                    if state.scaling == NSImageScaling::ProportionallyDown && ratio > 1.0 {
                        // Don't scale up
                        NSRect::new(
                            bounds.mid_x() - img_size.width * 0.5,
                            bounds.mid_y() - img_size.height * 0.5,
                            img_size.width,
                            img_size.height,
                        )
                    } else {
                        let new_w = img_size.width * ratio;
                        let new_h = img_size.height * ratio;
                        NSRect::new(
                            bounds.mid_x() - new_w * 0.5,
                            bounds.mid_y() - new_h * 0.5,
                            new_w,
                            new_h,
                        )
                    }
                }
            };

            context.draw_image(image, target_rect);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
