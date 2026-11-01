use std::sync::{Arc, Mutex};

/// Application delegate trait
pub trait NSAppDelegate: Send + Sync {
    fn application_did_finish_launching(&mut self) {}
    fn application_will_terminate(&mut self) {}

    // Support for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn application_should_terminate(&self) -> bool {
        true
    }
    fn application_did_become_active(&mut self) {}
    fn application_will_resign_active(&mut self) {}
    fn application_did_hide(&mut self) {}
    fn application_did_unhide(&mut self) {}
    fn application_open_urls(&mut self, _urls: &[String]) {}
}
