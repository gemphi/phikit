//! # NSButton — Apple's Button Control
//!
//! Button control for Windows AppKit following Apple's NSButton API.
//!
//! ## Apple Documentation
//! - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Create button
//! let mut btn = NSButton::new(NSRect::sized(80.0, 32.0));
//! btn.set_title("Save");
//! btn.set_action(|| save_file());
//!
//! // Factory methods
//! let primary = NSButton::primary("OK", || confirm());
//! let icon = NSButton::toolbar_icon(IconId::Play, || play());
//! ```

pub mod types;
pub mod delegate;
pub mod actions;

#[cfg(test)]
mod tests;

pub use types::{NSBezelStyle, NSButtonType, NSImagePosition};

use crate::control::NSControl;
use crate::{NSRect, NSView, NSColor};
use std::sync::{Arc, Mutex};
use types::NSButtonState;

/// Button control — Apple's NSButton.
///
/// ## Apple Documentation
/// - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)
#[derive(Clone)]
pub struct NSButton {
    pub(crate) control: NSControl,
    pub(crate) inner: Arc<Mutex<NSButtonState>>,
    pub(crate) action: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
}

impl NSButton {
    /// Create a new button with frame.
    ///
    /// Apple: `[[NSButton alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let inner = Arc::new(Mutex::new(NSButtonState::default()));
        let action = Arc::new(Mutex::new(None));
        
        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(delegate::NSButtonViewDelegate {
            state: inner.clone(),
            action: action.clone(),
        });
        
