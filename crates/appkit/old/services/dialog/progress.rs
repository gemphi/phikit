//! Progress indicator for long-running operations.
//!
//! Apple equivalent: `NSProgressIndicator` embedded in an `NSPanel` or sheet.
//! There is no standalone "progress dialog" in AppKit — this is a convenience
//! wrapper that combines an NSPanel + NSProgressIndicator + cancel button.
//!
//! Reference: https://developer.apple.com/documentation/appkit/nsprogressindicator
//! TODO: Rename to NSProgressPanel or embed in SheetPresentation.

/// A progress dialog that displays a progress bar.
pub struct ProgressDialog {
    pub title: String,
    pub message: String,
    pub progress: f32,
    pub cancellable: bool,
    cancelled: bool,
    visible: bool,
}

impl ProgressDialog {
    /// Create a new progress dialog.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            message: String::new(),
            progress: 0.0,
            cancellable: true,
            cancelled: false,
            visible: false,
        }
    }

    /// Update the progress value (0.0 to 1.0).
    pub fn set_progress(&mut self, value: f32) {
        self.progress = value.clamp(0.0, 1.0);
        println!(
            "[stub] ProgressDialog::set_progress({:.1}%) — \"{}\"",
            self.progress * 100.0,
            self.title
        );
    }

    /// Update the status message.
    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        println!("[stub] ProgressDialog::set_message(\"{msg}\")");
    }

    /// Show the progress dialog.
    pub fn show(&mut self) {
        self.visible = true;
        self.cancelled = false;
        println!("[stub] ProgressDialog::show() — \"{}\"", self.title);
    }

    /// Close the progress dialog.
    pub fn close(&mut self) {
        self.visible = false;
        println!("[stub] ProgressDialog::close()");
    }

    /// Check whether the user has cancelled the operation.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}
