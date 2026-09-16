//! Runtime state - keeps UI in sync with specs and user interaction.
//!
//! State modules are singular. Store modules are plural.
//!
//!   context/  → BContext (active space, active area, active host)
//!   host/     → HostState enum + per-host-type state + per-host-type effects
//!   modal/    → ModalState enum + per-modal state (settings/, help/, profile/)
//!   menu/     → MenuState (open dropdowns, checked items)

pub mod context;
pub mod host;
pub mod menu;
pub mod modal;
