//! # NSSplitView — Modular Split View Implementation
//!
//! Mirrors Apple's [`NSSplitView`](https://developer.apple.com/documentation/appkit/nssplitview)
//! from the AppKit framework.  Provides a container that arranges two or more
//! child views along a horizontal or vertical axis, separated by draggable
//! divider lines.
//!
//! ## Architecture
//!
//! | Type                    | Swift Analogue              | Role |
//! |-------------------------|-----------------------------|------|
//! | [`NSSplitView`]         | `NSSplitView`               | The container view that holds child panes |
//! | [`NSSplitViewState`]    | Internal split-view state   | Shared, mutex-guarded divider positions & orientation |
//! | [`NSSplitViewDelegate`] | `NSSplitViewDelegate`       | Draws divider lines and performs layout |
//! | [`NSSplitViewItem`]     | `NSSplitViewItem`           | Wraps a single child view with collapse semantics |
//! | [`NSSplitViewController`] | `NSSplitViewController`   | Higher-level controller (planned) |
//!
//! ## Collapse Behaviour
//!
//! When a child view is hidden via `set_hidden(true)`, the delegate's `layout`
//! pass skips it entirely � matching Swift's `NSSplitViewItem.isCollapsed`.
//! The reclaimed space is redistributed to the remaining visible panes.

pub mod controller;
pub mod delegate;
pub mod divider;
pub mod item;
pub mod resizing;
pub mod view_construction;

// Existing sub-modules from prior splits
pub mod drag_bounds;
pub mod layout;

pub use controller::NSSplitViewController;
pub use divider::{
    DividerChangedCallback, DividerDrag, NSSplitViewDividerStyle, NSSplitViewState,
};
pub use item::{NSSplitViewItem, NSSplitViewItemBehavior};
pub use view_construction::NSSplitView;
