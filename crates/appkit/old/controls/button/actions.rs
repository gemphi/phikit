//! # NSButton Actions Implementation
use super::NSButton;

impl NSButton {
    /// Invoke the registered action handler, if any.
    ///
    /// Apple: `[NSControl performClick:]`. Used for programmatic clicks and to simulate
    /// the tail end of a complete mouse-down + mouse-up event sequence.
    pub fn perform_click(&self) {
        let action = self.action.lock().unwrap();
        if let Some(ref handler) = *action {
            handler();
        }
    }

    /// Returns true if an action handler is currently registered.
    pub fn has_action(&self) -> bool {
        self.action.lock().unwrap().is_some()
    }

    /// Fire the registered action by name, logging a warning to stderr if no handler is
    /// registered. This is the single dispatch point for buttons that participate in the
    /// responder chain — the `name` is used purely for the diagnostic log.
    ///
    /// Validates Requirement 13.5: "IF a button has no registered action handler,
    /// THEN THE Renderer SHALL log a warning message indicating the unhandled action."
    pub fn fire_action(&self, name: &str) {
        let handler_opt = { self.action.lock().unwrap().clone() };
        match handler_opt {
            Some(handler) => handler(),
            None => {
                eprintln!("[Action] No handler for button: {}", name);
            }
        }
    }
}
