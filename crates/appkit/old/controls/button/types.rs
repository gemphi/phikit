//! # NSButton Type Definitions
//!
//! Core types for button controls following Apple's NSButton API.
//!
//! ## Apple Documentation
//! - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)
//! - [NSButtonType](https://developer.apple.com/documentation/appkit/nsbuttontype)
//! - [NSBezelStyle](https://developer.apple.com/documentation/appkit/nsbezelstyle)
//! - [NSImagePosition](https://developer.apple.com/documentation/appkit/nscellimageposition)

use crate::NSColor;

/// Button type — defines button behavior.
///
/// Maps to Apple's [`NSButtonType`](https://developer.apple.com/documentation/appkit/nsbuttontype).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSButtonType {
    /// Standard push button — fires action on click.
    MomentaryPushIn,
    /// Momentary change — shows alternate state while pressed.
    MomentaryChange,
    /// Toggle button — stays pressed until clicked again.
    PushOnPushOff,
    /// Toggle button (alias for PushOnPushOff).
    Toggle,
    /// Checkbox — on/off state with checkmark.
    Switch,
    /// Radio button — mutually exclusive selection.
    Radio,
}

/// Bezel style — defines button visual appearance.
///
/// Maps to Apple's [`NSBezelStyle`](https://developer.apple.com/documentation/appkit/nsbezelstyle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSBezelStyle {
    /// Standard rounded button (most common).
    Rounded,
    /// Square button with border.
    RegularSquare,
    /// Disclosure triangle (expand/collapse).
    Disclosure,
    /// Rounded disclosure button.
    RoundedDisclosure,
    /// Recessed button (sunken appearance).
    Recessed,
    /// Flat inline button (no border until hover).
    Inline,
    /// Textured square button.
    TexturedSquare,
    /// Help button (question mark icon).
    HelpButton,
    /// Circular button.
    Circular,
    /// Textured rounded button (toolbar style).
    TexturedRounded,
    /// Rounded rectangle button.
    RoundRect,
    /// Toolbar button style.
    Toolbar,
    /// Shadowless square button.
    ShadowlessSquare,
}

/// Image position relative to title.
///
/// Maps to Apple's [`NSCellImagePosition`](https://developer.apple.com/documentation/appkit/nscellimageposition).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSImagePosition {
    /// No image displayed.
    NoImage,
    /// Image only, no title.
    ImageOnly,
    /// Image to the left of title.
    Left,
    /// Image to the right of title.
    Right,
    /// Image below title.
    Below,
    /// Image above title.
    Above,
    /// Image overlaps title.
    Overlaps,
    /// Image at leading edge (left in LTR, right in RTL).
    Leading,
    /// Image at trailing edge (right in LTR, left in RTL).
    Trailing,
}

/// Shared button state (internal).
pub(crate) struct NSButtonState {
    pub(crate) title: String,
    pub(crate) button_type: NSButtonType,
    pub(crate) bezel_style: NSBezelStyle,
    pub(crate) image_position: NSImagePosition,
    pub(crate) image: Option<std::sync::Arc<crate::graphics::image::NSImage>>,
    pub(crate) is_hovered: bool,
    pub(crate) is_pressed: bool,
    pub(crate) state: i64,
    pub(crate) accent_color: Option<NSColor>,
}

impl Default for NSButtonState {
    fn default() -> Self {
        Self {
            title: String::new(),
            button_type: NSButtonType::MomentaryPushIn,
            bezel_style: NSBezelStyle::Rounded,
            image_position: NSImagePosition::NoImage,
            image: None,
            is_hovered: false,
            is_pressed: false,
            state: 0,
            accent_color: None,
        }
    }
}