        Self { control, inner, action }
    }

    /// Create a circular button.
    pub fn circle(radius: f32, color: NSColor) -> Self {
        let mut btn = Self::new(NSRect::sized(radius, radius));
        btn.set_button_type(NSButtonType::MomentaryPushIn);
        btn.set_bezel_style(NSBezelStyle::Circular);
        btn.set_accent_color(Some(color));
        btn
    }

    // ── Setters (Apple API) ──────────────────────────────────────────

    /// Set the button title.
    ///
    /// Apple: `setTitle:`
    pub fn set_title(&mut self, title: &str) {
        self.inner.lock().unwrap().title = title.to_string();
    }

    /// Set the button type.
    ///
    /// Apple: `setButtonType:`
    pub fn set_button_type(&mut self, bt: NSButtonType) {
        self.inner.lock().unwrap().button_type = bt;
    }

    /// Set the bezel style.
    ///
    /// Apple: `setBezelStyle:`
    pub fn set_bezel_style(&mut self, style: NSBezelStyle) {
        self.inner.lock().unwrap().bezel_style = style;
    }

    /// Set the button image.
    ///
    /// Apple: `setImage:`
    pub fn set_image(&mut self, image: Option<Arc<crate::graphics::image::NSImage>>) {
        self.inner.lock().unwrap().image = image;
    }

    /// Set the image position relative to title.
    ///
    /// Apple: `setImagePosition:`
    pub fn set_image_position(&mut self, position: NSImagePosition) {
        self.inner.lock().unwrap().image_position = position;
    }

    /// Set the button state (0 = off, 1 = on).
    ///
    /// Apple: `setState:`
    pub fn set_state(&mut self, state: i64) {
        self.inner.lock().unwrap().state = state;
    }

    /// Get the button state.
    ///
    /// Apple: `state`
    pub fn state(&self) -> i64 {
        self.inner.lock().unwrap().state
    }

    /// Set the accent color.
    ///
    /// Apple: `setContentTintColor:` (macOS 10.14+)
    pub fn set_accent_color(&mut self, color: Option<NSColor>) {
        self.inner.lock().unwrap().accent_color = color;
    }

    /// Set the action callback.
    ///
    /// Apple: `setTarget:action:`
    pub fn set_action(&mut self, action: impl Fn() + Send + Sync + 'static) {
        *self.action.lock().unwrap() = Some(Arc::new(action));
    }

    /// Set corner radius (custom extension).
    pub fn set_corner_radius(&mut self, radius: f32) {
        self.control.view_mut().set_corner_radius(radius);
    }

    // ── View Access ──────────────────────────────────────────────────

    /// Get mutable view reference.
    pub fn view_mut(&mut self) -> &mut NSView {
        self.control.view_mut()
    }

    /// Clone the underlying view.
    pub fn view_clone(&self) -> NSView {
        self.control.view().clone()
    }

    // ── Builder Methods ──────────────────────────────────────────────

    /// Set the view name (for debugging).
    pub fn with_name(mut self, name: &str) -> Self {
        self.control.view_mut().set_name(name);
        self
    }

    /// Set the title (builder pattern).
    pub fn with_title(mut self, title: &str) -> Self {
        self.set_title(title);
        self
    }

    /// Set the action (builder pattern).
    pub fn with_action(mut self, action: impl Fn() + Send + Sync + 'static) -> Self {
        self.set_action(action);
        self
    }

    /// Set the bezel style (builder pattern).
    pub fn with_bezel_style(mut self, style: NSBezelStyle) -> Self {
        self.set_bezel_style(style);
        self
    }

    /// Set the button type (builder pattern).
    pub fn with_button_type(mut self, bt: NSButtonType) -> Self {
        self.set_button_type(bt);
        self
    }

    /// Set the image (builder pattern).
    pub fn with_image(mut self, image: Option<Arc<crate::graphics::image::NSImage>>) -> Self {
        self.set_image(image);
        self
    }

    /// Set the image position (builder pattern).
    pub fn with_image_position(mut self, position: NSImagePosition) -> Self {
        self.set_image_position(position);
        self
    }

    /// Set the button state (builder pattern).
    pub fn with_state(mut self, state: i64) -> Self {
        self.set_state(state);
        self
    }

    /// Set the accent color (builder pattern).
    pub fn with_accent_color(mut self, color: Option<NSColor>) -> Self {
        self.set_accent_color(color);
        self
    }

    /// Set the corner radius (builder pattern).
    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.set_corner_radius(radius);
        self
    }

    /// Set the frame (builder pattern).
    pub fn with_frame(mut self, frame: NSRect) -> Self {
        self.control.view_mut().set_frame(frame);
        self
    }

    /// Set the width (builder pattern).
    pub fn with_width(mut self, width: f32) -> Self {
        let current = self.control.view().frame();
        self.control.view_mut().set_frame(NSRect::new(current.x(), current.y(), width, current.height()));
        self
    }

    /// Set the height (builder pattern).
    pub fn with_height(mut self, height: f32) -> Self {
        let current = self.control.view().frame();
        self.control.view_mut().set_frame(NSRect::new(current.x(), current.y(), current.width(), height));
        self
    }

    /// Set the background color (builder pattern).
    pub fn with_background_color(mut self, color: Option<NSColor>) -> Self {
        self.control.view_mut().set_background_color(color);
        self
    }

    /// Set enabled state (builder pattern).
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.control.set_enabled(enabled);
        self
    }
    
    /// Set border color (builder pattern).
    pub fn with_border_color(mut self, color: Option<NSColor>) -> Self {
        self.control.view_mut().set_border_color(color);
        self
    }
    
    /// Set border width (builder pattern).
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.control.view_mut().set_border_width(width);
        self
    }

    // ── Factory Methods (Apple-style class methods) ──────────────────

    /// Standard rounded push button.
    ///
    /// Apple: `+buttonWithTitle:target:action:`
    pub fn primary(title: &str, action: impl Fn() + Send + Sync + 'static) -> Self {
        let mut btn = Self::new(NSRect::sized(80.0, 32.0));
        btn.set_title(title);
        btn.set_bezel_style(NSBezelStyle::Rounded);
        btn.set_action(action);
        btn
    }

    /// Checkbox button.
    ///
    /// Apple: `+checkboxWithTitle:target:action:`
    pub fn checkbox(title: &str, action: impl Fn(bool) + Send + Sync + 'static) -> Self {
        let mut btn = Self::new(NSRect::sized(200.0, 24.0));
        btn.set_title(title);
        btn.set_button_type(NSButtonType::Switch);
        btn.set_bezel_style(NSBezelStyle::Rounded);
        
        // Wrap action to pass boolean state
        let state_ref = btn.inner.clone();
        btn.set_action(move || {
            let is_on = state_ref.lock().unwrap().state == 1;
            action(is_on);
        });
        btn
    }

    /// Radio button.
    ///
    /// Apple: `+radioButtonWithTitle:target:action:`
    pub fn radio(title: &str, action: impl Fn() + Send + Sync + 'static) -> Self {
        let mut btn = Self::new(NSRect::sized(200.0, 24.0));
        btn.set_title(title);
        btn.set_button_type(NSButtonType::Radio);
        btn.set_bezel_style(NSBezelStyle::Rounded);
        btn.set_action(action);
        btn
    }

    /// Toggle button (stays pressed).
    ///
    /// Apple: NSButtonType::PushOnPushOff
    pub fn toggle(title: &str, action: impl Fn(bool) + Send + Sync + 'static) -> Self {
        let mut btn = Self::new(NSRect::sized(80.0, 32.0));
        btn.set_title(title);
        btn.set_button_type(NSButtonType::PushOnPushOff);
        btn.set_bezel_style(NSBezelStyle::Rounded);
        
        // Wrap action to pass boolean state
        let state_ref = btn.inner.clone();
        btn.set_action(move || {
            let is_on = state_ref.lock().unwrap().state == 1;
            action(is_on);
        });
        btn
    }

    /// Disclosure triangle (expand/collapse).
    ///
    /// Apple: NSButton with disclosure bezel style
    pub fn disclosure(expanded: bool, action: impl Fn(bool) + Send + Sync + 'static) -> Self {
        let size = 16.0;
        let mut btn = Self::new(NSRect::sized(size, size));
        btn.set_button_type(NSButtonType::PushOnPushOff);
        btn.set_bezel_style(NSBezelStyle::Disclosure);
        btn.set_state(if expanded { 1 } else { 0 });
        
        // Wrap action to pass boolean state
        let state_ref = btn.inner.clone();
        btn.set_action(move || {
            let is_expanded = state_ref.lock().unwrap().state == 1;
            action(is_expanded);
        });
        btn
    }

    /// Toolbar icon button.
    pub fn toolbar_icon(icon_id: crate::svg_icons::IconId, action: impl Fn() + Send + Sync + 'static) -> Self {
        let size = 32.0;
        let mut btn = Self::new(NSRect::sized(size, size));
        
        if let Some(icon) = crate::svg_icons::get_sized(icon_id, 16.0, 16.0) {
            btn.set_image(Some(icon));
            btn.set_image_position(NSImagePosition::ImageOnly);
        }
        
        btn.set_bezel_style(NSBezelStyle::TexturedRounded);
        btn.set_action(action);
        btn
    }

    /// Activity bar icon button.
    pub fn activity_bar(
        icon_id: crate::svg_icons::IconId,
        is_active: bool,
        action: impl Fn() + Send + Sync + 'static
    ) -> Self {
        let size = 48.0;
        let mut btn = Self::new(NSRect::sized(size, size));
        
        if let Some(icon) = crate::svg_icons::get_sized(icon_id, 24.0, 24.0) {
            btn.set_image(Some(icon));
            btn.set_image_position(NSImagePosition::ImageOnly);
        }
        
        if is_active {
            btn.set_state(1);
        }
        
        btn.set_bezel_style(NSBezelStyle::Inline);
        btn.set_corner_radius(6.0);
        btn.set_action(action);
        btn
    }
}
