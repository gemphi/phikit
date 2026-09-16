//! App render bridge — delegates all rendering to BView::into_elm.
//! Zero divs or view construction live here; pure delegation.

use gpui::{AnyElement, Context};
use std::panic::AssertUnwindSafe;
use std::rc::Rc;

use super::PhiberApp;
use crate::bui::bview::{ActionHandler, BAction};

impl PhiberApp {
    pub fn render(&self, tree: &crate::bui::bview::BView, cx: &mut Context<Self>) -> AnyElement {
        let p = self.palette();
        let on_action: ActionHandler = Rc::new(cx.listener(|this, action: &BAction, _w, cx| {
            // Catch any panic in action handling to prevent the app from crashing.
            // A panic in event handling (e.g. re-entrant borrow) would otherwise
            // trigger std::terminate → abort → FAST_FAIL_FATAL_APP_EXIT.
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                this.handle_action(action, cx);
            }));
            if let Err(panic) = result {
                let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                eprintln!("[PANIC CAUGHT] handle_action panicked: {}", msg);
            }
        }));

        tree.into_elm(&p, on_action)
    }
}
