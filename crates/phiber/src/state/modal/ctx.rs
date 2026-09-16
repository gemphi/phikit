//! Modal context sync - top-level context effects when modals open/close.
//!
//! Modals bypass BContext (they overlay the entire UI), so this module
//! delegates to per-modal ctx.rs files for any modal-specific sync.

use super::ModalState;

/// Sync context when a modal opens or closes.
/// Currently modals don't affect BContext, but this hook exists
/// for consistency with the state/host/ctx.rs pattern.
pub fn sync_modal_change(_state: &ModalState) {
    // Modals overlay the UI and don't modify BContext.
    // Per-modal ctx.rs handles modal-specific sync if needed.
}
