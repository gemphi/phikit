//! # NSWindow — Window management for AppKit.
//!
//! Provides:
//! - `NSWindow` — the underlying native window (styles, chrome, content)
//! - `NSWindowController` — owns an NSWindow, manages lifecycle
//! - `WindowController` trait — implemented by rust-kit-pro windows
//! - Presentation types: Modal, Sheet, Popover

pub mod actions;
pub mod controller;
pub mod delegate;
pub mod events;
pub mod modal;
pub mod native;
pub mod popover;
pub mod sheet;
pub mod window;

pub use controller::{NSWindowController, NSViewController, WindowController};
pub use delegate::NSWindowDelegate;
pub use modal::ModalPresentation;
pub use popover::{Edge, PopoverPresentation};
pub use sheet::SheetPresentation;
pub use window::NSWindow;
pub use actions::set_cursor;

/// Window style mask values — controls window chrome appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSWindowStyle {
    Borderless,
    Titled,
    Closable,
    Miniaturizable,
    Resizable,
    Utility,
    DocModal,
    FullScreen,
    FullSizeContent,
}
