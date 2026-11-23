//! # NSColorPanel — System Color Picker
//!
//! Native system color picker integration for selecting colors.
//! Provides a modal or non-modal interface for users to select colors using the system's color picker.
//!
//! ## Apple Documentation
//! - [NSColorPanel](https://developer.apple.com/documentation/appkit/nscolorpanel)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use rust_appkit::controls::NSColorPanel;
//! use rust_appkit::core::graphics::NSColor;
//!
//! // Get the shared color panel instance
//! let mut panel = NSColorPanel::shared();
//!
//! // Set the initial color
//! panel.set_color(NSColor::rgb(1.0, 0.0, 0.0));
//!
//! // Show the color panel
//! panel.show();
//!
//! // Set a callback for color changes
//! panel.set_target(Box::new(|color| {
//!     println!("Selected color: R={}, G={}, B={}", color.r, color.g, color.b);
//! }));
//! ```
//!
//! ## Singleton Pattern
//!
//! NSColorPanel follows Apple's singleton pattern where a single shared instance
//! is used throughout the application:
//!
//! ```rust,ignore
//! let panel = NSColorPanel::shared();
//! ```
//!
//! ## See Also
//! - [NSColorWell](https://developer.apple.com/documentation/appkit/nscolorwell)
//! - [NSColor](https://developer.apple.com/documentation/appkit/nscolor)

use crate::graphics::NSColor;
use std::sync::{Arc, Mutex};

/// **NSColorPanel** — A system color picker panel
///
/// Provides native system color picker integration for selecting colors.
/// This is a singleton that integrates with the platform's native color picker.
///
/// ## Apple Documentation
/// - [NSColorPanel](https://developer.apple.com/documentation/appkit/nscolorpanel)
pub struct NSColorPanel {
    color: Arc<Mutex<NSColor>>,
    is_visible: Arc<Mutex<bool>>,
    target: Arc<Mutex<Option<Box<dyn Fn(NSColor) + Send + Sync>>>>,
}

impl NSColorPanel {
    /// Get the shared color panel instance.
    ///
    /// Apple: `[NSColorPanel sharedColorPanel]`
    ///
    /// ## Note
    /// This returns a new instance each time for simplicity. In a full implementation,
    /// this would return a true singleton instance.
    pub fn shared() -> Self {
        Self {
            color: Arc::new(Mutex::new(NSColor::white())),
            is_visible: Arc::new(Mutex::new(false)),
            target: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the current color of the color panel.
    ///
    /// Apple: `setColor:`
    ///
    /// ## Parameters
    /// - `color`: The color to display in the color panel
    pub fn set_color(&mut self, color: NSColor) {
        *self.color.lock().unwrap() = color;
    }

    /// Get the current color of the color panel.
    ///
    /// Apple: `color`
    ///
    /// ## Returns
    /// The currently selected color
    pub fn color(&self) -> NSColor {
        *self.color.lock().unwrap()
    }

    /// Show the color panel.
    ///
    /// Apple: `orderFront:`
    ///
    /// ## Note
    /// This is a stub implementation. In a full implementation, this would:
    /// - Display the native system color picker dialog
    /// - Allow the user to select a color
    /// - Call the target callback when the color changes
    pub fn show(&mut self) {
        *self.is_visible.lock().unwrap() = true;
        let color = *self.color.lock().unwrap();
        
        println!("[NSColorPanel] Showing Color Picker:");
        println!("  Current color: R={:.2}, G={:.2}, B={:.2}, A={:.2}", 
                 color.r, color.g, color.b, color.a);

        // Stub: In a real implementation, this would show a native color picker
        // For now, we just mark it as visible
    }

    /// Hide the color panel.
    ///
    /// Apple: `orderOut:`
    pub fn hide(&mut self) {
        *self.is_visible.lock().unwrap() = false;
        println!("[NSColorPanel] Hiding Color Picker");
    }

    /// Check if the color panel is currently visible.
    ///
    /// Apple: `isVisible`
    ///
    /// ## Returns
    /// `true` if the panel is visible, `false` otherwise
    pub fn is_visible(&self) -> bool {
        *self.is_visible.lock().unwrap()
    }

    /// Set the target callback for color changes.
    ///
    /// The callback will be invoked whenever the user selects a new color.
    ///
    /// Apple: Equivalent to setting target/action pattern
    ///
    /// ## Parameters
    /// - `callback`: A closure that receives the newly selected color
    ///
    /// ## Example
    /// ```rust,ignore
    /// panel.set_target(Box::new(|color| {
    ///     println!("New color selected: {:?}", color);
    /// }));
    /// ```
    pub fn set_target(&mut self, callback: Box<dyn Fn(NSColor) + Send + Sync>) {
        *self.target.lock().unwrap() = Some(callback);
    }

    /// Simulate a color selection (for testing/stub purposes).
    ///
    /// This method is not part of the Apple API but is useful for testing
    /// the callback mechanism in the stub implementation.
    ///
    /// ## Parameters
    /// - `color`: The color to simulate as selected
    pub fn simulate_color_selection(&self, color: NSColor) {
        *self.color.lock().unwrap() = color;
        
        if let Some(callback) = self.target.lock().unwrap().as_ref() {
            callback(color);
        }
    }
}

impl Default for NSColorPanel {
    fn default() -> Self {
        Self::shared()
    }
}

impl Clone for NSColorPanel {
    fn clone(&self) -> Self {
        Self {
            color: Arc::clone(&self.color),
            is_visible: Arc::clone(&self.is_visible),
            target: Arc::clone(&self.target),
        }
    }
}
